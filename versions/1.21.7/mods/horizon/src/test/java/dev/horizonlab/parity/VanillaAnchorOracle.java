package dev.horizonlab.parity;
import com.google.gson.*;
import com.mojang.serialization.JsonOps;
import java.io.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;
import net.minecraft.core.*;
import net.minecraft.world.level.*;
import net.minecraft.world.level.biome.*;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.levelgen.*;

/** Original transport for vertical anchor codecs and generation contexts. */
public final class VanillaAnchorOracle {
    // The fixture's sea level for the current case: 1.21.7's
    // WorldGenerationContext has no seaLevel() and no sea-level anchor (see
    // below), so the oracle tracks it to emulate those anchors.
    static int seaLevel;
    private static WorldGenerationContext context(JsonObject f) {
        var v=f.getAsJsonArray("context");int min=v.get(0).getAsInt(),height=v.get(1).getAsInt(),sea=v.get(2).getAsInt();
        // 1.21.7 has no WorldGenerationContext.of(LevelAccessor): the context
        // is always (ChunkGenerator, LevelHeightAccessor), so the
        // no-generator case builds a default generator too. 1.21.7's
        // NoiseSettings is 4-arg (noise sizes 1, 2: the vanilla overworld
        // defaults), its NoiseRouter takes 15 density functions, its
        // NoiseGeneratorSettings takes a SurfaceRules RuleSource (no
        // BlockRule, no DebugFunctions, four trailing booleans), and
        // DensityFunctions lives directly under levelgen. None of the
        // noise settings affect anchor resolution, which reads only the
        // height accessor (and the sea level, tracked above).
        var zero=DensityFunctions.zero();
        NoiseSettings noise;
        int level;
        if(f.has("generator")) {
            var g=f.getAsJsonArray("generator");
            noise=new NoiseSettings(g.get(0).getAsInt(),g.get(1).getAsInt(),1,2);
            level=g.get(2).getAsInt();
        } else {
            noise=new NoiseSettings(min,height,1,2);
            level=sea;
        }
        seaLevel=level;
        var settings=new NoiseGeneratorSettings(noise,Blocks.STONE.defaultBlockState(),Blocks.WATER.defaultBlockState(),new NoiseRouter(zero,zero,zero,zero,zero,zero,zero,zero,zero,zero,zero,zero,zero,zero,zero),SurfaceRules.state(Blocks.STONE.defaultBlockState()),List.of(),level,false,false,false,false);
        var biome=Holder.Reference.createStandAlone(new HolderOwner<Biome>(){},Biomes.PLAINS);
        var generator=new NoiseBasedChunkGenerator(new FixedBiomeSource(biome),Holder.direct(settings));
        return new WorldGenerationContext(generator,LevelHeightAccessor.create(min,height));
    }
    private static void anchor(VerticalAnchor anchor,WorldGenerationContext context,PrintWriter out) {
        var encoded=VerticalAnchor.CODEC.encodeStart(JsonOps.INSTANCE,anchor).result();
        out.println("anchor "+anchor.resolveY(context)+" "+encoded.map(Object::toString).orElse("error"));
    }
    public static void main(String[] args) throws Exception {
        net.minecraft.SharedConstants.tryDetectVersion();net.minecraft.server.Bootstrap.bootStrap();
        Path dir=Path.of(args[1]);
        try(var out=new PrintWriter(Files.newBufferedWriter(dir.resolve(args[0]+".txt"),StandardCharsets.UTF_8))) {
            for(String line:Files.readAllLines(dir.resolve("inputs.txt"))) {
                var f=JsonParser.parseString(line).getAsJsonObject();var context=context(f);
                out.println("context "+context.getMinGenY()+" "+context.getGenDepth()+" "+seaLevel);
                anchor(VerticalAnchor.bottom(),context,out);anchor(VerticalAnchor.top(),context,out);
                // 1.21.7 has no sea-level anchor (its VerticalAnchor has only
                // absolute/aboveBottom/belowTop): the sea-level anchors are
                // emulated as absolute(seaLevel [+ value]), which resolves to
                // the same Y; the codec encoding differs (absolute, not a
                // sea-level type), a real 1.21.7-vs-26.x difference the
                // family batch triages.
                anchor(VerticalAnchor.absolute(seaLevel),context,out);
                for(var e:f.getAsJsonArray("values")) {
                    int value=e.getAsInt();
                    for(var a:List.of(VerticalAnchor.absolute(value),VerticalAnchor.aboveBottom(value),VerticalAnchor.belowTop(value),VerticalAnchor.absolute(seaLevel+value)))anchor(a,context,out);
                }
                for(var e:f.getAsJsonArray("json")) {
                    var result=VerticalAnchor.CODEC.parse(JsonOps.INSTANCE,JsonParser.parseString(e.getAsString())).result();
                    if(result.isPresent())anchor(result.get(),context,out);else out.println("error");
                }
            }
        }
    }
}
