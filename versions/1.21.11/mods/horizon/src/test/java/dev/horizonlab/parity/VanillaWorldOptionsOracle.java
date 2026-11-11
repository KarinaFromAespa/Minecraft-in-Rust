package dev.horizonlab.parity;

import io.netty.buffer.Unpooled;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.HexFormat;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.protocol.game.ServerboundChangeDifficultyPacket;
import net.minecraft.network.protocol.game.ServerboundLockDifficultyPacket;
import net.minecraft.world.Difficulty;

/** Encode the same settings through the pinned game's own StreamCodecs. */
public final class VanillaWorldOptionsOracle {
    public static void main(String[] args) throws Exception {
        var output = new ArrayList<String>();
        for (String line : Files.readAllLines(Path.of(args[0]))) {
            String[] fields = line.split("\t");
            var buffer = new FriendlyByteBuf(Unpooled.buffer());
            switch (fields[0]) {
                case "difficulty" -> ServerboundChangeDifficultyPacket.STREAM_CODEC.encode(buffer,
                    new ServerboundChangeDifficultyPacket(Difficulty.byId(Integer.parseInt(fields[1]))));
                case "lock" -> ServerboundLockDifficultyPacket.STREAM_CODEC.encode(buffer,
                    new ServerboundLockDifficultyPacket(Boolean.parseBoolean(fields[1])));
                // 1.21.11 has no game-rule packets: `set` and `values` are not cases here.
                default -> throw new IllegalArgumentException(line);
            }
            byte[] bytes = new byte[buffer.readableBytes()];
            buffer.readBytes(bytes);
            output.add(HexFormat.of().formatHex(bytes));
            buffer.release();
        }
        Files.write(Path.of(args[1]), output);
    }
}
