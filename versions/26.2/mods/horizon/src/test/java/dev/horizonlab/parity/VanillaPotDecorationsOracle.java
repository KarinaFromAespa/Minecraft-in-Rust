package dev.horizonlab.parity;

import com.google.gson.*;
import com.mojang.serialization.DataResult;
import java.io.*;
import java.nio.file.*;
import java.util.*;
import net.minecraft.core.component.DataComponents;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.nbt.*;
import net.minecraft.world.item.Items;
import net.minecraft.world.level.block.entity.PotDecorations;

/**
 * The game path for pot_decorations, through vanilla's codec: NBT list ->
 * PotDecorations -> NBT list -> PotDecorations. Each output line is
 * `label\t<sides after decode>\t<re-encoded component>\t<sides after the
 * second decode>`, matching the Rust probe's form. The first line pins the
 * decorated pot's default component, which the harness's EMPTY rule rests on.
 */
public final class VanillaPotDecorationsOracle {
    static String side(Optional<net.minecraft.world.item.Item> item) {
        return item.map(i -> BuiltInRegistries.ITEM.getKey(i).toString()).orElse("-");
    }

    static String sides(PotDecorations d) {
        return side(d.back()) + "," + side(d.left()) + "," + side(d.right()) + "," + side(d.front());
    }

    static String encoded(Tag tag) {
        if (!(tag instanceof ListTag list)) return "not-a-list";
        var ids = new StringJoiner(",");
        for (var e : list) ids.add("\"" + ((StringTag) e).value() + "\"");
        return "[" + ids + "]";
    }

    public static void main(String[] args) throws Exception {
        net.minecraft.SharedConstants.tryDetectVersion();
        net.minecraft.server.Bootstrap.bootStrap();
        // Binding the holders' components: without the data-component
        // initializers, `Item.components()` throws "Components not bound yet".
        // (26.2 names the lookup factory `createLookup`; 26.3 renamed it
        // `createWorldLookup`.)
        var lookup = net.minecraft.data.registries.VanillaRegistries.createLookup();
        for (var p : BuiltInRegistries.DATA_COMPONENT_INITIALIZERS.build(lookup)) p.apply();
        Path dir = Path.of(args[1]);
        var def = Items.DECORATED_POT.components().get(DataComponents.POT_DECORATIONS);
        String defaultLine = "default " + (def == null ? "none" : def.equals(PotDecorations.EMPTY) ? "EMPTY" : "other");
        try (var out = new PrintWriter(Files.newBufferedWriter(dir.resolve(args[0] + ".txt")));
             var lines = Files.lines(dir.resolve("inputs.txt"))) {
            out.println(defaultLine);
            for (var line : (Iterable<String>) lines::iterator) {
                int tab = line.indexOf('\t');
                String label = line.substring(0, tab);
                JsonArray ids = JsonParser.parseString(line.substring(tab + 1)).getAsJsonArray();
                var input = new ListTag();
                for (var id : ids) input.add(StringTag.valueOf(id.getAsString()));
                DataResult<PotDecorations> parsed = PotDecorations.CODEC.parse(NbtOps.INSTANCE, input);
                if (parsed.isError()) {
                    out.println(label + "\terror");
                    continue;
                }
                PotDecorations first = parsed.getOrThrow();
                Tag saved = PotDecorations.CODEC.encodeStart(NbtOps.INSTANCE, first).getOrThrow();
                PotDecorations second = PotDecorations.CODEC.parse(NbtOps.INSTANCE, saved).getOrThrow();
                out.println(label + "\t" + sides(first) + "\t" + encoded(saved) + "\t" + sides(second));
            }
        }
    }
}
