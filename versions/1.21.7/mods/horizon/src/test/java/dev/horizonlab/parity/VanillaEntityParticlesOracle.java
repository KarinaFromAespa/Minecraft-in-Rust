package dev.horizonlab.parity;

import com.google.gson.*;
import java.lang.reflect.*;
import java.nio.file.*;
import java.util.*;
import net.minecraft.SharedConstants;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.core.particles.ParticleOptions;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.server.Bootstrap;
import net.minecraft.util.RandomSource;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.EntityDimensions;
import net.minecraft.world.entity.animal.allay.Allay;
import net.minecraft.world.entity.animal.camel.Camel;
import net.minecraft.world.entity.animal.Cow;
import net.minecraft.world.entity.animal.Dolphin;
import net.minecraft.world.entity.animal.horse.Horse;
import net.minecraft.world.entity.animal.horse.Llama;
import net.minecraft.world.entity.animal.Cat;
import net.minecraft.world.entity.animal.Ocelot;
import net.minecraft.world.entity.animal.Parrot;
import net.minecraft.world.entity.animal.wolf.Wolf;
import net.minecraft.world.entity.npc.Villager;
import net.minecraft.world.phys.Vec3;

/** Calls the real client handleEntityEvent of entities whose events are particle
 * bursts about their body, and records what they ask the level to spawn. The
 * entities are allocated without constructors and given only a level stand-in,
 * a position, dimensions and their random, which is all those handlers read. */
public final class VanillaEntityParticlesOracle {
    static String d(double v) { return String.format("%016x", Double.doubleToRawLongBits(v)); }
    static Field field(Class<?> type, String name) throws Exception {
        var f = type.getDeclaredField(name); f.setAccessible(true); return f;
    }
    static final class Level0 extends ClientLevel {
        List<String> events;
        Level0() { super(null, null, null, null, 0, 0, null, false, 0L, 0); }
        @Override public void addParticle(ParticleOptions o, double x, double y, double z, double xd, double yd, double zd) {
            events.add("+ " + BuiltInRegistries.PARTICLE_TYPE.getKey(o.getType()).getPath() + " " + d(x) + " " + d(y) + " " + d(z)
                + " " + d(xd) + " " + d(yd) + " " + d(zd));
        }
    }
    static final Map<String, Class<? extends Entity>> TYPES = Map.ofEntries(
        Map.entry("villager", Villager.class), Map.entry("cow", Cow.class), Map.entry("wolf", Wolf.class),
        Map.entry("cat", Cat.class), Map.entry("parrot", Parrot.class), Map.entry("horse", Horse.class),
        Map.entry("llama", Llama.class), Map.entry("camel", Camel.class), Map.entry("ocelot", Ocelot.class),
        Map.entry("dolphin", Dolphin.class), Map.entry("allay", Allay.class));
    // No nautilus entry: 1.21.7 has no nautilus entity (no such class in the
    // jar, no minecraft:nautilus entity type in the registry).

    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion(); Bootstrap.bootStrap();
        var u = Class.forName("sun.misc.Unsafe"); var unsafe = field(u, "theUnsafe").get(null);
        var allocate = u.getMethod("allocateInstance", Class.class);
        var level = (Level0) allocate.invoke(unsafe, Level0.class); level.events = new ArrayList<>();
        Field levelField = field(Entity.class, "level"), position = field(Entity.class, "position"),
            dimensions = field(Entity.class, "dimensions"), random = field(Entity.class, "random");
        try (var output = Files.newBufferedWriter(Path.of(args[1]))) {
            int n = 0;
            for (String line : Files.readAllLines(Path.of(args[0]))) {
                var c = JsonParser.parseString(line).getAsJsonObject();
                output.write("== " + n++); output.newLine();
                var entity = (Entity) allocate.invoke(unsafe, TYPES.get(c.get("entity").getAsString()));
                var feet = c.getAsJsonArray("feet");
                levelField.set(entity, level);
                position.set(entity, new Vec3(feet.get(0).getAsDouble(), feet.get(1).getAsDouble(), feet.get(2).getAsDouble()));
                dimensions.set(entity, EntityDimensions.scalable(c.get("width").getAsFloat(), c.get("height").getAsFloat()));
                var r = RandomSource.create(c.get("seed").getAsLong());
                random.set(entity, r);
                level.events.clear();
                entity.handleEntityEvent((byte) c.get("event").getAsInt());
                for (var e : level.events) { output.write(e); output.newLine(); }
                output.write("next " + r.nextLong()); output.newLine();
            }
        }
    }
}
