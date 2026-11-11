import java.io.BufferedReader;
import java.io.BufferedWriter;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;

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
 * ProGuard txt today, SRG/TSRG/tiny later), works out which namespace is the
 * obfuscated one and which is the named one, then applies the table in the
 * obfuscated -> named direction with tiny-remapper.
 *
 * Usage: Remap <mappings-file> <input.jar> <output.jar> <stats-file>
 *
 * The stats file gets one line per mapped class: "<obfuscated>\t<named>".
 * Nothing in this tool is Minecraft-specific except the probe class used to
 * detect the named namespace (see below); the direction can also be forced
 * with -Dmf.from=<ns> / -Dmf.to=<ns> namespace-name system properties.
 */
public final class Remap {
    /** A class that only exists under its real name in the named namespace. */
    private static final String PROBE_CLASS = "net/minecraft/client/Minecraft";

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

        String srcNs = tree.getSrcNamespace();
        List<String> dstNs = tree.getDstNamespaces();
        if (dstNs.size() != 1) {
            throw new IllegalStateException("expected exactly one destination namespace, got " + dstNs);
        }
        String otherNs = dstNs.get(0);
        System.out.println("namespaces: src=" + srcNs + " dst=" + otherNs);
        System.out.println("classes in mapping table: " + tree.getClasses().size());

        // Direction detection: the named namespace is the one in which the
        // probe class carries its real name. Mojang's official tables are
        // ProGuard format (named on the left), so this cannot be assumed --
        // detect it from the data instead.
        String fromNs;
        String toNs;
        String forcedFrom = System.getProperty("mf.from");
        String forcedTo = System.getProperty("mf.to");
        if (forcedFrom != null && forcedTo != null) {
            fromNs = forcedFrom;
            toNs = forcedTo;
        } else if (tree.getClass(PROBE_CLASS, tree.getNamespaceId(otherNs)) != null
                && tree.getClass(PROBE_CLASS, tree.getNamespaceId(srcNs)) == null) {
            toNs = otherNs;
            fromNs = srcNs;
        } else if (tree.getClass(PROBE_CLASS, tree.getNamespaceId(srcNs)) != null
                && tree.getClass(PROBE_CLASS, tree.getNamespaceId(otherNs)) == null) {
            toNs = srcNs;
            fromNs = otherNs;
        } else {
            throw new IllegalStateException(
                    "cannot detect direction: probe class " + PROBE_CLASS + " not found in exactly one namespace"
                    + " (src has it: " + (tree.getClass(PROBE_CLASS, tree.getNamespaceId(srcNs)) != null)
                    + ", dst has it: " + (tree.getClass(PROBE_CLASS, tree.getNamespaceId(otherNs)) != null)
                    + "); set -Dmf.from/-Dmf.to to force a direction");
        }
        System.out.println("direction: " + fromNs + " (obfuscated) -> " + toNs + " (named)");

        final String from = fromNs;
        final String to = toNs;

        // Dump the class table for the caller's statistics, and count how
        // many classes actually change name.
        int renamed = 0;
        try (BufferedWriter w = Files.newBufferedWriter(statsFile)) {
            for (MappingTree.ClassMapping cls : tree.getClasses()) {
                String f = cls.getName(from);
                String t = cls.getName(to);
                if (f == null || t == null) continue;
                w.write(f + "\t" + t + "\n");
                if (!f.equals(t)) renamed++;
            }
        }
        System.out.println("classes renamed: " + renamed);

        IMappingProvider provider = acceptor -> {
            for (MappingTree.ClassMapping cls : tree.getClasses()) {
                String ownerFrom = cls.getName(from);
                String ownerTo = cls.getName(to);
                if (ownerFrom == null || ownerTo == null) continue;
                acceptor.acceptClass(ownerFrom, ownerTo);
                for (MappingTree.FieldMapping field : cls.getFields()) {
                    String name = field.getName(to);
                    if (name == null) continue;
                    acceptor.acceptField(
                            new IMappingProvider.Member(ownerFrom, field.getName(from), field.getDesc(from)),
                            name);
                }
                for (MappingTree.MethodMapping method : cls.getMethods()) {
                    String name = method.getName(to);
                    if (name == null) continue;
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
        System.out.println("wrote " + outputJar);
    }
}
