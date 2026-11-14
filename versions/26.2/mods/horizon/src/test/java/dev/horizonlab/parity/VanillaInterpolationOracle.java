package dev.horizonlab.parity;

import java.io.PrintWriter;
import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Objects;
import net.minecraft.util.Mth;
import net.minecraft.world.phys.Vec3;

/**
 * How a client walks a living entity along the positions its server sends: the unmodified
 * {@code InterpolationHandler.InterpolationData} (decrease, addDelta, addRotation), driven the way
 * {@code InterpolationHandler.interpolateTo} / {@code interpolate} drive it, with the entity's own
 * {@code setPos} / {@code setRot}. The handler's entity hooks are out of scope: the entity moves
 * only by its interpolation (no deltas to adjust, and no level to check the target's collision
 * against) at the normal tick rate.
 *
 * <p>Input (args[0]), one event a line: {@code case STEPS X Y Z YAW PITCH} (a fresh entity there
 * with a handler of that many interpolation steps), {@code move L X Y Z} or {@code move N} (no
 * position), then {@code R YAW PITCH} or {@code -} (no rotation), and {@code tick}. Output (args[1]):
 * the entity after every event.
 */
public final class VanillaInterpolationOracle {
    private static Method method(Class<?> owner, String name, Class<?>... types) throws Exception {
        Method m = owner.getDeclaredMethod(name, types);
        m.setAccessible(true);
        return m;
    }

    private static Field field(Class<?> owner, String name) throws Exception {
        Field f = owner.getDeclaredField(name);
        f.setAccessible(true);
        return f;
    }

    private static String hex(double v) {
        return Long.toHexString(Double.doubleToRawLongBits(v));
    }

    private static String hex(float v) {
        return Integer.toHexString(Float.floatToRawIntBits(v));
    }

    public static void main(String[] args) throws Exception {
        Class<?> dataClass = Class.forName("net.minecraft.world.entity.InterpolationHandler$InterpolationData");
        Constructor<?> create = dataClass.getDeclaredConstructor(int.class, Vec3.class, float.class, float.class);
        create.setAccessible(true);
        Method decrease = method(dataClass, "decrease");
        Method addDelta = method(dataClass, "addDelta", Vec3.class);
        Method addRotation = method(dataClass, "addRotation", float.class, float.class);
        Field stepsField = field(dataClass, "steps");
        Field positionField = field(dataClass, "position");
        Field yRotField = field(dataClass, "yRot");
        Field xRotField = field(dataClass, "xRot");

        Object data = null;
        int steps = 0;
        Vec3 pos = Vec3.ZERO;
        float yRot = 0;
        float xRot = 0;
        Vec3 prevPos = null;
        Float prevRotX = null;
        Float prevRotY = null;
        try (PrintWriter out = new PrintWriter(Files.newBufferedWriter(Path.of(args[1])))) {
            for (String line : Files.readAllLines(Path.of(args[0]))) {
                String[] f = line.trim().split(" ");
                int i = 1;
                String what;
                switch (f[0]) {
                    case "case" -> {
                        data = create.newInstance(0, Vec3.ZERO, 0.0F, 0.0F);
                        steps = Integer.parseInt(f[1]);
                        pos = new Vec3(Double.parseDouble(f[2]), Double.parseDouble(f[3]), Double.parseDouble(f[4]));
                        yRot = Float.parseFloat(f[5]);
                        xRot = Float.parseFloat(f[6]);
                        prevPos = null;
                        prevRotX = null;
                        prevRotY = null;
                        what = "c";
                    }
                    case "move" -> {
                        Vec3 target = null;
                        switch (f[i++]) {
                            case "L" -> {
                                target = new Vec3(Double.parseDouble(f[i]), Double.parseDouble(f[i + 1]), Double.parseDouble(f[i + 2]));
                                i += 3;
                            }
                            default -> {
                            }
                        }
                        boolean hasRotation = f[i].equals("R");
                        float y = hasRotation ? Float.parseFloat(f[i + 1]) : 0;
                        float x = hasRotation ? Float.parseFloat(f[i + 2]) : 0;
                        // InterpolationHandler.interpolateTo: the heading is the
                        // target while interpolating, else the entity's own.
                        boolean active = stepsField.getInt(data) > 0;
                        Vec3 currentPos = active ? (Vec3) positionField.get(data) : pos;
                        float currentY = active ? yRotField.getFloat(data) : yRot;
                        float currentX = active ? xRotField.getFloat(data) : xRot;
                        if (target == null) target = currentPos;
                        if (!hasRotation) {
                            y = currentY;
                            x = currentX;
                        }
                        if (steps == 0) {
                            // snapTo, then cancel.
                            pos = target;
                            yRot = y % 360.0F;
                            xRot = Math.clamp(x % 360.0F % 360.0F, -90.0F, 90.0F);
                            stepsField.setInt(data, 0);
                            prevPos = null;
                            prevRotX = null;
                            prevRotY = null;
                            what = "s";
                        } else if (active
                            && Objects.equals(yRotField.getFloat(data), y)
                            && Objects.equals(xRotField.getFloat(data), x)
                            && Objects.equals(positionField.get(data), target)) {
                            what = "k";
                        } else {
                            stepsField.setInt(data, steps);
                            positionField.set(data, target);
                            yRotField.setFloat(data, y);
                            xRotField.setFloat(data, x);
                            prevPos = pos;
                            prevRotX = xRot;
                            prevRotY = yRot;
                            what = "i";
                        }
                    }
                    case "tick" -> {
                        // InterpolationHandler.interpolate.
                        if (stepsField.getInt(data) <= 0) {
                            stepsField.setInt(data, 0);
                            prevPos = null;
                            prevRotX = null;
                            prevRotY = null;
                            what = "n";
                        } else {
                            double alpha = 1.0 / stepsField.getInt(data);
                            if (prevPos != null) {
                                // No level here to check the collision against;
                                // the delta is zero (the entity moves only by
                                // its interpolation), so the target is unchanged.
                                addDelta.invoke(data, pos.subtract(prevPos));
                            }
                            if (prevRotX != null) {
                                addRotation.invoke(data, yRot - prevRotY, xRot - prevRotX);
                            }
                            Vec3 target = (Vec3) positionField.get(data);
                            Vec3 newPosition = new Vec3(
                                Mth.lerp(alpha, pos.x, target.x),
                                Mth.lerp(alpha, pos.y, target.y),
                                Mth.lerp(alpha, pos.z, target.z));
                            float newYRot = (float) Mth.rotLerp(alpha, yRot, yRotField.getFloat(data));
                            float newXRot = (float) Mth.lerp(alpha, xRot, xRotField.getFloat(data));
                            pos = newPosition;
                            yRot = newYRot % 360.0F;
                            xRot = Math.clamp(newXRot % 360.0F % 360.0F, -90.0F, 90.0F);
                            decrease.invoke(data);
                            prevPos = newPosition;
                            prevRotX = xRot;
                            prevRotY = yRot;
                            what = "t";
                        }
                    }
                    default -> throw new IllegalArgumentException(line);
                }
                out.println(what + " " + hex(pos.x) + " " + hex(pos.y) + " " + hex(pos.z) + " " + hex(yRot) + " " + hex(xRot));
            }
        }
    }
}
