package dev.horizonlab.parity;
import com.google.gson.*;
import com.mojang.serialization.JsonOps;
import java.io.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.function.Function;
import net.minecraft.core.*;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.Identifier;
import net.minecraft.resources.RegistryOps;
import net.minecraft.tags.*;
import net.minecraft.util.RandomSource;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.LevelHeightAccessor;
import net.minecraft.world.level.biome.*;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.chunk.*;
import net.minecraft.world.level.levelgen.*;
import net.minecraft.world.level.levelgen.DensityFunctions;
import net.minecraft.world.level.levelgen.carver.*;

/** Original transport calling actual decoded cave carvers and tracing mask writes.
 * 26.2 carves through {@link ConfiguredWorldCarver}: the context is a real
 * {@link CarvingContext} over the overworld or nether settings, the chunk is
 * an empty proto chunk at the target position (air is not replaceable, so the
 * mask — the only compared output — is unaffected by chunk contents), and the
 * aquifer is disabled. The mask subclass traces first-claim writes in order. */
public final class VanillaCaveCarverOracle {
    private static final class OutputFailure extends RuntimeException {}
    private static PalettedContainerFactory factory;
    private static final class TracingMask extends CarvingMask {
        final PrintWriter out;
        final int fail;
        int calls;
        TracingMask(int height,int minY,PrintWriter out,int fail) {super(height,minY);this.out=out;this.fail=fail;}
        @Override public void set(int x,int y,int z) {
            out.println("cell "+x+" "+y+" "+z);
            int previous=calls++;
            if(previous==fail)throw new OutputFailure();
            super.set(x,y,z);
        }
    }
    static void bindTags(JsonObject resources) {
        var registry=BuiltInRegistries.BLOCK;
        var loader=new TagLoader<Holder<Block>>((id,required)->registry.get(id).map(value->(Holder<Block>)value),"tags/block");
        var definitions=new java.util.HashMap<Identifier,java.util.List<TagLoader.EntryWithSource>>();
        for(var item:resources.entrySet()) {
            var file=TagFile.CODEC.parse(JsonOps.INSTANCE,JsonParser.parseString(item.getValue().getAsString())).getOrThrow();
            var entries=new java.util.ArrayList<TagLoader.EntryWithSource>();
            for(var entry:file.entries())entries.add(new TagLoader.EntryWithSource(entry,"vanilla"));
            definitions.put(Identifier.parse(item.getKey()),entries);
        }
        var tags=new java.util.HashMap<TagKey<Block>,java.util.List<Holder<Block>>>();
        loader.build(definitions).forEach((id,values)->tags.put(TagKey.create(Registries.BLOCK,id),java.util.List.copyOf(values)));
        if(tags.size()!=resources.size())throw new AssertionError("incomplete block tags");
        registry.prepareTagReload(new TagLoader.LoadResult<>(Registries.BLOCK,tags)).apply();
    }
    public static void main(String[] args) throws Exception {
        net.minecraft.SharedConstants.tryDetectVersion();net.minecraft.server.Bootstrap.bootStrap();Path dir=Path.of(args[1]);bindTags(JsonParser.parseString(Files.readString(dir.resolve("tags.json"))).getAsJsonObject());
        var ops=RegistryOps.create(JsonOps.INSTANCE,RegistryAccess.fromRegistryOfRegistries(BuiltInRegistries.REGISTRY));
        Holder<Biome> biome=Holder.Reference.createStandAlone(new HolderOwner<Biome>(){},Biomes.PLAINS);
        var ids=new IdMapper<Holder<Biome>>();ids.add(biome);
        factory=new PalettedContainerFactory(Strategy.createForBlockStates(Block.BLOCK_STATE_REGISTRY),Blocks.AIR.defaultBlockState(),null,Strategy.createForBiomes(ids),biome,null);
        Holder<Biome> plains=biome;
        var aquifer=Aquifer.createDisabled((x,y,z)->new Aquifer.FluidStatus(-1,Blocks.AIR.defaultBlockState()));
        Function<net.minecraft.core.BlockPos,Holder<Biome>> biomeGetter=pos->plains;
        try(var out=new PrintWriter(Files.newBufferedWriter(dir.resolve(args[0]+".txt"),StandardCharsets.UTF_8))) {
            for(var line:Files.readAllLines(dir.resolve("inputs.txt"))) {
                var f=JsonParser.parseString(line).getAsJsonObject();long seed=f.get("seed").getAsLong();
                var v=f.getAsJsonArray("context");int minY=v.get(0).getAsInt(),height=v.get(1).getAsInt(),sea=v.get(2).getAsInt();
                RandomSource random=f.get("legacy").getAsBoolean()?new LegacyRandomSource(seed):new XoroshiroRandomSource(seed);
                var decoded=ConfiguredWorldCarver.DIRECT_CODEC.parse(ops,JsonParser.parseString(f.get("definition").getAsString()));
                if(decoded.result().isEmpty())out.println("decode error");else {
                    var carver=decoded.result().get();out.println("decode ok");
                    var zero=DensityFunctions.zero();
                    var router=new NoiseRouter(zero,zero,zero,zero,zero,zero,zero,zero,zero,zero,zero,zero,zero,zero,zero);
                    var settings=new NoiseGeneratorSettings(new NoiseSettings(minY,height,1,2),Blocks.STONE.defaultBlockState(),Blocks.WATER.defaultBlockState(),router,SurfaceRules.state(Blocks.STONE.defaultBlockState()),java.util.List.of(),sea,false,false,false,false);
                    var generator=new NoiseBasedChunkGenerator(new FixedBiomeSource(plains),Holder.direct(settings));
                    var context=new CarvingContext(generator,null,LevelHeightAccessor.create(minY,height),null,null,null);
                    var t=f.getAsJsonArray("target");var o=f.getAsJsonArray("origin");var target=new ChunkPos(t.get(0).getAsInt(),t.get(1).getAsInt());var origin=new ChunkPos(o.get(0).getAsInt(),o.get(1).getAsInt());
                    var chunk=new ProtoChunk(target,UpgradeData.EMPTY,LevelHeightAccessor.create(minY,height),factory,null);
                    var mask=new TracingMask(height-8,minY+1,out,f.get("fail").getAsInt());
                    for(int repeat=0;repeat<f.get("repeats").getAsInt();repeat++) {
                        out.println("start "+carver.isStartChunk(random));mask.calls=0;
                        try{out.println("carve "+carver.carve(context,chunk,biomeGetter,random,aquifer,origin,mask)+" "+mask.calls);}catch(OutputFailure|IllegalArgumentException error){out.println("carve error "+mask.calls);}
                        for(int x=0;x<16;x++)for(int z=0;z<16;z++) {
                            int low=Integer.MIN_VALUE;
                            for(int y=minY+1;y<=minY+height-8;y++) {
                                if(mask.get(x,y,z)) {if(low==Integer.MIN_VALUE)low=y;}
                                else if(low!=Integer.MIN_VALUE) {out.println("column "+x+" "+z+" "+low+" "+(y-1));low=Integer.MIN_VALUE;}
                            }
                            if(low!=Integer.MIN_VALUE)out.println("column "+x+" "+z+" "+low+" "+(minY+height-8));
                        }
                        out.println("next "+random.nextLong());
                    }
                }
                out.println("tail "+random.nextLong());
            }
        }
    }
}
