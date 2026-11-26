package dev.horizonlab.parity;

import com.google.gson.Gson;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.io.PrintWriter;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.NbtOps;
import net.minecraft.nbt.Tag;
import net.minecraft.nbt.TagParser;
import net.minecraft.server.level.TicketType;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.TicketStorage;

/**
 * Original oracle: one level's saved chunk tickets through the shipped TicketStorage.
 * Each input line holds a saved {@code chunk_tickets} data compound (SNBT) and operations:
 * {@code tick} (purgeStaleTickets, which in 1.21.5 counts every ticket down), {@code portal}
 * (Entity.placePortalTicket's radius-3 PORTAL ticket) and {@code restart}
 * (deactivateTicketsOnClosing, the codec's save and load, activateAllDeactivatedTickets).
 * After activation and each operation it prints what the storage would save, and which of the
 * case's chunks a PORTAL ticket holds now ({@code getTickets}).
 * A tick op may repeat ({@code times}).
 */
public final class VanillaPortalTicketsOracle {
    private static ChunkPos chunk(JsonElement pos) {
        return new ChunkPos(pos.getAsJsonArray().get(0).getAsInt(), pos.getAsJsonArray().get(1).getAsInt());
    }

    private static TicketStorage load(Tag data) {
        // SavedDataStorage.readSavedData.
        return TicketStorage.CODEC.parse(NbtOps.INSTANCE, data).resultOrPartial(error -> {}).orElseThrow();
    }

    /** Each saved ticket as one line: type, level, chunk and ticks_left, "-" where the codec leaves it
     * out; then "held x z" for each of {@code chunks} a PORTAL ticket holds. */
    private static List<String> snapshot(TicketStorage storage, JsonArray chunks) {
        CompoundTag data = (CompoundTag) TicketStorage.CODEC.encodeStart(NbtOps.INSTANCE, storage).getOrThrow();
        List<String> rows = new ArrayList<>();
        for (Tag tag : data.getListOrEmpty("tickets")) {
            CompoundTag ticket = (CompoundTag) tag;
            int[] pos = ticket.getIntArray("chunk_pos").orElseThrow();
            rows.add(ticket.getStringOr("type", "?") + " " + ticket.getIntOr("level", -1) + " " + pos[0] + " " + pos[1]
                + " " + (ticket.contains("ticks_left") ? String.valueOf(ticket.getLongOr("ticks_left", 0)) : "-"));
        }
        for (JsonElement c : chunks) {
            ChunkPos pos = chunk(c);
            if (storage.getTickets(pos.pack()).stream().anyMatch(t -> t.getType() == TicketType.PORTAL)) {
                rows.add("held " + pos.x() + " " + pos.z());
            }
        }
        Collections.sort(rows);
        return rows;
    }

    public static void main(String[] args) throws Exception {
        net.minecraft.SharedConstants.tryDetectVersion();
        net.minecraft.server.Bootstrap.bootStrap();
        Gson gson = new Gson();
        try (var out = new PrintWriter(Files.newBufferedWriter(Path.of(args[1]), StandardCharsets.UTF_8))) {
            for (String line : Files.readAllLines(Path.of(args[0]), StandardCharsets.UTF_8)) {
                JsonObject input = JsonParser.parseString(line).getAsJsonObject();
                TicketStorage storage = load(TagParser.parseCompoundFully(input.get("saved").getAsString()));
                storage.activateAllDeactivatedTickets();
                JsonArray chunks = input.getAsJsonArray("chunks");
                List<List<String>> trace = new ArrayList<>();
                trace.add(snapshot(storage, chunks));
                for (JsonElement element : input.getAsJsonArray("ops")) {
                    JsonObject op = element.getAsJsonObject();
                    switch (op.get("op").getAsString()) {
                        case "tick" -> {
                            int times = op.has("times") ? op.get("times").getAsInt() : 1;
                            for (int i = 0; i < times; i++) {
                                storage.purgeStaleTickets();
                            }
                        }
                        case "portal" -> storage.addTicketWithRadius(TicketType.PORTAL, chunk(op.get("pos")), 3);
                        case "restart" -> {
                            storage.deactivateTicketsOnClosing();
                            storage = load(TicketStorage.CODEC.encodeStart(NbtOps.INSTANCE, storage).getOrThrow());
                            storage.activateAllDeactivatedTickets();
                        }
                        default -> throw new IllegalArgumentException(op.toString());
                    }
                    trace.add(snapshot(storage, chunks));
                }
                out.println(gson.toJson(trace));
            }
        }
    }
}
