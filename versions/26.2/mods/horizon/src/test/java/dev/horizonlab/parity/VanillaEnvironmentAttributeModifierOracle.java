package dev.horizonlab.parity;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.mojang.serialization.JsonOps;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import net.minecraft.SharedConstants;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.resources.Identifier;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.attribute.EnvironmentAttribute;
import net.minecraft.world.attribute.EnvironmentAttributeMap;
import net.minecraft.util.ARGB;

/**
 * Parses each case's entry with the pinned game's EnvironmentAttributeMap codec and applies it to
 * the subject value: one line per case with the applied value and the sanitized one, or "error".
 * 26.2's colors are packed ints; they print as the float bits of their channels, as the probe does.
 */
public final class VanillaEnvironmentAttributeModifierOracle {
    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        var output = new ArrayList<String>();
        for (String line : Files.readAllLines(Path.of(args[0]))) {
            if (line.equals("registry")) {
                registry(output);
                continue;
            }
            String[] fields = line.split("\t");
            output.add(run(fields[0], fields[1], fields[2]));
        }
        Files.write(Path.of(args[1]), output);
    }

    @SuppressWarnings("unchecked")
    private static String run(String id, String subjectJson, String entryJson) {
        var attribute = (EnvironmentAttribute<Object>) BuiltInRegistries.ENVIRONMENT_ATTRIBUTE.getValue(Identifier.parse(id));
        Object subject = attribute.valueCodec().parse(JsonOps.INSTANCE, JsonParser.parseString(subjectJson)).getOrThrow();
        var map = new JsonObject();
        map.add(id, JsonParser.parseString(entryJson));
        var parsed = EnvironmentAttributeMap.CODEC.parse(JsonOps.INSTANCE, map);
        if (parsed.isError()) {
            return "error";
        }
        Object value = parsed.getOrThrow().applyModifier(attribute, subject);
        return format(value, attribute) + "\t" + format(attribute.sanitizeValue(value), attribute);
    }

    private static final java.util.Set<String> PORTED = java.util.Set.of(
        "minecraft:boolean", "minecraft:float", "minecraft:angle_degrees", "minecraft:rgb_color", "minecraft:argb_color");

    /** Every attribute in registration order: id, type, default, and how a float range clamps. */
    @SuppressWarnings("unchecked")
    private static void registry(java.util.List<String> output) {
        for (var raw : BuiltInRegistries.ENVIRONMENT_ATTRIBUTE) {
            var attribute = (EnvironmentAttribute<Object>) raw;
            String type = BuiltInRegistries.ATTRIBUTE_TYPE.getKey(attribute.type()).toString();
            boolean ported = PORTED.contains(type);
            String range = attribute.defaultValue() instanceof Float
                ? format(attribute.sanitizeValue(Float.NEGATIVE_INFINITY), attribute) + " " + format(attribute.sanitizeValue(Float.POSITIVE_INFINITY), attribute)
                : "-";
            output.add(BuiltInRegistries.ENVIRONMENT_ATTRIBUTE.getKey(raw) + "\t" + (ported ? type : "other") + "\t"
                + (ported ? format(attribute.defaultValue(), attribute) : "unported") + "\t" + range);
        }
    }

    private static String bits(float f) {
        return String.format("%08x", Float.floatToRawIntBits(f));
    }

    private static String format(Object value, EnvironmentAttribute<?> attribute) {
        if (value instanceof Float f) {
            return bits(f);
        }
        if (value instanceof Integer color) {
            // A packed int, printed as the float bits of its channels (RGB: 3, ARGB: 4).
            String rgb = bits(ARGB.redFloat(color)) + " " + bits(ARGB.greenFloat(color)) + " " + bits(ARGB.blueFloat(color));
            String type = BuiltInRegistries.ATTRIBUTE_TYPE.getKey(attribute.type()).toString();
            return type.equals("minecraft:argb_color") ? rgb + " " + bits(ARGB.alphaFloat(color)) : rgb;
        }
        return String.valueOf(value);
    }
}
