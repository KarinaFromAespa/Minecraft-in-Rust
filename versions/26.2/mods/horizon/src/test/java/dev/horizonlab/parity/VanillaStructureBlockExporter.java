package dev.horizonlab.parity;

import com.google.gson.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import net.minecraft.SharedConstants;
import net.minecraft.core.BlockPos;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.RandomizableContainer;
import net.minecraft.world.level.EmptyBlockGetter;
import net.minecraft.world.level.block.*;
import net.minecraft.world.level.block.state.BlockState;

/**
 * Original exporter of per-state structure-template behavior: BlockState.rotate
 * and BlockState.mirror results, StructureTemplate's full-block ordering class
 * whether the state's block entity is a RandomizableContainer, and the block
 * entity type of every state that has one.
 */
public final class VanillaStructureBlockExporter {
    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion();Bootstrap.bootStrap();
        var entityTypes=new JsonObject();
        var rotate=new JsonArray();var mirror=new JsonArray();var full=new JsonArray();var loot=new JsonArray();
        Rotation[] rotations={Rotation.CLOCKWISE_90,Rotation.CLOCKWISE_180,Rotation.COUNTERCLOCKWISE_90};
        Mirror[] mirrors={Mirror.LEFT_RIGHT,Mirror.FRONT_BACK};
        for(BlockState state:Block.BLOCK_STATE_REGISTRY) {
            var r=new JsonArray();for(var rotation:rotations)r.add(Block.getId(state.rotate(rotation)));rotate.add(r);
            var m=new JsonArray();for(var value:mirrors)m.add(Block.getId(state.mirror(value)));mirror.add(m);
            full.add(!state.getBlock().hasDynamicShape()&&state.isCollisionShapeFullBlock(EmptyBlockGetter.INSTANCE,BlockPos.ZERO));
            boolean container=false;
            if(state.getBlock() instanceof EntityBlock entityBlock) {
                try{container=entityBlock.newBlockEntity(BlockPos.ZERO,state) instanceof RandomizableContainer;}catch(Throwable ignored){}
            }
            if(container)loot.add(Block.getId(state));
            if(state.hasBlockEntity()&&state.getBlock() instanceof EntityBlock entityBlock) {
                boolean recorded=false;
                try {
                    var type=net.minecraft.core.registries.BuiltInRegistries.BLOCK_ENTITY_TYPE.getKey(entityBlock.newBlockEntity(BlockPos.ZERO,state).getType()).toString();
                    if(!entityTypes.has(type))entityTypes.add(type,new JsonArray());
                    entityTypes.getAsJsonArray(type).add(Block.getId(state));
                    recorded=true;
                }catch(Throwable ignored){}
                if(!recorded) {
                    // Some block entities cannot be constructed at BlockPos.ZERO (the vault's
                    // throws), so construction alone silently drops their states from the map;
                    // fall back to the type registry's validity check, which needs no instance.
                    for(var type: net.minecraft.core.registries.BuiltInRegistries.BLOCK_ENTITY_TYPE) {
                        if(type.isValid(state)) {
                            var key=net.minecraft.core.registries.BuiltInRegistries.BLOCK_ENTITY_TYPE.getKey(type).toString();
                            if(!entityTypes.has(key))entityTypes.add(key,new JsonArray());
                            entityTypes.getAsJsonArray(key).add(Block.getId(state));
                            break;
                        }
                    }
                }
            }
        }
        var result=new JsonObject();
        result.addProperty("states",Block.BLOCK_STATE_REGISTRY.size());
        result.add("rotate",rotate);result.add("mirror",mirror);result.add("full_block",full);result.add("loot_containers",loot);result.add("block_entities",entityTypes);
        Files.writeString(Path.of(args[0]),new Gson().toJson(result),StandardCharsets.UTF_8);
    }
}
