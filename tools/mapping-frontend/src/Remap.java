import java.io.BufferedReader;
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
