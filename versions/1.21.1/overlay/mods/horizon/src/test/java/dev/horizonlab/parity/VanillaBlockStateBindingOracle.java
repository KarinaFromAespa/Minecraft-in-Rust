package dev.horizonlab.parity;

import com.google.gson.*;
import java.nio.file.*;
import java.util.*;
import net.minecraft.client.renderer.block.model.BlockModelDefinition;
import net.minecraft.client.renderer.block.model.MultiVariant;
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
    /** A variant key's partial property predicate, as BlockStateModelLoader parses it. */
    private static java.util.function.Predicate<BlockState> predicate(StateDefinition<Block,BlockState> definition,String key) {
        var values=new HashMap<Property<?>,Object>();
        for(var clause:key.split(",")) {
            var parts=clause.split("=",2);
            var property=definition.getProperty(parts[0]);
            if(property==null||parts.length<2) {
                if(parts[0].isEmpty())continue;
                throw new RuntimeException("Unknown variant property "+parts[0]);
            }
            Object value=property.getValue(parts[1]).orElse(null);
            if(value==null)throw new RuntimeException("Unknown variant value "+parts[1]);
            values.put(property,value);
        }
        return state->{for(var e:values.entrySet())if(!state.getValue((Property)e.getKey()).equals(e.getValue()))return false;return true;};
    }
    private static List<Integer> matching(List<java.util.function.Predicate<BlockState>> conditions,BlockState state) {
        var result=new ArrayList<Integer>();
        for(int i=0;i<conditions.size();i++)if(conditions.get(i).test(state))result.add(i);
        return result;
    }
    @SuppressWarnings("unchecked")
    private static Object bind(JsonObject row) throws Exception {
        var states=Objects.requireNonNull(mapper.apply(ResourceLocation.parse(row.get("block").getAsString())));
        // 1.21.1 has no instantiate()/UnbakedBlockStateModel/visualEqualityGroup:
        // parse with fromJsonElement (the Context carries the state definition,
        // as the loader sets it) and replay BlockStateModelLoader's selection:
        // multipart first (every state), then each variant key in order as a
        // partial property predicate. A variant overlapping an earlier variant
        // leaves that state on the missing model and aborts its walk.
        var context=new BlockModelDefinition.Context();context.setDefinition(states);
        var definition=BlockModelDefinition.fromJsonElement(context,JsonParser.parseString(row.get("json").getAsString()));
        var variants=definition.getVariants();
        var indices=new IdentityHashMap<MultiVariant,Integer>();
        int index=0;for(var value:variants.values())indices.put(value,index++);
        var possible=states.getPossibleStates();
        var selected=new ArrayList<Object>(Collections.nCopies(possible.size(),null));
        var fromVariant=new boolean[possible.size()];
        var multipart=definition.getMultiPart();
        if(multipart!=null) {
            var conditions=new ArrayList<java.util.function.Predicate<BlockState>>();
            for(var selector:multipart.getSelectors())conditions.add(selector.getPredicate(states));
            for(int i=0;i<possible.size();i++)selected.set(i,map("multipart",matching(conditions,possible.get(i))));
        }
        for(var entry:variants.entrySet()) {
            java.util.function.Predicate<BlockState> predicate;
            try {predicate=predicate(states,entry.getKey());}
            catch(RuntimeException ignored) {continue;} // the loader logs and skips the variant
            for(int i=0;i<possible.size();i++) {
                if(!predicate.test(possible.get(i)))continue;
                boolean wasVariant=fromVariant[i];
                selected.set(i,map("variant",indices.get(entry.getValue())));
                fromVariant[i]=true;
                if(wasVariant){selected.set(i,null);fromVariant[i]=false;break;}
            }
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
