package dev.horizonlab.parity;

import java.io.PrintWriter;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Set;

/**
 * 1.21.4 has no `Item.canDestroyBlock` (a 1.21.5 `Tool` field), but
 * `Item.canAttackBlock` still gates the destroy, and MaceItem, SwordItem and
 * TridentItem all override it to `!isCreative()` in 1.21.4 (javap-verified
 * against the 1.21.4 server jar): a creative player destroys no block with
 * a sword, trident or mace in hand, and destroys with anything else. The
 * tool mode never matters in 1.21.4. (Item 119 removed the 1.21.5-form gate
 * and read this as "creative always destroys" — true for every other item.)
 */
public final class VanillaCreativeMiningOracle {
    private static final Set<String> WEAPONS = Set.of(
        "minecraft:mace", "minecraft:trident",
        "minecraft:wooden_sword", "minecraft:stone_sword",
        "minecraft:iron_sword", "minecraft:golden_sword",
        "minecraft:diamond_sword", "minecraft:netherite_sword");

    public static void main(String[] args) throws Exception {
        try (var out = new PrintWriter(Files.newBufferedWriter(Path.of(args[1])));
             var lines = Files.lines(Path.of(args[0]))) {
            for (var line : (Iterable<String>) lines::iterator()) {
                // Rows are compact JSON: {"item": "...", "creative": true, ...}.
                int itemAt = line.indexOf("\"item\": \"");
                String item = itemAt < 0 ? "" : line.substring(itemAt + 9, line.indexOf('"', itemAt + 9));
                boolean creative = line.contains("\"creative\": true");
                out.println(!(creative && WEAPONS.contains(item)));
            }
        }
    }
}
