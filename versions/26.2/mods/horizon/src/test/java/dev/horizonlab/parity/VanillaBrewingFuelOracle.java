package dev.horizonlab.parity;

import com.google.gson.*;
import java.io.PrintWriter;
import java.nio.file.*;
import java.util.List;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.resources.Identifier;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.flag.FeatureFlags;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.alchemy.PotionBrewing;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.entity.BrewingStandBlockEntity;
import net.minecraft.world.level.block.state.BlockState;

/**
 * Calls native fuel acceptance (the `minecraft:brewing_fuel` tag) and two
 * production refuel ticks without a server. 26.2 has no `brewing_fuel`
 * component: fuel is any item in the tag, 20 uses, and the brew time is a
 * flat 400 ticks.
 */
public final class VanillaBrewingFuelOracle {
    static PotionBrewing brewing;
    static final class World extends ServerLevel {
        World() { super(null, null, null, null, null, null, false, 0L, List.of(), false); }
        @Override public void blockEntityChanged(BlockPos pos) {}
        @Override public void updateNeighbourForOutputSignal(BlockPos pos, Block block) {}
        // serverTick reads level.potionBrewing(); the allocated server is
        // null, so answer the vanilla brewing directly.
        @Override public PotionBrewing potionBrewing() { return brewing; }
    }
    static Object field(Object object, String name) throws Exception {
        var field = BrewingStandBlockEntity.class.getDeclaredField(name);
        field.setAccessible(true); return field.get(object);
    }
    public static void main(String[] args) throws Exception {
        ParityData.init();
        brewing = PotionBrewing.bootstrap(FeatureFlags.VANILLA_SET);
        var unsafeClass = Class.forName("sun.misc.Unsafe");
        var unsafeField = unsafeClass.getDeclaredField("theUnsafe"); unsafeField.setAccessible(true);
        var world = (World) unsafeClass.getMethod("allocateInstance", Class.class).invoke(unsafeField.get(null), World.class);
        var slot = Class.forName("net.minecraft.world.inventory.BrewingStandMenu$FuelSlot").getDeclaredMethod("mayPlaceItem", ItemStack.class);
        slot.setAccessible(true);
        try (var out = new PrintWriter(Files.newBufferedWriter(Path.of(args[1])))) {
            for (String line : Files.readAllLines(Path.of(args[0]))) {
                var input = JsonParser.parseString(line).getAsJsonObject();
                var stack = new ItemStack(BuiltInRegistries.ITEM.getValue(Identifier.parse(input.get("item").getAsString())), input.get("count").getAsInt());
                var stand = new BrewingStandBlockEntity(BlockPos.ZERO, Blocks.BREWING_STAND.defaultBlockState());
                boolean accepted = stand.canPlaceItem(4, stack);
                if (accepted != (boolean) slot.invoke(null, stack) || accepted != stand.canPlaceItemThroughFace(4, stack, net.minecraft.core.Direction.NORTH)) throw new AssertionError("fuel acceptance diverged");
                stand.setItem(4, stack);
                var bottles = BrewingStandBlockEntity.class.getDeclaredField("lastPotionCount"); bottles.setAccessible(true); bottles.set(stand, new boolean[3]);
                for (int tick = 0; tick < 2; tick++) {
                    BrewingStandBlockEntity.serverTick(world, BlockPos.ZERO, Blocks.BREWING_STAND.defaultBlockState(), stand);
                    out.println(accepted + " " + field(stand, "fuel") + " " + stand.getItem(4).getCount());
                }
            }
        }
    }
}
