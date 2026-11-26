package dev.horizonlab.parity;

import java.io.PrintWriter;
import java.lang.reflect.Field;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;
import net.minecraft.core.Direction;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.resources.Identifier;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.CampfireBlock;
import net.minecraft.world.level.block.RotatedPillarBlock;
import net.minecraft.world.level.block.WeatheringCopper;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.Property;

/**
 * Original oracle (26.2): the hardcoded axe/hoe/shovel tables of AxeItem,
 * HoeItem, ShovelItem and HoneycombItem. 26.2 has no block_transformer
 * registry. "dump" lists every block state; a case line "transformer TAB
 * face TAB clicked TAB above" prints the first transform that yields a state
 * and its settings, in the same field order as the Rust fixture.
 *
 * The tables are read from vanilla at runtime where they are reachable:
 * AxeItem.STRIPPABLES and ShovelItem.FLATTENABLES by reflection (protected
 * statics), WeatheringCopper.getPrevious and HoneycombItem.WAX_OFF_BY_BLOCK
 * by their public APIs. HoeItem.TILLABLES's consumers hide their target
 * states, so the hoe's five targets are hardcoded from the decompiled
 * source and the table's keys are checked against vanilla's at startup.
 */
public final class VanillaBlockTransformerOracle {
    static String describe(BlockState state) {
        StringBuilder out = new StringBuilder(BuiltInRegistries.BLOCK.getKey(state.getBlock()).toString()).append('|');
        List<Property<?>> props = new ArrayList<>(state.getProperties());
        props.sort(Comparator.comparing(Property::getName));
        boolean first = true;
        for (Property<?> p : props) {
            out.append(first ? "" : ",").append(p.getName()).append('=').append(name(state, p));
            first = false;
        }
        return out.toString();
    }

    @SuppressWarnings({"unchecked", "rawtypes"})
    static String name(BlockState state, Property p) {
        return p.getName(state.getValue(p));
    }

    @SuppressWarnings({"unchecked", "rawtypes"})
    static BlockState parse(String text) {
        String[] parts = text.split("\\|", -1);
        Block block = BuiltInRegistries.BLOCK.getValue(Identifier.parse(parts[0]));
        BlockState state = block.defaultBlockState();
        if (!parts[1].isEmpty()) {
            for (String kv : parts[1].split(",")) {
                String[] pair = kv.split("=");
                Property p = block.getStateDefinition().getProperty(pair[0]);
                state = state.setValue(p, (Comparable) p.getValue(pair[1]).orElseThrow());
            }
        }
        return state;
    }

    @SuppressWarnings("unchecked")
    static <K, V> Map<K, V> staticField(Class<?> owner, String name) throws Exception {
        Field f = owner.getDeclaredField(name);
        f.setAccessible(true);
        return (Map<K, V>) f.get(null);
    }

    /** One result line, in the fixture's field order. */
    static String result(int index, BlockState state, String sound, String particle, String loot, String type) {
        return index + "\t" + describe(state) + "\t" + sound + "\t" + particle + "\t" + loot
            + "\tfrom_middle\ttrue\t" + type + "\ttrue\t1";
    }

    public static void main(String[] args) throws Exception {
        ParityData.init();
        if (args[0].equals("dump")) {
            try (var out = new PrintWriter(Files.newBufferedWriter(Path.of(args[1]), StandardCharsets.UTF_8))) {
                for (Block block : BuiltInRegistries.BLOCK) {
                    for (BlockState state : block.getStateDefinition().getPossibleStates()) out.println(describe(state));
                }
            }
            return;
        }
        Map<Block, Block> strippables = staticField(net.minecraft.world.item.AxeItem.class, "STRIPPABLES");
        Map<Block, BlockState> flattenables = staticField(net.minecraft.world.item.ShovelItem.class, "FLATTENABLES");
        Map<Block, ?> tillables = staticField(net.minecraft.world.item.HoeItem.class, "TILLABLES");
        // HoeItem.TILLABLES's targets, from the decompiled source.
        Map<Block, BlockState> tillTargets = Map.of(
            Blocks.GRASS_BLOCK, Blocks.FARMLAND.defaultBlockState(),
            Blocks.DIRT_PATH, Blocks.FARMLAND.defaultBlockState(),
            Blocks.DIRT, Blocks.FARMLAND.defaultBlockState(),
            Blocks.COARSE_DIRT, Blocks.DIRT.defaultBlockState(),
            Blocks.ROOTED_DIRT, Blocks.DIRT.defaultBlockState());
        if (!tillables.keySet().equals(tillTargets.keySet())) {
            throw new IllegalStateException("HoeItem.TILLABLES keys changed: " + tillables.keySet());
        }
        try (var out = new PrintWriter(Files.newBufferedWriter(Path.of(args[1]), StandardCharsets.UTF_8));
             var lines = Files.lines(Path.of(args[0]))) {
            for (String line : (Iterable<String>) lines::iterator) {
                String[] f = line.split("\t", -1);
                try {
                    if (f[0].equals("honeycomb")) {
                        out.println(net.minecraft.world.item.HoneycombItem.getWaxed(parse(f[2])).map(VanillaBlockTransformerOracle::describe).orElse("none"));
                        continue;
                    }
                    Direction face = Direction.byName(f[1]);
                    BlockState clicked = parse(f[2]);
                    BlockState above = parse(f[3]);
                    String result = "none";
                    if (f[0].equals("axe")) {
                        // AxeItem.evaluateNewBlockState: strip, then scrape, then wax off.
                        Block stripped = strippables.get(clicked.getBlock());
                        if (stripped != null) {
                            BlockState next = stripped.defaultBlockState().setValue(RotatedPillarBlock.AXIS, clicked.getValue(RotatedPillarBlock.AXIS));
                            result = result(0, next, "minecraft:item.axe.strip", "none", "-", "single_block");
                        } else {
                            Optional<BlockState> scraped = WeatheringCopper.getPrevious(clicked);
                            if (scraped.isPresent()) {
                                result = result(1, scraped.get(), "minecraft:item.axe.scrape", "scrape", "-", "copper_chest");
                            } else {
                                Block waxOff = net.minecraft.world.item.HoneycombItem.WAX_OFF_BY_BLOCK.get().get(clicked.getBlock());
                                if (waxOff != null) {
                                    result = result(2, waxOff.withPropertiesOf(clicked), "minecraft:item.axe.wax_off", "wax_off", "-", "copper_chest");
                                }
                            }
                        }
                    } else if (f[0].equals("hoe")) {
                        // HoeItem.useOn: till needs face != down and air above; rooted dirt
                        // turns unconditionally and drops hanging roots.
                        if (clicked.getBlock() == Blocks.ROOTED_DIRT) {
                            result = result(1, Blocks.DIRT.defaultBlockState(), "minecraft:item.hoe.till", "none", "minecraft:till/rooted_dirt", "single_block");
                        } else if (face != Direction.DOWN && above.isAir()) {
                            BlockState till = tillTargets.get(clicked.getBlock());
                            if (till != null) {
                                result = result(0, till, "minecraft:item.hoe.till", "none", "-", "single_block");
                            }
                        }
                    } else if (f[0].equals("shovel")) {
                        // ShovelItem.useOn: face down passes; flatten needs air above;
                        // a lit campfire is dowsed (level event 1009, no direct sound).
                        if (face != Direction.DOWN) {
                            BlockState flat = flattenables.get(clicked.getBlock());
                            if (flat != null && above.isAir()) {
                                result = result(0, flat, "minecraft:item.shovel.flatten", "none", "-", "single_block");
                            } else if (clicked.getBlock() instanceof CampfireBlock && clicked.getValue(CampfireBlock.LIT)) {
                                result = result(1, clicked.setValue(CampfireBlock.LIT, false), "minecraft:empty", "dowse", "-", "single_block");
                            }
                        }
                    }
                    out.println(result);
                } catch (Throwable e) {
                    out.println("error " + e.getClass().getSimpleName() + " " + e.getMessage());
                }
            }
        }
    }
}
