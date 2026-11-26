package dev.horizonlab.parity;

import com.google.gson.*;
import java.io.*;
import java.nio.file.*;
import java.util.*;
import net.minecraft.client.renderer.block.model.BlockElement;
import net.minecraft.client.renderer.block.model.BlockElementFace;
import net.minecraft.client.renderer.block.model.BlockElementRotation;
import net.minecraft.client.renderer.block.model.BlockModel;
import net.minecraft.client.renderer.block.model.ItemTransform;
import net.minecraft.world.item.ItemDisplayContext;
import org.joml.Vector3fc;

/** Original adapter around the pinned client parser. No model assets are embedded. */
public final class VanillaCuboidModelOracle {
    private static final Gson JSON = new GsonBuilder().serializeNulls().create();
    private static long bits(float value) { return Integer.toUnsignedLong(Float.floatToIntBits(value)); }
    private static Object vector(Vector3fc value) { return new long[]{bits(value.x()), bits(value.y()), bits(value.z())}; }
    private static String direction(net.minecraft.core.Direction value) { return value == null ? null : value.getName(); }
    static Map<String, Object> map(Object... values) {
        var result = new TreeMap<String, Object>();
        for (int i=0;i<values.length;i+=2) result.put((String)values[i], values[i+1]);
        return result;
    }
    static Object transform(ItemTransform value) {
        if (value == ItemTransform.NO_TRANSFORM) return "inherit";
        if (value == null) return null;
        // 1.21.4 ItemTransform carries public fields, not record accessors.
        return map("rotation", vector(value.rotation), "translation", vector(value.translation), "scale", vector(value.scale));
    }
    private static Object rotation(BlockElementRotation value) {
        if (value == null) return null;
        // 1.21.10 rotations are single-axis records (origin, axis, angle,
        // rescale); the sealed SingleAxisRotation/EulerXYZRotation split
        // came later. Emit the same single-axis shape.
        Object angles = map("axis", value.axis().getName(), "angle", bits(value.angle()));
        return map("origin", vector(value.origin()), "value", angles, "rescale", value.rescale());
    }
    private static Object face(BlockElementFace value) {
        if (value == null) return null;
        var uv = value.uv();
        // 1.21.4 has no per-face rotation field: the rotation lives on the
        // BlockFaceUV as an int in degrees (0/90/180/270), and the uv box
        // is the public float[] uvs (minU, minV, maxU, maxV).
        return map("cull", direction(value.cullForDirection()), "tint", value.tintIndex(), "texture", value.texture(),
            "rotation", uv == null ? 0 : uv.rotation, "uv", uv == null ? null : new long[]{bits(uv.uvs[0]), bits(uv.uvs[1]), bits(uv.uvs[2]), bits(uv.uvs[3])});
    }
    static Object element(BlockElement value) {
        if (value == null) return null;
        var faces = new TreeMap<String, Object>();
        // 1.21.4 BlockElement carries public fields, not record accessors.
        value.faces.forEach((key, f) -> faces.put(key.getName(), face(f)));
        // 1.21.11 elements carry a boolean shade flag; the per-face shade direction override
        // the later cuboid format reports does not exist yet.
        return map("from", vector(value.from), "to", vector(value.to), "faces", faces,
            "rotation", rotation(value.rotation), "shade", value.shade, "light", value.lightEmission);
    }
    @SuppressWarnings("unchecked")
    private static List<BlockElement> elements(BlockModel value) throws Exception {
        // 1.21.3 BlockModel.getElements() is public.
        return value.getElements();
    }
    private static Object parent(BlockModel value) throws Exception {
        // 1.21.3 BlockModel has no getParentLocation(): the parent location
        // is the protected parentLocation field.
        var field = BlockModel.class.getDeclaredField("parentLocation"); field.setAccessible(true);
        var location = field.get(value);
        return location == null ? null : location.toString();
    }
    private static Object model(BlockModel value) throws Exception {
        if (value == null) return null;
        var textures = new TreeMap<String, Object>();
        // 1.21.3 has no texture slots: the protected textureMap holds
        // Either<Material, String> per key — a sprite, or a "#name" reference.
        var field = BlockModel.class.getDeclaredField("textureMap"); field.setAccessible(true);
        @SuppressWarnings("unchecked")
        var slots = (Map<String, com.mojang.datafixers.util.Either<net.minecraft.client.resources.model.Material, String>>)field.get(value);
        for (var entry : slots.entrySet()) {
            var either = entry.getValue();
            if (either.right().isPresent()) {
                textures.put(entry.getKey(), map("reference", either.right().get()));
            } else {
                textures.put(entry.getKey(), map("sprite", either.left().get().texture().toString()));
            }
        }
        List<Object> elements = new ArrayList<>();
        for (var element : elements(value)) elements.add(element(element));
        Object display = null;
        if (value.getTransforms() != null) {
            var transforms = new TreeMap<String, Object>();
            for (var context : ItemDisplayContext.values()) {
                if (context != ItemDisplayContext.NONE) transforms.put(context.getSerializedName(), transform(value.getTransforms().getTransform(context)));
            }
            display = transforms;
        }
        return map("parent", parent(value), "ao", value.hasAmbientOcclusion(),
            "gui", value.getGuiLight() == null ? null : value.getGuiLight().name().toLowerCase(Locale.ROOT), "textures", textures, "elements", elements, "display", display);
    }
    public static void main(String[] args) throws Exception {
        net.minecraft.SharedConstants.tryDetectVersion();
        try (var input=Files.newBufferedReader(Path.of(args[0])); var output=Files.newBufferedWriter(Path.of(args[1]))) {
            for (String line; (line=input.readLine())!=null;) {
                var row=JsonParser.parseString(line).getAsJsonArray();
                BlockModel decoded;
                try { decoded=BlockModel.fromStream(new StringReader(row.get(1).getAsString())); }
                catch (RuntimeException failure) {
                    output.write(JSON.toJson(map("name", row.get(0).getAsString(), "error", true))); output.newLine(); continue;
                }
                output.write(JSON.toJson(map("name", row.get(0).getAsString(), "model", model(decoded)))); output.newLine();
            }
        }
    }
}
