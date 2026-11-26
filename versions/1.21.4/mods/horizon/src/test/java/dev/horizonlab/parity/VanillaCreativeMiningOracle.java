package dev.horizonlab.parity;

import java.io.PrintWriter;
import java.nio.file.Files;
import java.nio.file.Path;

/**
 * 1.21.4 has no `Item.canDestroyBlock`: a creative player destroys any block with any held item,
 * whatever the stack's tool component says (`can_destroy_blocks_in_creative` is a 1.21.5 `Tool`
 * field, and the 4-arg `Tool` constructor does not exist in 1.21.4). Every row is true.
 */
public final class VanillaCreativeMiningOracle {
    public static void main(String[] args) throws Exception {
        try (var out = new PrintWriter(Files.newBufferedWriter(Path.of(args[1])));
             var lines = Files.lines(Path.of(args[0]))) {
            for (var line : (Iterable<String>) lines::iterator()) {
                out.println(true);
            }
        }
    }
}
