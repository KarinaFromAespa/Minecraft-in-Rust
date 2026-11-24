//! The remapping stage for obfuscated releases (everything before 26.1).
//!
//! Mojang publishes obfuscated jars for these releases, and the exporters read named
//! classes, so prepare renames the imported jars with Mojang's official mapping tables
//! before anything downstream runs. This is the Rust port of the pipeline's remapping
//! stage (`tools/lab.py`'s `remap_stage`, driven by the mappings front-end's `Remap`
//! driver): same inputs (the verified jars, Mojang's tables from the manifest's
//! `client_mappings`/`server_mappings` downloads), same tool (Fabric's tiny-remapper
//! 0.15.0 through the same driver), same outputs (`remapped.json`,
//! `server-mapped-classes.tsv`, `server-mappings.txt`) at the paths
//! `artifacts::verify` already authenticates. Unobfuscated releases (26.x) publish no
//! mapping tables, so the stage is a no-op for them and their import is exactly as
//! before.
//!
//! Nothing here is committed: the driver source is embedded (Java source, like the
//! exporter sources, compiled at prepare time), and the mapping tables and the
//! remapper's jars are downloaded at prepare time into `.local/` and checked against
//! their SHA-1s.

use crate::artifacts::Lock;
use crate::java::Jdk;
use crate::pyjson::Value;
use crate::{Log, digest, mkdir, sha256_hex, took, write_json};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The remapper's jars, pinned like the front-end pins them
/// (`tools/mapping-frontend/frontend.py`'s `DEPS`): name, URL, SHA-1 (each checked
/// against the checksum Maven publishes beside the jar). Downloaded into
/// `.local/mapping-frontend/deps/` at prepare time, never committed.
const DEPS: &[(&str, &str, &str)] = &[
    (
        "tiny-remapper-0.15.0.jar",
        "https://maven.fabricmc.net/net/fabricmc/tiny-remapper/0.15.0/tiny-remapper-0.15.0.jar",
        "25df4de29d20b4627d7e9fa2a25e356d000da8c0",
    ),
    (
        "mapping-io-0.7.1.jar",
        "https://maven.fabricmc.net/net/fabricmc/mapping-io/0.7.1/mapping-io-0.7.1.jar",
        "e96e1dda50d631c89f83ad5c2c951c3f2298e7c2",
    ),
    (
        "asm-9.9.1.jar",
        "https://repo1.maven.org/maven2/org/ow2/asm/asm/9.9.1/asm-9.9.1.jar",
        "2ceea6ab43bcae1979b2a6d85fc0ca429877e5ab",
    ),
    (
        "asm-tree-9.9.1.jar",
        "https://repo1.maven.org/maven2/org/ow2/asm/asm-tree/9.9.1/asm-tree-9.9.1.jar",
        "b6b1b3366296163b4b1f540731aad0a2baa484d8",
    ),
    (
        "asm-util-9.9.1.jar",
        "https://repo1.maven.org/maven2/org/ow2/asm/asm-util/9.9.1/asm-util-9.9.1.jar",
        "e51f5b0ae0b0c1960687ae970a2a3434d39d8abb",
    ),
    (
        "asm-analysis-9.9.1.jar",
        "https://repo1.maven.org/maven2/org/ow2/asm/asm-analysis/9.9.1/asm-analysis-9.9.1.jar",
        "1ab8d9316ef7a67240087919a708246c37ed1660",
    ),
    (
        "asm-commons-9.9.1.jar",
        "https://repo1.maven.org/maven2/org/ow2/asm/asm-commons/9.9.1/asm-commons-9.9.1.jar",
        "ab35de4c537184a09339069f1a3b3aacf2289149",
    ),
];

const USER_AGENT: &str = "horizon/0.1";

/// The mappings front-end's remap driver, verbatim from
/// `tools/mapping-frontend/src/Remap.java` (refresh it by copying that file in full
/// when the front-end changes it; the `the_embedded_driver_is_the_front_ends_driver`
/// test guards against a mangled copy). Java source, like the exporter sources:
/// compiled at prepare time into `.local/mapping-frontend/build/classes/`, never
/// committed as classes.
const REMAP_JAVA: &str = r##"import java.io.BufferedReader;
import java.io.BufferedWriter;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.regex.Pattern;

import net.fabricmc.mappingio.MappingReader;
import net.fabricmc.mappingio.tree.MappingTree;
import net.fabricmc.mappingio.tree.MemoryMappingTree;
import net.fabricmc.tinyremapper.IMappingProvider;
import net.fabricmc.tinyremapper.OutputConsumerPath;
import net.fabricmc.tinyremapper.TinyRemapper;

/**
 * Remap driver for the mappings front-end.
 *
 * Reads a mapping table with Fabric's mapping-io (the normalization layer:
 * ProGuard txt, SRG/TSRG, tiny v1/v2), works out which namespace is the
 * obfuscated one and which is the named one, then applies the table in the
 * obfuscated -&gt; named direction with tiny-remapper. Tables carrying an
 * intermediate namespace (e.g. tiny v2 official/intermediary/named) are
 * remapped as a chain of passes, so a member missing a named name still
 * lands on its intermediate name instead of staying obfuscated.
 *
 * Usage: Remap &lt;mappings-file&gt; &lt;input.jar&gt; &lt;output.jar&gt; &lt;stats-file&gt;
 *
 * The stats file gets one line per mapped class: "&lt;obfuscated&gt;\t&lt;named&gt;".
 * A JSON coverage report is written next to it (same stem, .json suffix, or
 * the path given by -Dmf.report): class/method/field totals, how many carry
 * a name in the target namespace, how many actually change name, and how
 * many target names are still placeholders (SRG func_/field_ names or Yarn
 * intermediary class_/method_/field_ names). That is the health check
 * gappy tables need: class-level coverage alone says nothing about members.
 *
 * Direction detection, in order:
 *
 * <ol>
 *   <li>Explicit overrides: -Dmf.from=&lt;ns&gt; -Dmf.to=&lt;ns&gt;. Either may be
 *       given alone; the other side is then detected. Both given skips
 *       detection entirely (this is how the pipeline's server-jar path
 *       drives the driver).</li>
 *   <li>Probe classes. A probe decides the target namespace when it carries
 *       its real name in exactly one namespace; with several matches the
 *       namespaces literally called "named" then "official" win. Probes are
 *       tried per side (-Dmf.side=client|server|auto, default auto: client
 *       probes first, then server probes). The source namespace is then the
 *       namespace where the deciding probe class carries a different name
 *       than it does in the target namespace; with several candidates the
 *       one whose probe name is not under net/minecraft/ (truly obfuscated)
 *       wins, then "official".</li>
 * </ol>
 *
 * Probe lists (tried in order; the first that discriminates decides):
 *
 * <ul>
 *   <li>Client: net/minecraft/client/Minecraft, net/minecraft/client/player/LocalPlayer</li>
 *   <li>Server: net/minecraft/server/MinecraftServer, net/minecraft/server/level/ServerLevel,
 *       net/minecraft/server/players/PlayerList</li>
 * </ul>
 *
 * Note on MinecraftServer: Mojang keeps it unobfuscated, so it is
 * identity-mapped in the 1.21.11 tables (present under its real name in
 * both namespaces) and never discriminates on its own. It stays first in
 * the server list because it verifies the table is a server-side one; the
 * obfuscated server classes behind it (ServerLevel -&gt; axf, PlayerList
 * -&gt; bbz in 1.21.11) do the deciding. If no probe can decide, the driver
 * fails naming every probe tried and what each found, instead of guessing.
 */
public final class Remap {
    /** Client-side probe classes, tried in order. */
    private static final String[] CLIENT_PROBES = {
        "net/minecraft/client/Minecraft",
        "net/minecraft/client/player/LocalPlayer",
    };

    /**
     * Server-side probe classes, tried in order. MinecraftServer is first
     * but identity-mapped in Mojang's tables (see class Javadoc); the
     * obfuscated classes behind it decide.
     */
    private static final String[] SERVER_PROBES = {
        "net/minecraft/server/MinecraftServer",
        "net/minecraft/server/level/ServerLevel",
        "net/minecraft/server/players/PlayerList",
    };

    /** Target-namespace tiebreak when several namespaces carry the probe's real name. */
    private static final String[] TARGET_PREFERENCE = {"named", "official"};

    /** Source-namespace tiebreak when several namespaces carry the probe obfuscated. */
    private static final String[] SOURCE_PREFERENCE = {"official", "obfuscated"};

    /** SRG-style placeholder member names (MCP/NeoForm tables). */
    private static final Pattern SRG_METHOD = Pattern.compile("func_\\d+_.*");
    private static final Pattern SRG_FIELD = Pattern.compile("field_\\d+_.*");

    /** Yarn intermediary placeholder names. */
    private static final Pattern INTERMEDIARY_MEMBER = Pattern.compile("(method|field)_\\d+");
    private static final Pattern INTERMEDIARY_CLASS = Pattern.compile("class_\\d+");

    /** Coverage counts for one member kind, from-namespace -&gt; to-namespace. */
    private static final class Counts {
        int total;       // entries with a from-namespace name
        int mapped;      // ... that also have a to-namespace name
        int renamed;     // ... whose to-namespace name differs
        int placeholders; // ... whose to-namespace name is a placeholder
        int fallback;    // ... with no to-namespace name but an intermediate one

        String toJson() {
            return "{\"total\": " + total
                    + ", \"mapped\": " + mapped
                    + ", \"renamed\": " + renamed
                    + ", \"placeholders\": " + placeholders
                    + ", \"fallback_to_intermediate\": " + fallback + "}";
        }
    }

    private static boolean isPlaceholderClass(String name) {
        String simple = name.substring(name.lastIndexOf('/') + 1);
        return INTERMEDIARY_CLASS.matcher(simple).matches();
    }

    private static boolean isPlaceholderMember(String name) {
        return SRG_METHOD.matcher(name).matches()
                || SRG_FIELD.matcher(name).matches()
                || INTERMEDIARY_MEMBER.matcher(name).matches();
    }

    private static String jsonEscape(String s) {
        return s.replace("\\", "\\\\").replace("\"", "\\\"");
    }

    public static void main(String[] args) throws Exception {
        if (args.length != 4) {
            System.err.println("usage: Remap <mappings-file> <input.jar> <output.jar> <stats-file>");
            System.exit(2);
        }
        Path mappingsFile = Paths.get(args[0]);
        Path inputJar = Paths.get(args[1]);
        Path outputJar = Paths.get(args[2]);
        Path statsFile = Paths.get(args[3]);

        MemoryMappingTree tree = new MemoryMappingTree();
        try (BufferedReader reader = Files.newBufferedReader(mappingsFile)) {
            MappingReader.read(reader, tree);
        }

        // Namespace order is the tree's own: source first, then destinations
        // in file order. The remap chain follows this order (either
        // direction) between the resolved source and target.
        List<String> ordered = new ArrayList<>();
        ordered.add(tree.getSrcNamespace());
        ordered.addAll(tree.getDstNamespaces());
        System.out.println("namespaces: " + ordered);
        System.out.println("classes in mapping table: " + tree.getClasses().size());

        String forcedFrom = System.getProperty("mf.from");
        String forcedTo = System.getProperty("mf.to");
        if (forcedFrom != null && !ordered.contains(forcedFrom)) {
            throw new IllegalStateException(
                    "-Dmf.from=" + forcedFrom + " is not a namespace in this table; available: " + ordered);
        }
        if (forcedTo != null && !ordered.contains(forcedTo)) {
            throw new IllegalStateException(
                    "-Dmf.to=" + forcedTo + " is not a namespace in this table; available: " + ordered);
        }
        String side = System.getProperty("mf.side", "auto");
        if (!side.equals("auto") && !side.equals("client") && !side.equals("server")) {
            throw new IllegalStateException(
                    "-Dmf.side must be client, server or auto, got '" + side + "'");
        }
        List<String> probes = new ArrayList<>();
        if (side.equals("auto") || side.equals("client")) {
            probes.addAll(Arrays.asList(CLIENT_PROBES));
        }
        if (side.equals("auto") || side.equals("server")) {
            probes.addAll(Arrays.asList(SERVER_PROBES));
        }

        // Resolve the target namespace: explicit override, else the first
        // probe that discriminates.
        String toNs = forcedTo;
        String decidingProbe = null;
        String decidingSide = null;
        StringBuilder probeLog = new StringBuilder();
        if (toNs == null) {
            for (String probe : probes) {
                List<String> matches = new ArrayList<>();
                for (String ns : ordered) {
                    if (tree.getClass(probe, tree.getNamespaceId(ns)) != null) {
                        matches.add(ns);
                    }
                }
                probeLog.append("  probe ").append(probe)
                        .append(": real name found in ").append(matches).append("\n");
                if (matches.isEmpty()) {
                    continue;
                }
                if (matches.size() == 1) {
                    toNs = matches.get(0);
                } else {
                    for (String preferred : TARGET_PREFERENCE) {
                        if (matches.contains(preferred)) {
                            toNs = preferred;
                            break;
                        }
                    }
                    if (toNs == null) {
                        continue; // ambiguous for this probe; try the next
                    }
                }
                decidingProbe = probe;
                decidingSide = Arrays.asList(CLIENT_PROBES).contains(probe) ? "client" : "server";
                break;
            }
            if (toNs == null) {
                throw new IllegalStateException(
                        "cannot detect direction: no probe class carries its real name in exactly one namespace.\n"
                        + probeLog
                        + "Set -Dmf.from/-Dmf.to to force a direction, or -Dmf.side to pick a probe side.");
            }
        }

        // Resolve the source namespace: explicit override, else the
        // namespace where the deciding probe class carries a different name
        // than it does in the target namespace. (When the target was
        // detected via a probe, the target name IS the probe's real name;
        // when the target was forced to an intermediate namespace, the
        // comparison is against the probe's intermediate name instead.)
        String fromNs = forcedFrom;
        if (fromNs == null) {
            MappingTree.ClassMapping probeCls = null;
            if (decidingProbe != null) {
                probeCls = tree.getClass(decidingProbe, tree.getNamespaceId(toNs));
            } else {
                // Target was forced: find a probe whose class exists in the
                // table and carries a name in the forced target namespace.
                for (String probe : probes) {
                    for (String ns : ordered) {
                        MappingTree.ClassMapping cls =
                                tree.getClass(probe, tree.getNamespaceId(ns));
                        if (cls != null && cls.getName(tree.getNamespaceId(toNs)) != null) {
                            decidingProbe = probe;
                            decidingSide = Arrays.asList(CLIENT_PROBES).contains(probe)
                                    ? "client" : "server";
                            probeCls = cls;
                            break;
                        }
                    }
                    if (probeCls != null) {
                        break;
                    }
                }
                if (probeCls == null) {
                    throw new IllegalStateException(
                            "cannot detect the source namespace: none of the probe classes "
                            + probes + " has a name in the forced target namespace '"
                            + toNs + "'. Set -Dmf.from to force the source.");
                }
            }
            String targetName = probeCls.getName(tree.getNamespaceId(toNs));
            List<String> candidates = new ArrayList<>();
            for (String ns : ordered) {
                if (ns.equals(toNs)) {
                    continue;
                }
                String name = probeCls.getName(tree.getNamespaceId(ns));
                if (name != null && !name.equals(targetName)) {
                    candidates.add(ns);
                }
            }
            if (candidates.isEmpty()) {
                throw new IllegalStateException(
                        "cannot detect the source namespace: probe class " + decidingProbe
                        + " carries the same name ('" + targetName + "') in every namespace"
                        + " (it may be identity-mapped). Set -Dmf.from to force the source.");
            }
            if (candidates.size() == 1) {
                fromNs = candidates.get(0);
            } else {
                for (String cand : candidates) {
                    String name = probeCls.getName(tree.getNamespaceId(cand));
                    if (name != null && !name.startsWith("net/minecraft/")) {
                        fromNs = cand; // truly obfuscated, not an intermediate
                        break;
                    }
                }
                if (fromNs == null) {
                    for (String preferred : SOURCE_PREFERENCE) {
                        if (candidates.contains(preferred)) {
                            fromNs = preferred;
                            break;
                        }
                    }
                }
                if (fromNs == null) {
                    fromNs = candidates.get(0);
                }
            }
        }
        if (fromNs.equals(toNs)) {
            throw new IllegalStateException(
                    "source and target namespace are the same ('" + fromNs + "'); nothing to remap");
        }
        System.out.println("direction: " + fromNs + " (obfuscated) -> " + toNs + " (named)"
                + (decidingProbe != null
                        ? " [probe " + decidingProbe + ", " + decidingSide + "]" : " [forced]"));

        // The chain is the namespaces in tree order between source and
        // target (either direction). Adjacent namespaces mean one pass.
        int fromIdx = ordered.indexOf(fromNs);
        int toIdx = ordered.indexOf(toNs);
        List<String> chain = new ArrayList<>();
        if (fromIdx < toIdx) {
            chain.addAll(ordered.subList(fromIdx, toIdx + 1));
        } else {
            for (int i = fromIdx; i >= toIdx; i--) {
                chain.add(ordered.get(i));
            }
        }
        if (chain.size() > 2) {
            System.out.println("chain: " + chain);
        }
        List<String> intermediates = chain.subList(1, chain.size() - 1);

        final String from = fromNs;
        final String to = toNs;

        // Coverage counts over the whole tree, from-namespace -> to-namespace.
        Counts classes = new Counts();
        Counts methods = new Counts();
        Counts fields = new Counts();
        for (MappingTree.ClassMapping cls : tree.getClasses()) {
            String cf = cls.getName(from);
            if (cf == null) {
                continue;
            }
            classes.total++;
            String ct = cls.getName(to);
            if (ct != null) {
                classes.mapped++;
                if (!cf.equals(ct)) {
                    classes.renamed++;
                }
                if (isPlaceholderClass(ct)) {
                    classes.placeholders++;
                }
            }
            for (MappingTree.FieldMapping field : cls.getFields()) {
                countMember(field.getName(from), field.getName(to), fields,
                        () -> hasIntermediateName(field, intermediates));
            }
            for (MappingTree.MethodMapping method : cls.getMethods()) {
                countMember(method.getName(from), method.getName(to), methods,
                        () -> hasIntermediateName(method, intermediates));
            }
        }

        // Dump the class table for the caller's statistics (unchanged format).
        try (BufferedWriter w = Files.newBufferedWriter(statsFile)) {
            for (MappingTree.ClassMapping cls : tree.getClasses()) {
                String f = cls.getName(from);
                String t = cls.getName(to);
                if (f == null || t == null) {
                    continue;
                }
                w.write(f + "\t" + t + "\n");
            }
        }

        // JSON coverage report next to the stats file (same stem, .json),
        // or at -Dmf.report when given.
        String reportProp = System.getProperty("mf.report");
        Path reportFile;
        if (reportProp != null) {
            reportFile = Paths.get(reportProp);
        } else {
            String name = statsFile.getFileName().toString();
            String base = name.endsWith(".tsv") ? name.substring(0, name.length() - 4) : name;
            reportFile = statsFile.resolveSibling(base + ".json");
        }
        int placeholdersTotal = classes.placeholders + methods.placeholders + fields.placeholders;
        StringBuilder json = new StringBuilder("{\n");
        json.append("  \"from_namespace\": \"").append(jsonEscape(from)).append("\",\n");
        json.append("  \"to_namespace\": \"").append(jsonEscape(to)).append("\",\n");
        json.append("  \"chain\": [");
        for (int i = 0; i < chain.size(); i++) {
            if (i > 0) {
                json.append(", ");
            }
            json.append("\"").append(jsonEscape(chain.get(i))).append("\"");
        }
        json.append("],\n");
        json.append("  \"probe\": ")
                .append(decidingProbe == null ? "null" : "\"" + jsonEscape(decidingProbe) + "\"")
                .append(",\n");
        json.append("  \"side\": ")
                .append(decidingSide == null ? "\"forced\"" : "\"" + decidingSide + "\"")
                .append(",\n");
        json.append("  \"classes\": ").append(classes.toJson()).append(",\n");
        json.append("  \"methods\": ").append(methods.toJson()).append(",\n");
        json.append("  \"fields\": ").append(fields.toJson()).append(",\n");
        json.append("  \"placeholders_total\": ").append(placeholdersTotal).append("\n");
        json.append("}\n");
        Files.writeString(reportFile, json.toString());

        System.out.println("coverage (" + from + " -> " + to + "):");
        System.out.println("  classes: " + classes.mapped + " mapped / " + classes.total + " total, "
                + classes.renamed + " renamed, " + classes.placeholders + " placeholders");
        System.out.println("  methods: " + methods.mapped + " mapped / " + methods.total + " total, "
                + methods.renamed + " renamed, " + methods.placeholders + " placeholders, "
                + methods.fallback + " fallback to intermediate");
        System.out.println("  fields:  " + fields.mapped + " mapped / " + fields.total + " total, "
                + fields.renamed + " renamed, " + fields.placeholders + " placeholders, "
                + fields.fallback + " fallback to intermediate");
        System.out.println("coverage report: " + reportFile);

        // Remap along the chain, one tiny-remapper pass per leg. Intermediate
        // jars are temporary and deleted afterwards.
        List<Path> temps = new ArrayList<>();
        try {
            Path current = inputJar;
            for (int i = 0; i < chain.size() - 1; i++) {
                boolean last = (i == chain.size() - 2);
                Path out;
                if (last) {
                    out = outputJar;
                } else {
                    out = Files.createTempFile(
                            outputJar.toAbsolutePath().getParent(),
                            outputJar.getFileName().toString() + ".chain", ".jar");
                    // tiny-remapper opens the output path as a zip filesystem,
                    // and a 0-byte file is not a valid zip: remove the empty
                    // file createTempFile just made, keeping its unique name.
                    Files.deleteIfExists(out);
                    temps.add(out);
                }
                remapPass(tree, chain.get(i), chain.get(i + 1), current, out);
                if (!current.equals(inputJar)) {
                    Files.deleteIfExists(current);
                    temps.remove(current);
                }
                current = out;
            }
        } finally {
            for (Path temp : temps) {
                Files.deleteIfExists(temp);
            }
        }
        System.out.println("wrote " + outputJar);
    }

    private static void countMember(String fromName, String toName, Counts counts,
                                    java.util.function.BooleanSupplier hasIntermediate) {
        if (fromName == null) {
            return;
        }
        counts.total++;
        if (toName != null) {
            counts.mapped++;
            if (!fromName.equals(toName)) {
                counts.renamed++;
            }
            if (isPlaceholderMember(toName)) {
                counts.placeholders++;
            }
        } else if (hasIntermediate.getAsBoolean()) {
            counts.fallback++;
        }
    }

    private static boolean hasIntermediateName(MappingTree.MemberMapping member,
                                               List<String> intermediates) {
        for (String ns : intermediates) {
            if (member.getName(ns) != null) {
                return true;
            }
        }
        return false;
    }

    /** One tiny-remapper pass from one namespace to the next. */
    private static void remapPass(MappingTree tree, String from, String to,
                                  Path inputJar, Path outputJar) throws Exception {
        IMappingProvider provider = acceptor -> {
            for (MappingTree.ClassMapping cls : tree.getClasses()) {
                String ownerFrom = cls.getName(from);
                String ownerTo = cls.getName(to);
                if (ownerFrom == null || ownerTo == null) {
                    continue;
                }
                acceptor.acceptClass(ownerFrom, ownerTo);
                for (MappingTree.FieldMapping field : cls.getFields()) {
                    String name = field.getName(to);
                    if (name == null) {
                        continue;
                    }
                    acceptor.acceptField(
                            new IMappingProvider.Member(ownerFrom, field.getName(from), field.getDesc(from)),
                            name);
                }
                for (MappingTree.MethodMapping method : cls.getMethods()) {
                    String name = method.getName(to);
                    if (name == null) {
                        continue;
                    }
                    acceptor.acceptMethod(
                            new IMappingProvider.Member(ownerFrom, method.getName(from), method.getDesc(from)),
                            name);
                }
            }
        };

        TinyRemapper remapper = TinyRemapper.newRemapper()
                .withMappings(provider)
                .build();
        try {
            remapper.readInputs(inputJar);
            try (OutputConsumerPath output = new OutputConsumerPath.Builder(outputJar).build()) {
                output.addNonClassFiles(inputJar);
                remapper.apply(output);
            }
        } finally {
            remapper.finish();
        }
    }
}
"##;

/// Remap the imported jars with Mojang's official mapping tables when the manifest
/// publishes them (an obfuscated release), recording what the stage produced in
/// `remapped.json` for `artifacts::verify`. A manifest without mapping tables
/// (unobfuscated) returns at once, having touched nothing and needed no Java.
/// `java_home` is the runtime's home, the first place the compiler lookup tries
/// (the launcher's pick, e.g. Mojang's own Java it downloaded): the driver is
/// compiled with the same JDK the exporters use.
#[allow(clippy::too_many_arguments)]
pub(crate) fn remap_if_needed(
    root: &Path,
    lock: &Lock,
    manifest: &Value,
    client: &Path,
    server: &Path,
    server_from_client: bool,
    java_home: &Path,
    log: Log,
) -> Result<(), String> {
    let downloads = manifest.get("downloads");
    let table = |side: &str| -> Option<(&str, &str)> {
        let download = downloads?.get(format!("{side}_mappings").as_str())?;
        Some((
            download.get("url")?.as_str()?,
            download.get("sha1")?.as_str()?,
        ))
    };
    let client_table = table("client");
    let server_table = table("server");
    if client_table.is_none() && server_table.is_none() {
        return Ok(());
    }
    let started = std::time::Instant::now();
    log(&format!(
        "Remapping {} with Mojang's official mappings",
        lock.version
    ));
    let jdk = crate::java::find(lock.java_major, &[java_home.to_path_buf()])?;
    let home = root.join(".local/mapping-frontend");
    let deps = ensure_deps(&home, log)?;
    let classes = ensure_driver(&home, &deps, &jdk, lock.java_major, log)?;
    let mut record = Vec::new();
    if let Some((url, sha1)) = client_table {
        let version_dir = home.join(&lock.version);
        let table_path = version_dir.join("client-mappings.txt");
        download(url, &table_path, sha1, log)?;
        let named = version_dir.join("client-named.jar");
        let stats = version_dir.join("mapped-classes.tsv");
        remap_one(
            &jdk,
            &classes,
            &deps,
            &table_path,
            client,
            &named,
            &stats,
            &version_dir.join("client-remap.log"),
            &["-Dmf.side=auto"],
            log,
        )?;
        replace_with(&named, client)?;
        strip_signature(client)?;
        record.push((
            "client".to_string(),
            Value::Object(vec![
                ("sha1".into(), Value::Str(digest(client, "sha1")?)),
                ("table_sha1".into(), Value::Str(sha1.to_string())),
            ]),
        ));
        log("Remapped the client jar");
    }
    if let Some((url, sha1)) = server_table {
        // A Prism install has no server jar: the client jar is the server side, already
        // remapped with the client table above, so there is no second jar to remap.
        if !server_from_client {
            let table_path = lock.directory.join("server-mappings.txt");
            download(url, &table_path, sha1, log)?;
            let named = server.with_file_name(format!(
                "{}-named.jar",
                server.file_stem().unwrap_or_default().to_string_lossy()
            ));
            let stats = lock.directory.join("server-mapped-classes.tsv");
            remap_one(
                &jdk,
                &classes,
                &deps,
                &table_path,
                server,
                &named,
                &stats,
                &lock.directory.join("server-remap.log"),
                &["-Dmf.from=target", "-Dmf.to=source"],
                log,
            )?;
            replace_with(&named, server)?;
            fs::remove_file(&named).map_err(|e| e.to_string())?;
            strip_signature(server)?;
            record.push((
                "server".to_string(),
                Value::Object(vec![
                    ("sha1".into(), Value::Str(digest(server, "sha1")?)),
                    ("sha256".into(), Value::Str(digest(server, "sha256")?)),
                    ("table_sha1".into(), Value::Str(sha1.to_string())),
                ]),
            ));
            log("Remapped the server jar");
        }
    }
    // The remapped jars are no longer byte-identical to Mojang's, so record what the
    // stage produced (and from which verified table) for the artifact authentication
    // to check against.
    write_json(
        &lock.directory.join("remapped.json"),
        &Value::Object(record),
    )?;
    log(&format!("Remapped{}", took(started)));
    Ok(())
}

/// Fetch `url` to `destination`, checked against `sha1` like a download; an existing
/// verified copy is kept (the front-end's `fetch`, the pipeline's `download`).
fn download(url: &str, destination: &Path, sha1: &str, log: Log) -> Result<PathBuf, String> {
    let name = destination
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    if destination.is_file() {
        if let Ok(actual) = digest(destination, "sha1") {
            if actual == sha1 {
                log(&format!("  cached  {name}"));
                return Ok(destination.to_path_buf());
            }
        }
        log(&format!("  sha1 mismatch, refetching {name}"));
    }
    mkdir(destination.parent().unwrap())?;
    let temporary = destination.with_file_name(format!("{name}.part"));
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(30))
        .timeout_read(Duration::from_secs(120))
        .build();
    let response = agent
        .get(url)
        .set("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(code, _) => format!("{url} answered {code}"),
            ureq::Error::Transport(t) => format!("Cannot reach {url}: {t}"),
        })?;
    {
        let mut reader = response.into_reader();
        let mut file = fs::File::create(&temporary).map_err(|e| e.to_string())?;
        std::io::copy(&mut reader, &mut file).map_err(|e| e.to_string())?;
    }
    let actual = digest(&temporary, "sha1")?;
    if actual != sha1 {
        let _ = fs::remove_file(&temporary);
        return Err(format!(
            "Checksum mismatch for {name}: expected {sha1}, got {actual}"
        ));
    }
    fs::rename(&temporary, destination).map_err(|e| e.to_string())?;
    log(&format!("  saved   {name} (sha1 ok)"));
    Ok(destination.to_path_buf())
}

/// The remapper's jars in `.local/mapping-frontend/deps/` (the front-end's
/// `ensure_deps`), each checked against its pinned SHA-1.
fn ensure_deps(home: &Path, log: Log) -> Result<Vec<PathBuf>, String> {
    let deps = home.join("deps");
    let mut paths = Vec::new();
    for (name, url, sha1) in DEPS {
        paths.push(download(url, &deps.join(name), sha1, log)?);
    }
    Ok(paths)
}

/// The compiled driver (the front-end's `ensure_driver_compiled`): the embedded
/// `Remap.java`, compiled into `.local/mapping-frontend/build/classes/`, recompiled
/// when the embedded source changes.
fn ensure_driver(
    home: &Path,
    deps: &[PathBuf],
    jdk: &Jdk,
    major: i64,
    log: Log,
) -> Result<PathBuf, String> {
    let classes = home.join("build/classes");
    let stamp = home.join("build/driver-sha256.txt");
    let wanted = sha256_hex(REMAP_JAVA.as_bytes());
    // The nested Counts class compiles beside Remap: both must be there.
    if classes.join("Remap.class").is_file()
        && classes.join("Remap$Counts.class").is_file()
        && fs::read_to_string(&stamp).is_ok_and(|current| current.trim() == wanted)
    {
        return Ok(classes);
    }
    let source = home.join("src/Remap.java");
    mkdir(source.parent().unwrap())?;
    mkdir(&classes)?;
    fs::write(&source, REMAP_JAVA).map_err(|e| e.to_string())?;
    let find = |name: &str| -> Result<&Path, String> {
        deps.iter()
            .map(PathBuf::as_path)
            .find(|p| p.file_name().is_some_and(|n| n == name))
            .ok_or_else(|| format!("The remapper is missing its {name}"))
    };
    let cp = crate::java::classpath([
        find("mapping-io-0.7.1.jar")?,
        find("tiny-remapper-0.15.0.jar")?,
    ]);
    log("Compiling the remapper driver (Remap.java)");
    let log_path = home.join("build/compile.log");
    let log_file = fs::File::create(&log_path).map_err(|e| e.to_string())?;
    let args = vec![
        "-encoding".to_string(),
        "UTF-8".to_string(),
        "-cp".to_string(),
        cp,
        "-d".to_string(),
        classes.display().to_string(),
        source.display().to_string(),
    ];
    jdk.javac(major, &args, &log_file).map_err(|e| {
        format!(
            "Compiling the remapper driver failed: {e}\n{}",
            log_tail(&log_path)
        )
    })?;
    fs::write(&stamp, format!("{wanted}\n")).map_err(|e| e.to_string())?;
    Ok(classes)
}

/// One jar through the front-end's remap driver (the pipeline's `remap_with_driver`).
/// The driver writes the class map to `stats` and a JSON coverage report beside it.
#[allow(clippy::too_many_arguments)]
fn remap_one(
    jdk: &Jdk,
    classes: &Path,
    deps: &[PathBuf],
    table: &Path,
    input: &Path,
    output: &Path,
    stats: &Path,
    log_path: &Path,
    properties: &[&str],
    log: Log,
) -> Result<(), String> {
    let mut paths = vec![classes.to_path_buf()];
    paths.extend(deps.iter().cloned());
    let cp = crate::java::classpath(paths.iter().map(PathBuf::as_path).collect::<Vec<_>>());
    let mut args: Vec<String> = properties.iter().map(|p| p.to_string()).collect();
    args.extend([
        "-cp".to_string(),
        cp,
        "Remap".to_string(),
        table.display().to_string(),
        input.display().to_string(),
        output.display().to_string(),
        stats.display().to_string(),
    ]);
    log(&format!(
        "Remapping {} with {}",
        input.file_name().unwrap_or_default().to_string_lossy(),
        table.file_name().unwrap_or_default().to_string_lossy()
    ));
    mkdir(log_path.parent().unwrap())?;
    let file = fs::File::create(log_path).map_err(|e| e.to_string())?;
    jdk.java(&args, None, &file).map_err(|e| {
        format!(
            "The remapper failed on {}: {e}\n{}",
            input.file_name().unwrap_or_default().to_string_lossy(),
            log_tail(log_path)
        )
    })
}

/// The last 2000 characters of a JVM log, for a failure's diagnostic.
fn log_tail(path: &Path) -> String {
    let text = fs::read_to_string(path).unwrap_or_default();
    let tail: String = text.chars().rev().take(2000).collect();
    tail.chars().rev().collect()
}

/// Move `source` over `destination` (the pipeline's `replace_with`): through a
/// temporary file in the destination's folder, so a failure leaves the destination's
/// old copy in place. The source is left alone.
fn replace_with(source: &Path, destination: &Path) -> Result<(), String> {
    let temporary = destination.with_file_name(format!(
        ".{}.tmp-{}",
        destination
            .file_name()
            .unwrap_or_default()
            .to_string_lossy(),
        std::process::id()
    ));
    fs::copy(source, &temporary).map_err(|e| e.to_string())?;
    if let Err(e) = fs::rename(&temporary, destination) {
        let _ = fs::remove_file(&temporary);
        return Err(e.to_string());
    }
    Ok(())
}

/// Whether a jar member is one of Mojang's signature files.
fn is_signature_entry(name: &str) -> bool {
    name.starts_with("META-INF/")
        && name.rsplit('.').next().is_some_and(|ext| {
            ext.eq_ignore_ascii_case("SF")
                || ext.eq_ignore_ascii_case("RSA")
                || ext.eq_ignore_ascii_case("DSA")
        })
}

/// Drop Mojang's signature from a remapped jar (the pipeline's `strip_signature`).
/// Remapping rewrites the class bytes, so the signed manifest's per-entry digests no
/// longer match them; the JVM verifies the entries that keep a signed digest as their
/// classes load and refuses the mismatches, which stops any exporter that runs the
/// game from loading classes such as the JFR event classes. A remapped jar is a
/// derived artifact whose authenticity here is `remapped.json`'s recorded digests, so
/// the signature block goes: no .SF/.RSA/.DSA files, and the manifest keeps only its
/// main attributes.
fn strip_signature(jar: &Path) -> Result<(), String> {
    let temporary = jar.with_file_name(format!(
        "{}.unsigned",
        jar.file_name().unwrap_or_default().to_string_lossy()
    ));
    let result = (|| {
        let mut source = crate::open_zip(jar)?;
        let file = fs::File::create(&temporary).map_err(|e| e.to_string())?;
        let mut out = zip::ZipWriter::new(file);
        for index in 0..source.len() {
            let mut entry = source.by_index(index).map_err(|e| e.to_string())?;
            let name = entry.name().map_err(|e| e.to_string())?.to_string();
            if is_signature_entry(&name) {
                continue;
            }
            let mut data = Vec::with_capacity(entry.size() as usize);
            std::io::Read::read_to_end(&mut entry, &mut data).map_err(|e| e.to_string())?;
            if name == "META-INF/MANIFEST.MF" {
                let text = String::from_utf8_lossy(&data).replace("\r\n", "\n");
                let main = text.split("\n\n").next().unwrap_or("");
                data = format!("{main}\n").into_bytes();
            }
            let options =
                zip::write::SimpleFileOptions::default().compression_method(entry.compression());
            let options = match entry.last_modified() {
                Some(time) => options.last_modified_time(time),
                None => options,
            };
            if entry.is_dir() {
                out.add_directory(name, options)
                    .map_err(|e| e.to_string())?;
            } else {
                out.start_file(name, options).map_err(|e| e.to_string())?;
                std::io::Write::write_all(&mut out, &data).map_err(|e| e.to_string())?;
            }
        }
        out.finish().map_err(|e| e.to_string())?;
        Ok(())
    })();
    if let Err(e) = result {
        let _ = fs::remove_file(&temporary);
        return Err(e);
    }
    fs::rename(&temporary, jar).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("horizon-remap-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn unobfuscated_manifests_need_no_remapping() {
        // No mapping tables in the manifest: the stage returns at once, having touched
        // nothing and needed no Java (the java home named here does not exist).
        let root = dir("noop");
        let lock = Lock {
            json: Value::Object(Vec::new()),
            version: "26.3".to_string(),
            java_major: 25,
            directory: root.join(".local/minecraft/26.3"),
        };
        let manifest = Value::Object(vec![
            ("id".into(), Value::str("26.3")),
            (
                "downloads".into(),
                Value::Object(vec![
                    ("client".into(), Value::Object(vec![])),
                    ("server".into(), Value::Object(vec![])),
                ]),
            ),
        ]);
        remap_if_needed(
            &root,
            &lock,
            &manifest,
            Path::new("client.jar"),
            Path::new("server.jar"),
            false,
            Path::new("no-java-here"),
            &|_| {},
        )
        .unwrap();
        assert!(!root.join(".local/mapping-frontend").exists());
        assert!(!lock.directory.join("remapped.json").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn the_embedded_driver_is_the_front_ends_driver() {
        // A mangled embed (a bad raw-string delimiter truncating it, a partial copy)
        // fails loudly here instead of at a tester's prepare.
        assert!(REMAP_JAVA.contains("public final class Remap"));
        assert!(
            REMAP_JAVA
                .contains("usage: Remap <mappings-file> <input.jar> <output.jar> <stats-file>")
        );
        assert!(REMAP_JAVA.contains("TinyRemapper.newRemapper()"));
        assert!(REMAP_JAVA.contains("mf.from"));
        assert!(REMAP_JAVA.contains("mf.to"));
    }

    #[test]
    fn stripping_a_signature_keeps_classes_and_the_manifest_head() {
        let root = dir("strip");
        let jar = root.join("game.jar");
        let manifest = "Manifest-Version: 1.0\r\nMain-Class: net.minecraft.server.Main\r\n\r\nName: a/b.class\r\nSHA-384-Digest: deadbeef\r\n";
        {
            let file = fs::File::create(&jar).unwrap();
            let mut out = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default();
            out.start_file("META-INF/MANIFEST.MF", options).unwrap();
            out.write_all(manifest.as_bytes()).unwrap();
            out.start_file("META-INF/MOJANGCS.SF", options).unwrap();
            out.write_all(b"Signature-Version: 1.0\n").unwrap();
            out.start_file("META-INF/MOJANGCS.RSA", options).unwrap();
            out.write_all(b"rsa-block").unwrap();
            out.start_file("a/b.class", options).unwrap();
            out.write_all(b"\xca\xfe\xba\xbe class").unwrap();
            out.start_file("data/pack.mcmeta", options).unwrap();
            out.write_all(b"{}").unwrap();
            out.finish().unwrap();
        }
        strip_signature(&jar).unwrap();
        let mut archive = crate::open_zip(&jar).unwrap();
        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().unwrap().to_string())
            .collect();
        assert_eq!(
            names,
            ["META-INF/MANIFEST.MF", "a/b.class", "data/pack.mcmeta"]
        );
        assert_eq!(
            crate::zip_text(&mut archive, "META-INF/MANIFEST.MF").unwrap(),
            "Manifest-Version: 1.0\nMain-Class: net.minecraft.server.Main\n"
        );
        assert_eq!(
            crate::zip_bytes(&mut archive, "a/b.class").unwrap(),
            b"\xca\xfe\xba\xbe class"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
