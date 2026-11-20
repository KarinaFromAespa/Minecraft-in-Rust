package dev.horizonlab.parity;

import com.google.gson.*;
import java.lang.reflect.Field;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;
import net.minecraft.SharedConstants;
import net.minecraft.client.model.geom.LayerDefinitions;
import net.minecraft.client.model.geom.ModelPart;
import net.minecraft.server.Bootstrap;

/**
 * Original exporter of every baked entity model layer (part tree, initial poses and
 * textured polygons) produced by vanilla's own LayerDefinitions.
 */
public final class VanillaEntityModelExporter {
    private static Field cubes;
    private static Field children;

    @SuppressWarnings("unchecked")
    private static JsonObject part(ModelPart part) throws Exception {
        var o = new JsonObject();
        var pose = new JsonArray();
        for (float f : new float[] {part.x, part.y, part.z, part.xRot, part.yRot, part.zRot, part.xScale, part.yScale, part.zScale})
            pose.add(f);
        o.add("pose", pose);
        var polygons = new JsonArray();
        var normals = new JsonArray();
        for (ModelPart.Cube cube : (List<ModelPart.Cube>) cubes.get(part)) {
            for (ModelPart.Polygon polygon : cube.polygons) {
                var p = new JsonArray();
                for (ModelPart.Vertex v : polygon.vertices()) {
                    var vertex = new JsonArray();
                    // Degenerate (zero-size) faces have NaN UVs: write them as 0.
                    // 1.21.8's Vertex is a (pos, u, v) record: no x()/y()/z().
                    for (float f : new float[] {v.pos().x, v.pos().y, v.pos().z, v.u(), v.v()})
                        vertex.add(Float.isFinite(f) ? f : 0.0F);
                    p.add(vertex);
                }
                polygons.add(p);
                var normal = new JsonArray();
                normal.add(polygon.normal().x());
                normal.add(polygon.normal().y());
                normal.add(polygon.normal().z());
                normals.add(normal);
            }
        }
        o.add("polygons", polygons);
        o.add("normals", normals);
        var kids = new JsonObject();
        for (var entry : ((Map<String, ModelPart>) children.get(part)).entrySet())
            kids.add(entry.getKey(), part(entry.getValue()));
        o.add("children", kids);
        return o;
    }

    // 1.21.8 registers no separate warden layer definitions: its renderer
    // draws the emissive layers as part lists from the one WARDEN model
    // (WardenModel.get*LayerModelParts), while 1.21.9 bakes four layers from
    // create*Layer factories. The Rust draws the layers as separate models
    // (mobs/warden.rs emissive_layers), so synthesize the four keys here by
    // filtering the baked WARDEN tree to the parts each getter lists — the
    // same part names 1.21.9's factories retain (the drawn result differs
    // for pulsating spots: see retainParts).
    private static final Map<String, Set<String>> WARDEN_LAYERS = Map.of(
        "tendrils", Set.of("left_tendril", "right_tendril"),
        "heart", Set.of("body"),
        "bioluminescent", Set.of("head", "left_arm", "right_arm", "left_leg", "right_leg"),
        "pulsating_spots", Set.of("body", "head", "left_arm", "right_arm", "left_leg", "right_leg")
    );

    // A copy of `tree` with only `keep`'s parts left drawn, mirroring 1.21.8's
    // LivingEntityEmissiveLayer: it sets skipDraw on every part of the model,
    // then clears it on the listed ones, and ModelPart.render skips only a
    // part's own cubes (its children still render). So a part draws exactly
    // when its name is in the set, whatever its ancestors do — NOT 1.21.9's
    // retainExactParts, which guts a kept part's whole subtree (1.21.9's
    // baked pulsating-spots layer lost the head and arms that way; 1.21.8's
    // still glows there). Poses are untouched, so the kept parts keep their
    // transform chain (the tendrils still hang under the head's rotation).
    private static JsonObject retainParts(JsonObject tree, Set<String> keep) {
        var copy = tree.deepCopy();
        retainInto(copy, "", keep);
        return copy;
    }

    private static void retainInto(JsonObject node, String name, Set<String> keep) {
        if (!keep.contains(name)) {
            node.add("polygons", new JsonArray());
            node.add("normals", new JsonArray());
        }
        for (var entry : node.getAsJsonObject("children").entrySet()) {
            retainInto(entry.getValue().getAsJsonObject(), entry.getKey(), keep);
        }
    }

    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        cubes = ModelPart.class.getDeclaredField("cubes");
        cubes.setAccessible(true);
        children = ModelPart.class.getDeclaredField("children");
        children.setAccessible(true);
        var out = new JsonObject();
        for (var entry : LayerDefinitions.createRoots().entrySet()) {
            try {
                out.add(entry.getKey().toString(), part(entry.getValue().bakeRoot()));
            } catch (RuntimeException error) {
                System.err.println("skipped " + entry.getKey() + ": " + error);
            }
        }
        var warden = out.getAsJsonObject("minecraft:warden#main");
        if (warden != null) {
            for (var layer : WARDEN_LAYERS.entrySet()) {
                out.add("minecraft:warden#" + layer.getKey(), retainParts(warden, layer.getValue()));
            }
        }
        Files.writeString(Path.of(args[0]), new Gson().toJson(out), StandardCharsets.UTF_8);
    }
}
