package dev.horizonlab.parity;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.io.PrintWriter;
import java.lang.reflect.Field;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.resources.Identifier;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.tags.ItemTags;
import net.minecraft.world.flag.FeatureFlags;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Items;
import net.minecraft.world.item.alchemy.PotionBrewing;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.entity.BrewingStandBlockEntity;
import net.minecraft.world.level.block.state.BlockState;

/**
 * Oracle for 26.2's brewing: a real `PotionBrewing` over the vanilla
 * registries, the three brewing-slot filters, and `mix`. Fixture lines are
 * "input-snbt reagent-snbt candidate-snbt"; "tick" lines build a brewing
 * stand in a block-update world and call the real
 * `BrewingStandBlockEntity.serverTick`, printing the slots, brew time, fuel,
 * bottle bits, ingredient and brew-event count. 26.2 has no
 * `minecraft:brewing` recipes, no `BrewingRecipe`/`PotionIngredient` classes
 * and no brewing property sets — brewing is `PotionBrewing`'s hardcoded
 * mixes alone.
 */
public final class VanillaBrewingOracle {
    public static void main(String[] args) throws Exception {
        ParityData.init();
        VanillaBlockUpdateOracle.AIR = Blocks.AIR.defaultBlockState();
        var allocator = Class.forName("sun.misc.Unsafe");
        var unsafeField = allocator.getDeclaredField("theUnsafe");
        unsafeField.setAccessible(true);
        VanillaBlockUpdateOracle.unsafe = unsafeField.get(null);
        VanillaBlockUpdateOracle.allocate = allocator.getMethod("allocateInstance", Class.class);
        var brewing = PotionBrewing.bootstrap(FeatureFlags.VANILLA_SET);
        Path in = Path.of(args[0]), out = Path.of(args[1]);
        int n = 0;
        try (var w = new PrintWriter(Files.newBufferedWriter(out, StandardCharsets.UTF_8));
             var lines = Files.newBufferedReader(in, StandardCharsets.UTF_8)) {
            for (String line = lines.readLine(); line != null; line = lines.readLine()) {
                if (line.startsWith("#")) continue;
                String[] f = line.split(" ");
                if (f[0].equals("tick")) {
                    w.println(tick(n, f, brewing));
                    n++;
                    continue;
                }
                ItemStack input = ParityData.stack(f[0]);
                ItemStack reagent = ParityData.stack(f[1]);
                ItemStack candidate = ParityData.stack(f[2]);
                // BrewingStandBlockEntity.canPlaceItem, slots 0-2 (item part).
                boolean potionSlot = !candidate.isEmpty()
                        && (candidate.is(Items.POTION) || candidate.is(Items.SPLASH_POTION)
                            || candidate.is(Items.LINGERING_POTION) || candidate.is(Items.GLASS_BOTTLE));
                boolean reagentSlot = !candidate.isEmpty() && brewing.isIngredient(candidate);
                boolean fuelSlot = !candidate.isEmpty() && candidate.is(ItemTags.BREWING_FUEL);
                // PotionBrewing.mix returns the source itself when nothing
                // matches, and a fresh stack when a mix does.
                String result = "-";
                if (!input.isEmpty() && !reagent.isEmpty()) {
                    ItemStack mixed = brewing.mix(reagent, input);
                    if (mixed != input) result = ParityData.canon(mixed);
                }
                w.println(n + " " + potionSlot + " " + reagentSlot + " " + fuelSlot + " " + result);
                n++;
            }
        }
        System.exit(0);
    }

    private static Field field(Class<?> c, String name) throws Exception {
        Field f = c.getDeclaredField(name);
        f.setAccessible(true);
        return f;
    }

    private static void setInt(Object o, Class<?> c, String name, int v) throws Exception {
        field(c, name).setInt(o, v);
    }

    /** One brewing-stand tick run: initial slots/fields, `ticks` serverTick calls, final state. */
    private static String tick(int n, String[] f, PotionBrewing brewing) throws Exception {
        int ticks = Integer.parseInt(f[1]);
        ItemStack[] slots = new ItemStack[5];
        for (int i = 0; i < 5; i++) slots[i] = ParityData.stack(f[2 + i]);
        int brewTime = Integer.parseInt(f[7]);
        int fuel = Integer.parseInt(f[8]);
        String ingredient = f[9];

        JsonObject caseJson = new JsonObject();
        JsonArray cells = new JsonArray();
        JsonArray cell = new JsonArray();
        cell.add(0);
        cell.add(0);
        cell.add(0);
        cell.add(Block.getId(Blocks.BREWING_STAND.defaultBlockState()));
        cells.add(cell);
        caseJson.add("cells", cells);
        caseJson.addProperty("seed", 0);
        VanillaBlockUpdateOracle.World w = VanillaBlockUpdateOracle.build(caseJson);
        w.registries = ParityData.access;
        // serverTick reads level.potionBrewing(), which reads the server
        // level's own server field and the server's brewing field — both
        // null on the allocated instances, so wire them up.
        VanillaBlockUpdateOracle.set(ServerLevel.class, w, "server", w.server);
        VanillaBlockUpdateOracle.set(MinecraftServer.class, w.server, "potionBrewing", brewing);
        var pos = new BlockPos(0, 0, 0);
        BlockState state = w.getBlockState(pos);
        var be = (BrewingStandBlockEntity) w.getBlockEntity(pos);
        for (int i = 0; i < 5; i++) be.setItem(i, slots[i]);
        setInt(be, BrewingStandBlockEntity.class, "brewTime", brewTime);
        setInt(be, BrewingStandBlockEntity.class, "fuel", fuel);
        Item ingredientItem = ingredient.equals("-") ? null
                : BuiltInRegistries.ITEM.getValue(Identifier.parse(ingredient));
        field(BrewingStandBlockEntity.class, "ingredient").set(be, ingredientItem);
        for (int t = 0; t < ticks; t++) {
            BrewingStandBlockEntity.serverTick(w, pos, state, be);
        }
        int brews = 0;
        for (int at = w.log.indexOf("L 1035 "); at >= 0; at = w.log.indexOf("L 1035 ", at + 1)) brews++;
        StringBuilder sb = new StringBuilder(Integer.toString(n));
        for (int i = 0; i < 5; i++) sb.append(" s").append(i).append('=').append(ParityData.canon(be.getItem(i)));
        sb.append(" time=").append(field(BrewingStandBlockEntity.class, "brewTime").getInt(be))
          .append(" fuel=").append(field(BrewingStandBlockEntity.class, "fuel").getInt(be))
          .append(" bits=");
        for (int i = 0; i < 3; i++) sb.append(be.getItem(i).isEmpty() ? '0' : '1');
        Item ing = (Item) field(BrewingStandBlockEntity.class, "ingredient").get(be);
        sb.append(" ing=").append(ing == null ? "-" : BuiltInRegistries.ITEM.getKey(ing).toString())
          .append(" brews=").append(brews);
        return sb.toString();
    }
}
