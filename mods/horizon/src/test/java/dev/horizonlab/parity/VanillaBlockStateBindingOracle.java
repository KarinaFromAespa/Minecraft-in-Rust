package dev.horizonlab.parity;

import com.google.gson.*;
import java.nio.file.*;
import java.util.*;
import net.minecraft.client.renderer.block.model.BlockModelDefinition;
import net.minecraft.client.renderer.block.model.MultiVariant;
import net.minecraft.client.renderer.block.model.UnbakedBlockStateModel;
import net.minecraft.client.renderer.block.model.multipart.MultiPart;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.StateDefinition;
import net.minecraft.world.level.block.state.properties.Property;
import com.google.gson.JsonParser;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.IntegerProperty;
import static dev.horizonlab.parity.VanillaCuboidModelOracle.map;

/** Native registry binding and multipart visual-equality selection, without a GPU. */
public final class VanillaBlockStateBindingOracle {
    private static java.util.function.Function<ResourceLocation,StateDefinition<Block,BlockState>> mapper;
    private static final Gson JSON = new GsonBuilder().serializeNulls().create();
    private static Object field(Object value,String name) throws Exception {
        var f=value.getClass().getDeclaredField(name);f.setAccessible(true);return f.get(value);
    }
    private static void export(Path path) throws Exception {
        var blocks=new TreeMap<String,Object>();
        var ids=new TreeSet<>(BuiltInRegistries.BLOCK.keySet());
        ids.add(ResourceLocation.parse("minecraft:item_frame"));ids.add(ResourceLocation.parse("minecraft:glow_item_frame"));
        for(var id:ids) {
            var definition=Objects.requireNonNull(mapper.apply(id));
            var schema=new TreeMap<String,Object>();
            for(var property:definition.getProperties()) {
                if(property instanceof IntegerProperty integer) {
                    var values=integer.getPossibleValues();schema.put(property.getName(),map("integer",new int[]{Collections.min(values),Collections.max(values)}));
                } else {
                    var values=values(property);
                    schema.put(property.getName(),map("named",values));
                }
            }
            var states=new ArrayList<Object>();
            for(var state:definition.getPossibleStates()) {
                var properties=new TreeMap<String,String>();
                for(var property:definition.getProperties())properties.put(property.getName(),value(state,property));
                states.add(properties);
            }
            blocks.put(id.toString(),map("schema",schema,"states",states));
        }
        Files.writeString(path,JSON.toJson(blocks));
    }
    private static <T extends Comparable<T>> Set<String> values(Property<T> property) {
        var values=new TreeSet<String>();for(var value:property.getPossibleValues())values.add(property.getName(value));return values;
    }
    private static <T extends Comparable<T>> String value(BlockState state,Property<T> property) {return property.getName(state.getValue(property));}
    @SuppressWarnings("unchecked")
    private static Object bind(JsonObject row) throws Exception {
        var states=Objects.requireNonNull(mapper.apply(ResourceLocation.parse(row.get("block").getAsString())));
        // 1.21.4 has no BlockModelDefinition.CODEC: parse with fromJsonElement.
        // There is no BlockStateModel/MultiPartModel either: instantiate()
        // returns MultiVariant roots for simple definitions and MultiPart
        // roots for multipart ones, both UnbakedBlockStateModel.
        var definition=BlockModelDefinition.fromJsonElement(JsonParser.parseString(row.get("json").getAsString()));
        var roots=definition.instantiate(states,"horizon:binding_probe");
        var indices=new IdentityHashMap<MultiVariant,Integer>();
        var variants=(Map<String,MultiVariant>)field(definition,"variants");
        int index=0;for(var value:variants.values())indices.put(value,index++);
        var selected=new ArrayList<Object>();
        for(var state:states.getPossibleStates()) {
            UnbakedBlockStateModel root=roots.get(state);
            if(root==null) {selected.add(null);continue;}
            if(root instanceof MultiVariant variant) {
                var i=indices.get(variant);if(i==null)throw new AssertionError("unrecognized simple root");
                selected.add(map("variant",i));
            } else if(root instanceof MultiPart) {
                var group=root.visualEqualityGroup(state);Object parts=null;
                for(var f:group.getClass().getDeclaredFields())if(it.unimi.dsi.fastutil.ints.IntList.class.isAssignableFrom(f.getType())) {f.setAccessible(true);parts=f.get(group);}
                if(parts==null)throw new AssertionError("unrecognized multipart group");
                selected.add(map("multipart",parts));
            } else throw new AssertionError("unrecognized root "+root.getClass());
        }
        return selected;
    }
    public static void main(String[] args) throws Exception {
        net.minecraft.SharedConstants.tryDetectVersion();net.minecraft.server.Bootstrap.bootStrap();
        // 1.21.4 has no BlockStateDefinitions: the state definition comes
        // straight from the block registry.
        mapper=id->BuiltInRegistries.BLOCK.get(id).getStateDefinition();
        if(args[0].equals("export")){export(Path.of(args[1]));return;}
        try(var input=Files.newBufferedReader(Path.of(args[0]));var output=Files.newBufferedWriter(Path.of(args[1]))) {
            for(String line;(line=input.readLine())!=null;) {
                var row=JsonParser.parseString(line).getAsJsonObject();Object result;
                try {result=map("name",row.get("name").getAsString(),"selected",bind(row));}
                catch(RuntimeException error) {result=map("name",row.get("name").getAsString(),"error",true);}
                output.write(JSON.toJson(result));output.newLine();
            }
        }
    }
}
