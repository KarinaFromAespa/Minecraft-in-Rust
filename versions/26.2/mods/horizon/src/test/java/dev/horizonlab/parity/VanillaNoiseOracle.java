package dev.horizonlab.parity;

import it.unimi.dsi.fastutil.doubles.DoubleArrayList;
import java.io.PrintWriter;
import java.lang.reflect.Field;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.StringTokenizer;
import java.util.stream.IntStream;
import net.minecraft.util.RandomSource;
import net.minecraft.world.level.levelgen.DensityFunction;
import net.minecraft.world.level.levelgen.LegacyRandomSource;
import net.minecraft.world.level.levelgen.XoroshiroRandomSource;
import net.minecraft.world.level.levelgen.synth.BlendedNoise;
import net.minecraft.world.level.levelgen.synth.ImprovedNoise;
import net.minecraft.world.level.levelgen.synth.NoiseUtils;
import net.minecraft.world.level.levelgen.synth.NormalNoise;
import net.minecraft.world.level.levelgen.synth.PerlinNoise;
import net.minecraft.world.level.levelgen.synth.PerlinSimplexNoise;
import net.minecraft.world.level.levelgen.synth.SimplexNoise;

/** 26.2 transport adapter; samples the 26.2 synth classes directly. */
public final class VanillaNoiseOracle {
    private static int integer(StringTokenizer args) { return Integer.parseInt(args.nextToken()); }
    private static double number(StringTokenizer args) { return Double.longBitsToDouble(Long.parseUnsignedLong(args.nextToken(),16)); }
    private static String bits(double value) { return Long.toUnsignedString(Double.doubleToLongBits(value)); }
    private static RandomSource stream(String kind, long seed) {
        return switch (kind) {
            case "legacy" -> new LegacyRandomSource(seed);
            case "xoroshiro" -> new XoroshiroRandomSource(seed);
            default -> throw new AssertionError();
        };
    }
    private static Object declared(Object object, String name) throws Exception {
        Field field = object.getClass().getDeclaredField(name);
        field.setAccessible(true);
        return field.get(object);
    }
    private static String config(ImprovedNoise noise) {
        var sb = new StringBuilder();
        noise.parityConfigString(sb);
        return sb.toString();
    }
    private static String config(SimplexNoise noise) throws Exception {
        var sb = new StringBuilder();
        NoiseUtils.parityNoiseOctaveConfigString(sb, noise.xo, noise.yo, noise.zo, (int[]) declared(noise, "p"));
        return sb.toString();
    }
    private static String config(PerlinNoise noise) {
        var sb = new StringBuilder();
        noise.parityConfigString(sb);
        return sb.toString();
    }
    private static String config(NormalNoise noise) {
        var sb = new StringBuilder();
        noise.parityConfigString(sb);
        return sb.toString();
    }
    private static String config(BlendedNoise noise) {
        var sb = new StringBuilder();
        noise.parityConfigString(sb);
        return sb.toString();
    }

    public static void main(String[] paths) throws Exception {
        // BlendedNoise.compute takes a DensityFunction.FunctionContext, and
        // touching that type initializes DensityFunctions, which needs the
        // registries bootstrapped (same as the density oracle).
        net.minecraft.SharedConstants.tryDetectVersion();
        net.minecraft.server.Bootstrap.bootStrap();
        var improved = new ArrayList<ImprovedNoise>();
        var perlin = new ArrayList<PerlinNoise>();
        var simplex = new ArrayList<SimplexNoise>();
        var psimplex = new ArrayList<PerlinSimplexNoise>();
        var normal = new ArrayList<NormalNoise>();
        var blended = new ArrayList<BlendedNoise>();
        try (var reader = Files.newBufferedReader(Path.of(paths[0]), StandardCharsets.UTF_8);
            var out = new PrintWriter(Files.newBufferedWriter(Path.of(paths[1]), StandardCharsets.UTF_8))) {
            String line;
            while ((line = reader.readLine()) != null) {
                var args = new StringTokenizer(line);
                String operation = args.nextToken();
                switch (operation) {
                    case "wrap" -> out.println("wrap " + bits(PerlinNoise.wrap(number(args))));
                    case "bias" -> out.println("bias " + bits(NoiseUtils.biasTowardsExtreme(number(args), number(args))));
                    case "create-improved" -> {
                        RandomSource random = stream(args.nextToken(), Long.parseLong(args.nextToken()));
                        ImprovedNoise noise = new ImprovedNoise(random);
                        improved.add(noise);
                        out.printf("improved %s %s %s %d%n", bits(noise.xo), bits(noise.yo), bits(noise.zo), random.nextLong());
                        out.print("permutations");
                        for (byte v : (byte[]) declared(noise, "p")) out.print(" " + (v & 255));
                        out.println();
                        out.println("config " + config(noise));
                    }
                    case "query-improved", "local-improved" -> {
                        ImprovedNoise noise = improved.get(integer(args));
                        double x = number(args), y = number(args), z = number(args);
                        if (operation.equals("local-improved")) {
                            x -= noise.xo;
                            y -= noise.yo;
                            z -= noise.zo;
                        }
                        double yScale = number(args), yFudge = number(args);
                        double[] derivative = {number(args), number(args), number(args)};
                        out.printf("improved-query %s %s %s %s %s %s%n",
                            bits(noise.noise(x, y, z)),
                            bits(noise.noise(x, y, z, yScale, yFudge)),
                            bits(noise.noiseWithDerivative(x, y, z, derivative)),
                            bits(derivative[0]), bits(derivative[1]), bits(derivative[2]));
                    }
                    case "create-perlin" -> {
                        RandomSource random = stream(args.nextToken(), Long.parseLong(args.nextToken()));
                        int init = integer(args);
                        int firstOctave = integer(args);
                        double[] amplitudes = new double[integer(args)];
                        for (int i = 0; i < amplitudes.length; i++) amplitudes[i] = number(args);
                        PerlinNoise noise;
                        try {
                            noise = init == 0
                                ? PerlinNoise.create(random, firstOctave, new DoubleArrayList(amplitudes))
                                : PerlinNoise.createLegacyForLegacyNetherBiome(random, firstOctave, new DoubleArrayList(amplitudes));
                        } catch (IllegalArgumentException | IllegalStateException e) {
                            out.println("perlin error " + random.nextLong());
                            break;
                        }
                        perlin.add(noise);
                        out.printf("perlin %d %s%n", firstOctave, bits((double) declared(noise, "maxValue")));
                        out.println("perlin-config " + config(noise));
                        out.println("perlin-parent " + random.nextLong());
                    }
                    case "create-perlin-octaves" -> {
                        RandomSource random = stream(args.nextToken(), Long.parseLong(args.nextToken()));
                        int init = integer(args);
                        int[] octaves = new int[integer(args)];
                        for (int i = 0; i < octaves.length; i++) octaves[i] = integer(args);
                        PerlinNoise noise;
                        try {
                            noise = init == 0
                                ? PerlinNoise.create(random, Arrays.stream(octaves).boxed().toList())
                                : PerlinNoise.createLegacyForBlendedNoise(random, Arrays.stream(octaves));
                        } catch (IllegalArgumentException | IllegalStateException e) {
                            out.println("perlin error " + random.nextLong());
                            break;
                        }
                        perlin.add(noise);
                        out.printf("perlin %d %s%n", (int) declared(noise, "firstOctave"), bits((double) declared(noise, "maxValue")));
                        out.println("perlin-config " + config(noise));
                        out.println("perlin-parent " + random.nextLong());
                    }
                    case "query-perlin" -> {
                        PerlinNoise noise = perlin.get(integer(args));
                        double x = number(args), y = number(args), z = number(args);
                        double yScale = number(args), yFudge = number(args);
                        out.printf("perlin-query %s %s%n",
                            bits(noise.getValue(x, y, z)),
                            bits(noise.getValue(x, y, z, yScale, yFudge)));
                    }
                    case "perlin-maxbroken" -> {
                        PerlinNoise noise = perlin.get(integer(args));
                        out.println("perlin-maxbroken " + bits(noise.maxBrokenValue(number(args))));
                    }
                    case "create-simplex" -> {
                        RandomSource random = stream(args.nextToken(), Long.parseLong(args.nextToken()));
                        SimplexNoise noise = new SimplexNoise(random);
                        simplex.add(noise);
                        out.printf("simplex %s %s %s %d%n", bits(noise.xo), bits(noise.yo), bits(noise.zo), random.nextLong());
                        out.print("permutations");
                        // The table is int[512]; only the first 256 entries are filled.
                        int[] table = (int[]) declared(noise, "p");
                        for (int i = 0; i < 256; i++) out.print(" " + table[i]);
                        out.println();
                        out.println("config " + config(noise));
                    }
                    case "query-simplex" -> {
                        SimplexNoise noise = simplex.get(integer(args));
                        double x = number(args), y = number(args), z = number(args);
                        out.printf("simplex-query %s %s%n",
                            bits(noise.getValue(x, y)),
                            bits(noise.getValue(x, y, z)));
                    }
                    case "create-perlin-simplex" -> {
                        RandomSource random = stream(args.nextToken(), Long.parseLong(args.nextToken()));
                        List<Integer> octaves = new ArrayList<>();
                        int count = integer(args);
                        for (int i = 0; i < count; i++) octaves.add(integer(args));
                        PerlinSimplexNoise noise;
                        try {
                            noise = new PerlinSimplexNoise(random, octaves);
                        } catch (IllegalArgumentException e) {
                            out.println("perlin-simplex error " + random.nextLong());
                            break;
                        }
                        psimplex.add(noise);
                        out.printf("perlin-simplex %s %s%n",
                            bits((double) declared(noise, "highestFreqInputFactor")),
                            bits((double) declared(noise, "highestFreqValueFactor")));
                        SimplexNoise[] levels = (SimplexNoise[]) declared(noise, "noiseLevels");
                        for (int i = 0; i < levels.length; i++) {
                            if (levels[i] == null) {
                                out.printf("psimplex-level %d absent%n", i);
                            } else {
                                out.printf("psimplex-level %d %s %s %s%n", i,
                                    bits(levels[i].xo), bits(levels[i].yo), bits(levels[i].zo));
                            }
                        }
                        out.println("perlin-simplex-parent " + random.nextLong());
                    }
                    case "query-perlin-simplex" -> {
                        PerlinSimplexNoise noise = psimplex.get(integer(args));
                        double x = number(args), y = number(args);
                        boolean useStart = integer(args) != 0;
                        out.println("psimplex-query " + bits(noise.getValue(x, y, useStart)));
                    }
                    case "normal" -> {
                        RandomSource random = stream(args.nextToken(), Long.parseLong(args.nextToken()));
                        int legacy = integer(args);
                        int firstOctave = integer(args);
                        double[] amplitudes = new double[integer(args)];
                        for (int i = 0; i < amplitudes.length; i++) amplitudes[i] = number(args);
                        double[][] points = new double[integer(args)][3];
                        for (double[] point : points) for (int i = 0; i < 3; i++) point[i] = number(args);
                        NormalNoise noise;
                        try {
                            if (legacy != 0) {
                                var params = new NormalNoise.NoiseParameters(firstOctave, Arrays.stream(amplitudes).boxed().toList());
                                noise = NormalNoise.createLegacyNetherBiome(random, params);
                            } else {
                                noise = NormalNoise.create(random, firstOctave, amplitudes);
                            }
                        } catch (IllegalArgumentException | IllegalStateException e) {
                            out.println("normal error " + random.nextLong());
                            break;
                        }
                        normal.add(noise);
                        out.printf("normal %s %s%n",
                            bits((double) declared(noise, "valueFactor")),
                            bits(noise.maxValue()));
                        out.println("normal-config " + config(noise));
                        for (double[] point : points) {
                            out.println("normal-query " + bits(noise.getValue(point[0], point[1], point[2])));
                        }
                        out.println("normal-parent " + random.nextLong());
                    }
                    case "blended", "blended-unseeded" -> {
                        RandomSource random = null;
                        if (operation.equals("blended")) {
                            random = stream(args.nextToken(), Long.parseLong(args.nextToken()));
                        }
                        double xzScale = number(args), yScale = number(args), xzFactor = number(args),
                            yFactor = number(args), smear = number(args);
                        int[][] points = new int[integer(args)][3];
                        for (int[] point : points) for (int i = 0; i < 3; i++) point[i] = integer(args);
                        BlendedNoise noise;
                        try {
                            noise = operation.equals("blended")
                                ? new BlendedNoise(random, xzScale, yScale, xzFactor, yFactor, smear)
                                : BlendedNoise.createUnseeded(xzScale, yScale, xzFactor, yFactor, smear);
                        } catch (IllegalArgumentException | IllegalStateException e) {
                            out.println("blended error" + (random == null ? "" : " " + random.nextLong()));
                            break;
                        }
                        blended.add(noise);
                        out.printf("blended %s %s%n", bits(noise.maxValue()), bits(noise.minValue()));
                        out.println("blended-config " + config(noise));
                        for (int[] point : points) {
                            int bx = point[0], by = point[1], bz = point[2];
                            var context = new DensityFunction.FunctionContext() {
                                public int blockX() { return bx; }
                                public int blockY() { return by; }
                                public int blockZ() { return bz; }
                            };
                            out.println("blended-query " + bits(noise.compute(context)));
                        }
                        if (random != null) out.println("blended-parent " + random.nextLong());
                    }
                    default -> throw new AssertionError();
                }
                if (args.hasMoreTokens()) throw new AssertionError("extra argument");
            }
            if (out.checkError()) throw new java.io.IOException("reference output failed");
        }
    }
}
