package dev.horizonlab.parity;

import com.google.gson.JsonElement;
import com.mojang.serialization.Dynamic;
import com.mojang.serialization.JsonOps;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.NbtOps;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.Property;

/**
 * The flat {id, properties} block-state form the parity fixtures consume.
 * Later releases expose it as BlockState.FULL_CODEC; 1.21.11 has only the
 * palette codec ({Name, Properties}), so the same data is assembled here
 * from the registry and the state's property map.
 */
final class VanillaStateCodec {
    private VanillaStateCodec() {}

    static CompoundTag encodeFull(BlockState state) {
        var tag = new CompoundTag();
        tag.putString("id", BuiltInRegistries.BLOCK.getKey(state.getBlock()).toString());
        var properties = new CompoundTag();
        for (var entry : state.getValues().entrySet()) {
            properties.putString(entry.getKey().getName(), valueName(entry.getKey(), entry.getValue()));
        }
        if (!properties.isEmpty()) {
            tag.put("properties", properties);
        }
        return tag;
    }

    static JsonElement encodeFullJson(BlockState state) {
        return new Dynamic<>(NbtOps.INSTANCE, encodeFull(state)).convert(JsonOps.INSTANCE).getValue();
    }

    static BlockState decodeFull(JsonElement json) {
        var object = json.getAsJsonObject();
        var block = BuiltInRegistries.BLOCK.getValue(
            ResourceLocation.parse(object.get("id").getAsString()));
        var state = block.defaultBlockState();
        if (object.has("properties")) {
            for (var entry : object.getAsJsonObject("properties").entrySet()) {
                state = apply(state, entry.getKey(), entry.getValue().getAsString());
            }
        }
        return state;
    }

    @SuppressWarnings({"unchecked", "rawtypes"})
    private static String valueName(Property property, Comparable value) {
        return property.getName(value);
    }

    @SuppressWarnings({"unchecked", "rawtypes"})
    private static BlockState apply(BlockState state, String name, String value) {
        Property property = state.getBlock().getStateDefinition().getProperty(name);
        if (property == null) {
            throw new IllegalArgumentException("unknown property " + name);
        }
        return state.setValue(property, (Comparable) property.getValue(value).orElseThrow());
    }
}
