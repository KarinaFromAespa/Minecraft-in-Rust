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
    // 1.21.1 Cube.polygons is private and Polygon/Vertex are package-private
    // classes with plain fields (not records): read them reflectively.
    private static Field polygons;
    private static Field vertices;
    private static Field polygonNormal;
    private static Field vertexPos;
    private static Field vertexU;
    private static Field vertexV;

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
            for (Object polygon : (Object[]) VanillaEntityModelExporter.polygons.get(cube)) {
                var p = new JsonArray();
                for (Object v : (Object[]) vertices.get(polygon)) {
                    var vertex = new JsonArray();
                    // Degenerate (zero-size) faces have NaN UVs: write them as 0.
                    var pos = (org.joml.Vector3f) vertexPos.get(v);
                    for (float f : new float[] {pos.x, pos.y, pos.z, vertexU.getFloat(v), vertexV.getFloat(v)})
                        vertex.add(Float.isFinite(f) ? f : 0.0F);
                    p.add(vertex);
                }
                polygons.add(p);
                var n = (org.joml.Vector3f) polygonNormal.get(polygon);
                var normal = new JsonArray();
                normal.add(n.x);
                normal.add(n.y);
                normal.add(n.z);
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

    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        cubes = ModelPart.class.getDeclaredField("cubes");
        cubes.setAccessible(true);
        children = ModelPart.class.getDeclaredField("children");
        children.setAccessible(true);
        polygons = ModelPart.Cube.class.getDeclaredField("polygons");
        polygons.setAccessible(true);
        var polygonClass = Class.forName("net.minecraft.client.model.geom.ModelPart$Polygon");
        vertices = polygonClass.getDeclaredField("vertices");
        vertices.setAccessible(true);
        polygonNormal = polygonClass.getDeclaredField("normal");
        polygonNormal.setAccessible(true);
        var vertexClass = Class.forName("net.minecraft.client.model.geom.ModelPart$Vertex");
        vertexPos = vertexClass.getDeclaredField("pos");
        vertexPos.setAccessible(true);
        vertexU = vertexClass.getDeclaredField("u");
        vertexU.setAccessible(true);
        vertexV = vertexClass.getDeclaredField("v");
        vertexV.setAccessible(true);
        var out = new JsonObject();
        for (var entry : LayerDefinitions.createRoots().entrySet()) {
            try {
                out.add(entry.getKey().toString(), part(entry.getValue().bakeRoot()));
            } catch (RuntimeException error) {
                System.err.println("skipped " + entry.getKey() + ": " + error);
            }
        }
        Files.writeString(Path.of(args[0]), new Gson().toJson(out), StandardCharsets.UTF_8);
    }
}
