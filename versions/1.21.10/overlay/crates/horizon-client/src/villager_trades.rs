//! The hardcoded 1.21.10 `VillagerTrades` tables and listing logic
//! (`net/minecraft/world/entity/npc/villager/VillagerTrades`).
//!
//! 26.1 moved villager trades into data (`trade_set` / `villager_trade`,
//! see `merchant::Trades`); 1.21.10 has no such registries and hardcodes
//! every trade in `VillagerTrades.TRADES` (13 professions x 5 levels) and
//! `VillagerTrades.WANDERING_TRADER_TRADES` (3 groups). The tables below
//! are transcribed from the decompiled 1.21.10 class; `Listing::offer`
//! ports each listing class's `getOffer`. `EXPERIMENTAL_TRADES` (behind
//! the `trade_rebalance` feature flag, off by default) is not included.
//!
//! Gaps against vanilla, noted where they bite: `TreasureMapForEmeralds`
//! skips `MapItem.renderBiomePreviewMap` (the data-driven exploration-map
//! path has no biome preview either); `EnchantBookForEmeralds` takes the
//! enchantment's minimum level as 1, which every 1.21.10 enchantment has
//! (the engine's `Enchantment` carries no minimum level).

use crate::{
    items::ItemStack,
    merchant::{ExplorationMap, ItemCost, MerchantOffer, TradeContext},
};
use horizon_core::random::RandomSource;
use horizon_world::nbt::Tag;

/// One `VillagerTrades.ItemListing`, with its constructor arguments.
#[derive(Clone, Copy, Debug)]
pub enum Listing {
    /// `DyedArmorForEmeralds`: a dyed leather piece for `value` emeralds.
    DyedArmor {
        item: &'static str,
        value: i32,
        max_uses: i32,
        xp: i32,
    },
    /// `EmeraldForItems`: `count` of an item for `emeralds` emeralds.
    /// `potion` is the wandering trader's water-bottle cost
    /// (`potionCost(Potions.WATER)`), a potion stack, not a bare item.
    EmeraldForItems {
        item: &'static str,
        count: i32,
        max_uses: i32,
        xp: i32,
        emeralds: i32,
        potion: Option<&'static str>,
    },
    /// `EmeraldsForVillagerTypeItem`: `cost` of the item for this villager
    /// type (a boat, per biome) for one emerald.
    EmeraldsForTypeItem {
        cost: i32,
        max_uses: i32,
        xp: i32,
        trades: &'static [(&'static str, &'static str)],
    },
    /// `EnchantBookForEmeralds`: a random enchanted book from `tag`,
    /// level clamped to `min_level..=max_level`, priced by level.
    EnchantBook {
        xp: i32,
        min_level: i32,
        max_level: i32,
        tag: &'static str,
    },
    /// `EnchantedItemForEmeralds`: an item enchanted at a random level
    /// 5..=19 from the `on_traded_equipment` tag.
    EnchantedItem {
        item: &'static str,
        base_cost: i32,
        max_uses: i32,
        xp: i32,
        mult: f32,
    },
    /// `FailureItemListing`: never an offer (unused by the tables).
    Failure,
    /// `ItemsAndEmeraldsToItems`: an item plus emeralds for an item.
    ItemsAndEmeralds {
        from_item: &'static str,
        from_count: i32,
        emerald_cost: i32,
        to_item: &'static str,
        to_count: i32,
        max_uses: i32,
        xp: i32,
        mult: f32,
    },
    /// `ItemsForEmeralds`: `count` of an item for emeralds. `potion` is
    /// the wandering trader's long-invisibility potion (`potion(...)`).
    ItemsForEmeralds {
        item: &'static str,
        emerald_cost: i32,
        count: i32,
        max_uses: i32,
        xp: i32,
        mult: f32,
        potion: Option<&'static str>,
    },
    /// `SuspiciousStewForEmerald`: a stew with one effect, for 1 emerald.
    /// `duration` is in ticks, as the Java constructor takes it.
    SuspiciousStew {
        effect: &'static str,
        duration: i32,
        xp: i32,
        mult: f32,
    },
    /// `TippedArrowForItemsAndEmeralds`: arrows plus emeralds for tipped
    /// arrows with a random brewable potion.
    TippedArrow {
        from_item: &'static str,
        from_count: i32,
        to_item: &'static str,
        to_count: i32,
        emerald_cost: i32,
        max_uses: i32,
        xp: i32,
    },
    /// `TreasureMapForEmeralds`: an explorer map to the nearest structure
    /// in `destination`, plus a compass, for emeralds. Null when none is
    /// found within 100 chunks.
    TreasureMap {
        emerald_cost: i32,
        destination: &'static str,
        name: &'static str,
        decoration: &'static str,
        max_uses: i32,
        xp: i32,
    },
    /// `TypeSpecificTrade.oneTradeInBiomes`: `listing`, only for a
    /// villager whose type is in `types`.
    TypeSpecific {
        types: &'static [&'static str],
        listing: &'static Listing,
    },
}

/// `TippedArrowForItemsAndEmeralds`'s pool: the potion registry in order,
/// keeping potions with effects that are the output of a brewing mix
/// (`PotionBrewing.isBrewablePotion`). Water, mundane, thick and awkward
/// have no effects; luck has an effect but no brewing recipe.
const TIPPED_ARROW_POTIONS: &[&str] = &[
    "minecraft:night_vision",
    "minecraft:long_night_vision",
    "minecraft:invisibility",
    "minecraft:long_invisibility",
    "minecraft:leaping",
    "minecraft:long_leaping",
    "minecraft:strong_leaping",
    "minecraft:fire_resistance",
    "minecraft:long_fire_resistance",
    "minecraft:swiftness",
    "minecraft:long_swiftness",
    "minecraft:strong_swiftness",
    "minecraft:slowness",
    "minecraft:long_slowness",
    "minecraft:strong_slowness",
    "minecraft:turtle_master",
    "minecraft:long_turtle_master",
    "minecraft:strong_turtle_master",
    "minecraft:water_breathing",
    "minecraft:long_water_breathing",
    "minecraft:healing",
    "minecraft:strong_healing",
    "minecraft:harming",
    "minecraft:strong_harming",
    "minecraft:poison",
    "minecraft:long_poison",
    "minecraft:strong_poison",
    "minecraft:regeneration",
    "minecraft:long_regeneration",
    "minecraft:strong_regeneration",
    "minecraft:strength",
    "minecraft:long_strength",
    "minecraft:strong_strength",
    "minecraft:weakness",
    "minecraft:long_weakness",
    "minecraft:slow_falling",
    "minecraft:long_slow_falling",
    "minecraft:wind_charged",
    "minecraft:weaving",
    "minecraft:oozing",
    "minecraft:infested",
];

fn emeralds(count: i32) -> ItemStack {
    ItemStack::new("minecraft:emerald", count.max(0) as u32)
}

fn offer(
    a: ItemStack,
    b: Option<ItemStack>,
    result: ItemStack,
    max_uses: i32,
    xp: i32,
    mult: f32,
) -> MerchantOffer {
    MerchantOffer {
        base_cost_a: ItemCost { stack: a },
        cost_b: b.map(|stack| ItemCost { stack }),
        result,
        uses: 0,
        max_uses,
        reward_exp: true,
        special_price_diff: 0,
        demand: 0,
        price_multiplier: mult,
        xp,
    }
}

/// `EnchantBookForEmeralds`'s price: 2 + a roll of 5 + 10 per level +
/// 3 per level, doubled for `double_trade_price` enchantments, capped
/// at a stack.
fn book_price<R: RandomSource>(random: &mut R, level: i32, doubled: bool) -> i32 {
    let mut price = 2 + random.next_int_bound(5 + level * 10).unwrap_or(0) + 3 * level;
    if doubled {
        price *= 2;
    }
    price.min(64)
}

impl Listing {
    /// `ItemListing.getOffer`: the offer, or None when the listing has
    /// none for this merchant (wrong villager type, no map structure,
    /// no map maker). Draws from the merchant's own random, as vanilla
    /// does (`this.random`, not a random sequence).
    pub fn offer<R: RandomSource>(
        &self,
        ctx: &TradeContext<'_>,
        random: &mut R,
    ) -> Option<MerchantOffer> {
        match *self {
            Listing::DyedArmor {
                item,
                value,
                max_uses,
                xp,
            } => {
                let mut result = ItemStack::new(item, 1);
                // `DyedItemColor.applyDyes` with 1-3 random dyes, only
                // when the item is dyeable (`ItemTags.DYEABLE`).
                let dyeable = ctx
                    .items
                    .item_tags
                    .get("minecraft:dyeable")
                    .is_some_and(|t| t.contains(item));
                if dyeable {
                    let mut dyes = vec![random.next_int_bound(16).unwrap_or(0) as usize];
                    if random.next_float() > 0.7 {
                        dyes.push(random.next_int_bound(16).unwrap_or(0) as usize);
                    }
                    if random.next_float() > 0.8 {
                        dyes.push(random.next_int_bound(16).unwrap_or(0) as usize);
                    }
                    let current = result.data().and_then(|d| d.dyed_color);
                    result.data_mut().dyed_color =
                        Some(crate::crafting::apply_dyes(current, &dyes));
                }
                Some(offer(emeralds(value), None, result, max_uses, xp, 0.2))
            }
            Listing::EmeraldForItems {
                item,
                count,
                max_uses,
                xp,
                emeralds: n,
                potion,
            } => {
                let mut cost = ItemStack::new(item, count.max(0) as u32);
                if let Some(potion) = potion {
                    cost = cost.with_potion(potion);
                }
                Some(offer(
                    cost,
                    None,
                    ItemStack::new("minecraft:emerald", n.max(0) as u32),
                    max_uses,
                    xp,
                    0.05,
                ))
            }
            Listing::EmeraldsForTypeItem {
                cost,
                max_uses,
                xp,
                trades,
            } => {
                let item = trades
                    .iter()
                    .find(|(t, _)| *t == ctx.variant)
                    .map(|(_, item)| *item)?;
                Some(offer(
                    ItemStack::new(item, cost.max(0) as u32),
                    None,
                    ItemStack::new("minecraft:emerald", 1),
                    max_uses,
                    xp,
                    0.05,
                ))
            }
            Listing::EnchantBook {
                xp,
                min_level,
                max_level,
                tag,
            } => {
                let pool = ctx.enchantments.tag(tag);
                let pick = if pool.is_empty() {
                    None
                } else {
                    let i = random.next_int_bound(pool.len() as i32).unwrap_or(0) as usize;
                    Some(pool[i].clone())
                };
                let (result, price) = match pick {
                    Some(id) => {
                        // Every 1.21.10 enchantment's minimum level is 1.
                        let max = ctx
                            .enchantments
                            .get(&id)
                            .map_or(1, |d| d.max_level as i32)
                            .min(max_level);
                        let level = random
                            .next_int_between_inclusive(min_level.max(1), max)
                            .unwrap_or(min_level.max(1));
                        let mut book = ItemStack::new("minecraft:enchanted_book", 1);
                        book.enchant(&id, level as u32);
                        let doubled = ctx
                            .enchantments
                            .in_tag("minecraft:double_trade_price", &id);
                        (book, book_price(random, level, doubled))
                    }
                    // No tradeable enchantment: a plain book for 1 emerald.
                    None => (ItemStack::new("minecraft:book", 1), 1),
                };
                Some(offer(
                    emeralds(price),
                    Some(ItemStack::new("minecraft:book", 1)),
                    result,
                    12,
                    xp,
                    0.2,
                ))
            }
            Listing::EnchantedItem {
                item,
                base_cost,
                max_uses,
                xp,
                mult,
            } => {
                let level = 5 + random.next_int_bound(15).unwrap_or(0);
                let pool = ctx.enchantments.tag("minecraft:on_traded_equipment");
                let chosen = ctx
                    .enchantments
                    .select(ctx.items, random, item, level, pool);
                let mut result = ItemStack::new(item, 1);
                for (id, l) in chosen {
                    result.enchant(&id, l);
                }
                Some(offer(
                    emeralds((base_cost + level).min(64)),
                    None,
                    result,
                    max_uses,
                    xp,
                    mult,
                ))
            }
            Listing::Failure => None,
            Listing::ItemsAndEmeralds {
                from_item,
                from_count,
                emerald_cost,
                to_item,
                to_count,
                max_uses,
                xp,
                mult,
            } => Some(offer(
                emeralds(emerald_cost),
                Some(ItemStack::new(from_item, from_count.max(0) as u32)),
                ItemStack::new(to_item, to_count.max(0) as u32),
                max_uses,
                xp,
                mult,
            )),
            Listing::ItemsForEmeralds {
                item,
                emerald_cost,
                count,
                max_uses,
                xp,
                mult,
                potion,
            } => {
                let mut result = ItemStack::new(item, count.max(0) as u32);
                if let Some(potion) = potion {
                    result = result.with_potion(potion);
                }
                Some(offer(emeralds(emerald_cost), None, result, max_uses, xp, mult))
            }
            Listing::SuspiciousStew {
                effect,
                duration,
                xp,
                mult,
            } => {
                let mut result = ItemStack::new("minecraft:suspicious_stew", 1);
                result
                    .data_mut()
                    .stew_effects
                    .push((std::sync::Arc::from(effect), duration));
                Some(offer(emeralds(1), None, result, 12, xp, mult))
            }
            Listing::TippedArrow {
                from_item,
                from_count,
                to_item,
                to_count,
                emerald_cost,
                max_uses,
                xp,
            } => {
                let pick = TIPPED_ARROW_POTIONS
                    [random.next_int_bound(TIPPED_ARROW_POTIONS.len() as i32).unwrap_or(0) as usize];
                let result = ItemStack::new(to_item, to_count.max(0) as u32).with_potion(pick);
                Some(offer(
                    emeralds(emerald_cost),
                    Some(ItemStack::new(from_item, from_count.max(0) as u32)),
                    result,
                    max_uses,
                    xp,
                    0.05,
                ))
            }
            Listing::TreasureMap {
                emerald_cost,
                destination,
                name,
                decoration,
                max_uses,
                xp,
            } => {
                let maps = ctx.maps?;
                // `findNearestMapStructure(tag, pos, 100, true)`: a scale-2
                // map, skipping known structures. No structure found leaves
                // the stack without a map id, and the listing has no offer.
                let map = ExplorationMap {
                    destination: destination.to_owned(),
                    decoration: decoration.to_owned(),
                    zoom: 2,
                    search_radius: 100,
                    skip_existing_chunks: true,
                };
                let mut result = maps.exploration_map(
                    ItemStack::new("minecraft:filled_map", 1),
                    &map,
                    ctx.origin,
                );
                if result.data().and_then(|d| d.map_id).is_none() {
                    return None;
                }
                result.data_mut().set_other(
                    "minecraft:item_name",
                    Some(Tag::compound([(
                        "translate",
                        Tag::String(name.into()),
                    )])),
                );
                Some(offer(
                    emeralds(emerald_cost),
                    Some(ItemStack::new("minecraft:compass", 1)),
                    result,
                    max_uses,
                    xp,
                    0.2,
                ))
            }
            Listing::TypeSpecific { types, listing } => {
                if types.contains(&ctx.variant) {
                    listing.offer(ctx, random)
                } else {
                    None
                }
            }
        }
    }
}

/// `AbstractVillager.addOffersFromItemListings`: draw `count` offers from
/// `listings` without replacement; a listing with no offer does not
/// count toward the draw.
pub fn offers_from_listings<R: RandomSource>(
    listings: &[Listing],
    count: i32,
    ctx: &TradeContext<'_>,
    random: &mut R,
) -> Vec<MerchantOffer> {
    let mut pool: Vec<&Listing> = listings.iter().collect();
    let mut out = Vec::new();
    let mut added = 0;
    while added < count && !pool.is_empty() {
        let roll = random.next_int_bound(pool.len() as i32).unwrap_or(0) as usize;
        let listing = pool.remove(roll);
        if let Some(o) = listing.offer(ctx, random) {
            out.push(o);
            added += 1;
        }
    }
    out
}

/// `Villager.updateTrades`: 2 offers from the profession's table for
/// `level` (1-5). An unknown profession or level has no offers.
pub fn offers_for<R: RandomSource>(
    profession: &str,
    level: i32,
    ctx: &TradeContext<'_>,
    random: &mut R,
) -> Vec<MerchantOffer> {
    let Some((_, levels)) = TRADES.iter().find(|(p, _)| *p == profession) else {
        return Vec::new();
    };
    let Some((_, listings)) = levels.iter().find(|(l, _)| *l == level) else {
        return Vec::new();
    };
    offers_from_listings(listings, 2, ctx, random)
}

/// `WanderingTrader.updateTrades`: each pair's draw from its listings
/// (2 buying, 2 from the first group, 5 from the second).
pub fn wandering_trader_offers<R: RandomSource>(
    ctx: &TradeContext<'_>,
    random: &mut R,
) -> Vec<MerchantOffer> {
    let mut out = Vec::new();
    for (listings, count) in WANDERING_TRADER_TRADES {
        out.extend(offers_from_listings(listings, *count, ctx, random));
    }
    out
}

// The hardcoded 1.21.10 trade tables, transcribed from the decompiled
// `VillagerTrades` (server-game.jar) by /tmp/vt/parse_trades.py.
// `EXPERIMENTAL_TRADES` (behind the trade_rebalance feature flag) is not
// included: the flag is off by default and the engine has no feature flags.

static TYPE_SPECIFIC_0: Listing = Listing::TreasureMap { emerald_cost: 8, destination: "minecraft:on_taiga_village_maps", name: "filled_map.village_taiga", decoration: "minecraft:village_taiga", max_uses: 12, xp: 5 };
static TYPE_SPECIFIC_1: Listing = Listing::TreasureMap { emerald_cost: 8, destination: "minecraft:on_swamp_explorer_maps", name: "filled_map.explorer_swamp", decoration: "minecraft:swamp_hut", max_uses: 12, xp: 5 };
static TYPE_SPECIFIC_2: Listing = Listing::TreasureMap { emerald_cost: 8, destination: "minecraft:on_snowy_village_maps", name: "filled_map.village_snowy", decoration: "minecraft:village_snowy", max_uses: 12, xp: 5 };
static TYPE_SPECIFIC_3: Listing = Listing::TreasureMap { emerald_cost: 8, destination: "minecraft:on_savanna_village_maps", name: "filled_map.village_savanna", decoration: "minecraft:village_savanna", max_uses: 12, xp: 5 };
static TYPE_SPECIFIC_4: Listing = Listing::TreasureMap { emerald_cost: 8, destination: "minecraft:on_plains_village_maps", name: "filled_map.village_plains", decoration: "minecraft:village_plains", max_uses: 12, xp: 5 };
static TYPE_SPECIFIC_5: Listing = Listing::TreasureMap { emerald_cost: 8, destination: "minecraft:on_jungle_explorer_maps", name: "filled_map.explorer_jungle", decoration: "minecraft:jungle_temple", max_uses: 12, xp: 5 };
static TYPE_SPECIFIC_6: Listing = Listing::TreasureMap { emerald_cost: 8, destination: "minecraft:on_desert_village_maps", name: "filled_map.village_desert", decoration: "minecraft:village_desert", max_uses: 12, xp: 5 };
static TYPE_SPECIFIC_7: Listing = Listing::ItemsForEmeralds { item: "minecraft:blue_banner", emerald_cost: 2, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None };
static TYPE_SPECIFIC_8: Listing = Listing::ItemsForEmeralds { item: "minecraft:white_banner", emerald_cost: 2, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None };
static TYPE_SPECIFIC_9: Listing = Listing::ItemsForEmeralds { item: "minecraft:red_banner", emerald_cost: 2, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None };
static TYPE_SPECIFIC_10: Listing = Listing::ItemsForEmeralds { item: "minecraft:green_banner", emerald_cost: 2, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None };
static TYPE_SPECIFIC_11: Listing = Listing::ItemsForEmeralds { item: "minecraft:lime_banner", emerald_cost: 2, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None };
static TYPE_SPECIFIC_12: Listing = Listing::ItemsForEmeralds { item: "minecraft:purple_banner", emerald_cost: 2, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None };
static TYPE_SPECIFIC_13: Listing = Listing::ItemsForEmeralds { item: "minecraft:cyan_banner", emerald_cost: 2, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None };
static TYPE_SPECIFIC_14: Listing = Listing::ItemsForEmeralds { item: "minecraft:yellow_banner", emerald_cost: 2, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None };
static TYPE_SPECIFIC_15: Listing = Listing::ItemsForEmeralds { item: "minecraft:orange_banner", emerald_cost: 2, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None };
static TYPE_SPECIFIC_16: Listing = Listing::ItemsForEmeralds { item: "minecraft:brown_banner", emerald_cost: 2, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None };
static TYPE_SPECIFIC_17: Listing = Listing::ItemsForEmeralds { item: "minecraft:magenta_banner", emerald_cost: 2, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None };
static TYPE_SPECIFIC_18: Listing = Listing::ItemsForEmeralds { item: "minecraft:light_blue_banner", emerald_cost: 2, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None };
static TYPE_SPECIFIC_19: Listing = Listing::ItemsForEmeralds { item: "minecraft:pink_banner", emerald_cost: 2, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None };
static TYPE_SPECIFIC_20: Listing = Listing::ItemsForEmeralds { item: "minecraft:gray_banner", emerald_cost: 2, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None };
static TYPE_SPECIFIC_21: Listing = Listing::ItemsForEmeralds { item: "minecraft:black_banner", emerald_cost: 2, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None };

/// `VillagerTrades.TRADES`: profession -> (level, listings).
pub static TRADES: &[(&str, &[(i32, &[Listing])])] = &[
    ("minecraft:farmer", &[
        (1, &[
            Listing::EmeraldForItems { item: "minecraft:wheat", count: 20, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:potato", count: 26, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:carrot", count: 22, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:beetroot", count: 15, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:bread", emerald_cost: 1, count: 6, max_uses: 16, xp: 1, mult: 0.05, potion: None },
        ]),
        (2, &[
            Listing::EmeraldForItems { item: "minecraft:pumpkin", count: 6, max_uses: 12, xp: 10, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:pumpkin_pie", emerald_cost: 1, count: 4, max_uses: 12, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:apple", emerald_cost: 1, count: 4, max_uses: 16, xp: 5, mult: 0.05, potion: None },
        ]),
        (3, &[
            Listing::ItemsForEmeralds { item: "minecraft:cookie", emerald_cost: 3, count: 18, max_uses: 12, xp: 10, mult: 0.05, potion: None },
            Listing::EmeraldForItems { item: "minecraft:melon", count: 4, max_uses: 12, xp: 20, emeralds: 1, potion: None },
        ]),
        (4, &[
            Listing::ItemsForEmeralds { item: "minecraft:cake", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::SuspiciousStew { effect: "minecraft:night_vision", duration: 100, xp: 15, mult: 0.05 },
            Listing::SuspiciousStew { effect: "minecraft:jump_boost", duration: 160, xp: 15, mult: 0.05 },
            Listing::SuspiciousStew { effect: "minecraft:weakness", duration: 140, xp: 15, mult: 0.05 },
            Listing::SuspiciousStew { effect: "minecraft:blindness", duration: 120, xp: 15, mult: 0.05 },
            Listing::SuspiciousStew { effect: "minecraft:poison", duration: 280, xp: 15, mult: 0.05 },
            Listing::SuspiciousStew { effect: "minecraft:saturation", duration: 7, xp: 15, mult: 0.05 },
        ]),
        (5, &[
            Listing::ItemsForEmeralds { item: "minecraft:golden_carrot", emerald_cost: 3, count: 3, max_uses: 12, xp: 30, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:glistering_melon_slice", emerald_cost: 4, count: 3, max_uses: 12, xp: 30, mult: 0.05, potion: None },
        ]),
    ]),
    ("minecraft:fisherman", &[
        (1, &[
            Listing::EmeraldForItems { item: "minecraft:string", count: 20, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:coal", count: 10, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::ItemsAndEmeralds { from_item: "minecraft:cod", from_count: 6, emerald_cost: 1, to_item: "minecraft:cooked_cod", to_count: 6, max_uses: 16, xp: 1, mult: 0.05 },
            Listing::ItemsForEmeralds { item: "minecraft:cod_bucket", emerald_cost: 3, count: 1, max_uses: 16, xp: 1, mult: 0.05, potion: None },
        ]),
        (2, &[
            Listing::EmeraldForItems { item: "minecraft:cod", count: 15, max_uses: 16, xp: 10, emeralds: 1, potion: None },
            Listing::ItemsAndEmeralds { from_item: "minecraft:salmon", from_count: 6, emerald_cost: 1, to_item: "minecraft:cooked_salmon", to_count: 6, max_uses: 16, xp: 5, mult: 0.05 },
            Listing::ItemsForEmeralds { item: "minecraft:campfire", emerald_cost: 2, count: 1, max_uses: 12, xp: 5, mult: 0.05, potion: None },
        ]),
        (3, &[
            Listing::EmeraldForItems { item: "minecraft:salmon", count: 13, max_uses: 16, xp: 20, emeralds: 1, potion: None },
            Listing::EnchantedItem { item: "minecraft:fishing_rod", base_cost: 3, max_uses: 3, xp: 10, mult: 0.2 },
        ]),
        (4, &[
            Listing::EmeraldForItems { item: "minecraft:tropical_fish", count: 6, max_uses: 12, xp: 30, emeralds: 1, potion: None },
        ]),
        (5, &[
            Listing::EmeraldForItems { item: "minecraft:pufferfish", count: 4, max_uses: 12, xp: 30, emeralds: 1, potion: None },
            Listing::EmeraldsForTypeItem { cost: 1, max_uses: 12, xp: 30, trades: &[("minecraft:plains", "minecraft:oak_boat"), ("minecraft:taiga", "minecraft:spruce_boat"), ("minecraft:snow", "minecraft:spruce_boat"), ("minecraft:desert", "minecraft:jungle_boat"), ("minecraft:jungle", "minecraft:jungle_boat"), ("minecraft:savanna", "minecraft:acacia_boat"), ("minecraft:swamp", "minecraft:dark_oak_boat")] },
        ]),
    ]),
    ("minecraft:shepherd", &[
        (1, &[
            Listing::EmeraldForItems { item: "minecraft:white_wool", count: 18, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:brown_wool", count: 18, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:black_wool", count: 18, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:gray_wool", count: 18, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:shears", emerald_cost: 2, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        ]),
        (2, &[
            Listing::EmeraldForItems { item: "minecraft:white_dye", count: 12, max_uses: 16, xp: 10, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:gray_dye", count: 12, max_uses: 16, xp: 10, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:black_dye", count: 12, max_uses: 16, xp: 10, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:light_blue_dye", count: 12, max_uses: 16, xp: 10, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:lime_dye", count: 12, max_uses: 16, xp: 10, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:white_wool", emerald_cost: 1, count: 1, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:orange_wool", emerald_cost: 1, count: 1, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:magenta_wool", emerald_cost: 1, count: 1, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:light_blue_wool", emerald_cost: 1, count: 1, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:yellow_wool", emerald_cost: 1, count: 1, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:lime_wool", emerald_cost: 1, count: 1, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:pink_wool", emerald_cost: 1, count: 1, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:gray_wool", emerald_cost: 1, count: 1, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:light_gray_wool", emerald_cost: 1, count: 1, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:cyan_wool", emerald_cost: 1, count: 1, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:purple_wool", emerald_cost: 1, count: 1, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:blue_wool", emerald_cost: 1, count: 1, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:brown_wool", emerald_cost: 1, count: 1, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:green_wool", emerald_cost: 1, count: 1, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:red_wool", emerald_cost: 1, count: 1, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:black_wool", emerald_cost: 1, count: 1, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:white_carpet", emerald_cost: 1, count: 4, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:orange_carpet", emerald_cost: 1, count: 4, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:magenta_carpet", emerald_cost: 1, count: 4, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:light_blue_carpet", emerald_cost: 1, count: 4, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:yellow_carpet", emerald_cost: 1, count: 4, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:lime_carpet", emerald_cost: 1, count: 4, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:pink_carpet", emerald_cost: 1, count: 4, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:gray_carpet", emerald_cost: 1, count: 4, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:light_gray_carpet", emerald_cost: 1, count: 4, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:cyan_carpet", emerald_cost: 1, count: 4, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:purple_carpet", emerald_cost: 1, count: 4, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:blue_carpet", emerald_cost: 1, count: 4, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:brown_carpet", emerald_cost: 1, count: 4, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:green_carpet", emerald_cost: 1, count: 4, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:red_carpet", emerald_cost: 1, count: 4, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:black_carpet", emerald_cost: 1, count: 4, max_uses: 16, xp: 5, mult: 0.05, potion: None },
        ]),
        (3, &[
            Listing::EmeraldForItems { item: "minecraft:yellow_dye", count: 12, max_uses: 16, xp: 20, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:light_gray_dye", count: 12, max_uses: 16, xp: 20, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:orange_dye", count: 12, max_uses: 16, xp: 20, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:red_dye", count: 12, max_uses: 16, xp: 20, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:pink_dye", count: 12, max_uses: 16, xp: 20, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:white_bed", emerald_cost: 3, count: 1, max_uses: 12, xp: 10, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:yellow_bed", emerald_cost: 3, count: 1, max_uses: 12, xp: 10, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:red_bed", emerald_cost: 3, count: 1, max_uses: 12, xp: 10, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:black_bed", emerald_cost: 3, count: 1, max_uses: 12, xp: 10, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:blue_bed", emerald_cost: 3, count: 1, max_uses: 12, xp: 10, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:brown_bed", emerald_cost: 3, count: 1, max_uses: 12, xp: 10, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:cyan_bed", emerald_cost: 3, count: 1, max_uses: 12, xp: 10, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:gray_bed", emerald_cost: 3, count: 1, max_uses: 12, xp: 10, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:green_bed", emerald_cost: 3, count: 1, max_uses: 12, xp: 10, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:light_blue_bed", emerald_cost: 3, count: 1, max_uses: 12, xp: 10, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:light_gray_bed", emerald_cost: 3, count: 1, max_uses: 12, xp: 10, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:lime_bed", emerald_cost: 3, count: 1, max_uses: 12, xp: 10, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:magenta_bed", emerald_cost: 3, count: 1, max_uses: 12, xp: 10, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:orange_bed", emerald_cost: 3, count: 1, max_uses: 12, xp: 10, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:pink_bed", emerald_cost: 3, count: 1, max_uses: 12, xp: 10, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:purple_bed", emerald_cost: 3, count: 1, max_uses: 12, xp: 10, mult: 0.05, potion: None },
        ]),
        (4, &[
            Listing::EmeraldForItems { item: "minecraft:brown_dye", count: 12, max_uses: 16, xp: 30, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:purple_dye", count: 12, max_uses: 16, xp: 30, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:blue_dye", count: 12, max_uses: 16, xp: 30, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:green_dye", count: 12, max_uses: 16, xp: 30, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:magenta_dye", count: 12, max_uses: 16, xp: 30, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:cyan_dye", count: 12, max_uses: 16, xp: 30, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:white_banner", emerald_cost: 3, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:blue_banner", emerald_cost: 3, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:light_blue_banner", emerald_cost: 3, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:red_banner", emerald_cost: 3, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:pink_banner", emerald_cost: 3, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:green_banner", emerald_cost: 3, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:lime_banner", emerald_cost: 3, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:gray_banner", emerald_cost: 3, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:black_banner", emerald_cost: 3, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:purple_banner", emerald_cost: 3, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:magenta_banner", emerald_cost: 3, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:cyan_banner", emerald_cost: 3, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:brown_banner", emerald_cost: 3, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:yellow_banner", emerald_cost: 3, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:orange_banner", emerald_cost: 3, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:light_gray_banner", emerald_cost: 3, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
        ]),
        (5, &[
            Listing::ItemsForEmeralds { item: "minecraft:painting", emerald_cost: 2, count: 3, max_uses: 12, xp: 30, mult: 0.05, potion: None },
        ]),
    ]),
    ("minecraft:fletcher", &[
        (1, &[
            Listing::EmeraldForItems { item: "minecraft:stick", count: 32, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:arrow", emerald_cost: 1, count: 16, max_uses: 12, xp: 1, mult: 0.05, potion: None },
            Listing::ItemsAndEmeralds { from_item: "minecraft:gravel", from_count: 10, emerald_cost: 1, to_item: "minecraft:flint", to_count: 10, max_uses: 12, xp: 1, mult: 0.05 },
        ]),
        (2, &[
            Listing::EmeraldForItems { item: "minecraft:flint", count: 26, max_uses: 12, xp: 10, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:bow", emerald_cost: 2, count: 1, max_uses: 12, xp: 5, mult: 0.05, potion: None },
        ]),
        (3, &[
            Listing::EmeraldForItems { item: "minecraft:string", count: 14, max_uses: 16, xp: 20, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:crossbow", emerald_cost: 3, count: 1, max_uses: 12, xp: 10, mult: 0.05, potion: None },
        ]),
        (4, &[
            Listing::EmeraldForItems { item: "minecraft:feather", count: 24, max_uses: 16, xp: 30, emeralds: 1, potion: None },
            Listing::EnchantedItem { item: "minecraft:bow", base_cost: 2, max_uses: 3, xp: 15, mult: 0.05 },
        ]),
        (5, &[
            Listing::EmeraldForItems { item: "minecraft:tripwire_hook", count: 8, max_uses: 12, xp: 30, emeralds: 1, potion: None },
            Listing::EnchantedItem { item: "minecraft:crossbow", base_cost: 3, max_uses: 3, xp: 15, mult: 0.05 },
            Listing::TippedArrow { from_item: "minecraft:arrow", from_count: 5, to_item: "minecraft:tipped_arrow", to_count: 5, emerald_cost: 2, max_uses: 12, xp: 30 },
        ]),
    ]),
    ("minecraft:librarian", &[
        (1, &[
            Listing::EmeraldForItems { item: "minecraft:paper", count: 24, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::EnchantBook { xp: 1, min_level: 0, max_level: i32::MAX, tag: "minecraft:tradeable" },
            Listing::ItemsForEmeralds { item: "minecraft:bookshelf", emerald_cost: 9, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        ]),
        (2, &[
            Listing::EmeraldForItems { item: "minecraft:book", count: 4, max_uses: 12, xp: 10, emeralds: 1, potion: None },
            Listing::EnchantBook { xp: 5, min_level: 0, max_level: i32::MAX, tag: "minecraft:tradeable" },
            Listing::ItemsForEmeralds { item: "minecraft:lantern", emerald_cost: 1, count: 1, max_uses: 12, xp: 5, mult: 0.05, potion: None },
        ]),
        (3, &[
            Listing::EmeraldForItems { item: "minecraft:ink_sac", count: 5, max_uses: 12, xp: 20, emeralds: 1, potion: None },
            Listing::EnchantBook { xp: 10, min_level: 0, max_level: i32::MAX, tag: "minecraft:tradeable" },
            Listing::ItemsForEmeralds { item: "minecraft:glass", emerald_cost: 1, count: 4, max_uses: 12, xp: 10, mult: 0.05, potion: None },
        ]),
        (4, &[
            Listing::EmeraldForItems { item: "minecraft:writable_book", count: 2, max_uses: 12, xp: 30, emeralds: 1, potion: None },
            Listing::EnchantBook { xp: 15, min_level: 0, max_level: i32::MAX, tag: "minecraft:tradeable" },
            Listing::ItemsForEmeralds { item: "minecraft:clock", emerald_cost: 5, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:compass", emerald_cost: 4, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
        ]),
        (5, &[
            Listing::ItemsForEmeralds { item: "minecraft:name_tag", emerald_cost: 20, count: 1, max_uses: 12, xp: 30, mult: 0.05, potion: None },
        ]),
    ]),
    ("minecraft:cartographer", &[
        (1, &[
            Listing::EmeraldForItems { item: "minecraft:paper", count: 24, max_uses: 12, xp: 2, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:map", emerald_cost: 7, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        ]),
        (2, &[
            Listing::EmeraldForItems { item: "minecraft:glass_pane", count: 11, max_uses: 12, xp: 10, emeralds: 1, potion: None },
            Listing::TypeSpecific { types: &["minecraft:swamp", "minecraft:snow", "minecraft:plains"], listing: &TYPE_SPECIFIC_0 },
            Listing::TypeSpecific { types: &["minecraft:taiga", "minecraft:snow", "minecraft:jungle"], listing: &TYPE_SPECIFIC_1 },
            Listing::TypeSpecific { types: &["minecraft:taiga", "minecraft:swamp"], listing: &TYPE_SPECIFIC_2 },
            Listing::TypeSpecific { types: &["minecraft:plains", "minecraft:jungle", "minecraft:desert"], listing: &TYPE_SPECIFIC_3 },
            Listing::TypeSpecific { types: &["minecraft:taiga", "minecraft:snow", "minecraft:savanna", "minecraft:desert"], listing: &TYPE_SPECIFIC_4 },
            Listing::TypeSpecific { types: &["minecraft:swamp", "minecraft:savanna", "minecraft:desert"], listing: &TYPE_SPECIFIC_5 },
            Listing::TypeSpecific { types: &["minecraft:savanna", "minecraft:jungle"], listing: &TYPE_SPECIFIC_6 },
        ]),
        (3, &[
            Listing::EmeraldForItems { item: "minecraft:compass", count: 1, max_uses: 12, xp: 20, emeralds: 1, potion: None },
            Listing::TreasureMap { emerald_cost: 13, destination: "minecraft:on_ocean_explorer_maps", name: "filled_map.monument", decoration: "minecraft:monument", max_uses: 12, xp: 10 },
            Listing::TreasureMap { emerald_cost: 12, destination: "minecraft:on_trial_chambers_maps", name: "filled_map.trial_chambers", decoration: "minecraft:trial_chambers", max_uses: 12, xp: 10 },
        ]),
        (4, &[
            Listing::ItemsForEmeralds { item: "minecraft:item_frame", emerald_cost: 7, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::TypeSpecific { types: &["minecraft:snow", "minecraft:taiga"], listing: &TYPE_SPECIFIC_7 },
            Listing::TypeSpecific { types: &["minecraft:snow", "minecraft:plains"], listing: &TYPE_SPECIFIC_8 },
            Listing::TypeSpecific { types: &["minecraft:snow", "minecraft:savanna"], listing: &TYPE_SPECIFIC_9 },
            Listing::TypeSpecific { types: &["minecraft:desert", "minecraft:savanna", "minecraft:jungle"], listing: &TYPE_SPECIFIC_10 },
            Listing::TypeSpecific { types: &["minecraft:desert", "minecraft:taiga"], listing: &TYPE_SPECIFIC_11 },
            Listing::TypeSpecific { types: &["minecraft:taiga", "minecraft:swamp"], listing: &TYPE_SPECIFIC_12 },
            Listing::TypeSpecific { types: &["minecraft:desert", "minecraft:snow"], listing: &TYPE_SPECIFIC_13 },
            Listing::TypeSpecific { types: &["minecraft:plains", "minecraft:jungle"], listing: &TYPE_SPECIFIC_14 },
            Listing::TypeSpecific { types: &["minecraft:savanna", "minecraft:desert"], listing: &TYPE_SPECIFIC_15 },
            Listing::TypeSpecific { types: &["minecraft:plains", "minecraft:jungle"], listing: &TYPE_SPECIFIC_16 },
            Listing::TypeSpecific { types: &["minecraft:savanna"], listing: &TYPE_SPECIFIC_17 },
            Listing::TypeSpecific { types: &["minecraft:snow", "minecraft:swamp"], listing: &TYPE_SPECIFIC_18 },
            Listing::TypeSpecific { types: &["minecraft:taiga", "minecraft:plains"], listing: &TYPE_SPECIFIC_19 },
            Listing::TypeSpecific { types: &["minecraft:desert"], listing: &TYPE_SPECIFIC_20 },
            Listing::TypeSpecific { types: &["minecraft:swamp"], listing: &TYPE_SPECIFIC_21 },
        ]),
        (5, &[
            Listing::ItemsForEmeralds { item: "minecraft:globe_banner_pattern", emerald_cost: 8, count: 1, max_uses: 12, xp: 30, mult: 0.05, potion: None },
            Listing::TreasureMap { emerald_cost: 14, destination: "minecraft:on_woodland_explorer_maps", name: "filled_map.mansion", decoration: "minecraft:mansion", max_uses: 12, xp: 30 },
        ]),
    ]),
    ("minecraft:cleric", &[
        (1, &[
            Listing::EmeraldForItems { item: "minecraft:rotten_flesh", count: 32, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:redstone", emerald_cost: 1, count: 2, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        ]),
        (2, &[
            Listing::EmeraldForItems { item: "minecraft:gold_ingot", count: 3, max_uses: 12, xp: 10, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:lapis_lazuli", emerald_cost: 1, count: 1, max_uses: 12, xp: 5, mult: 0.05, potion: None },
        ]),
        (3, &[
            Listing::EmeraldForItems { item: "minecraft:rabbit_foot", count: 2, max_uses: 12, xp: 20, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:glowstone", emerald_cost: 4, count: 1, max_uses: 12, xp: 10, mult: 0.05, potion: None },
        ]),
        (4, &[
            Listing::EmeraldForItems { item: "minecraft:turtle_scute", count: 4, max_uses: 12, xp: 30, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:glass_bottle", count: 9, max_uses: 12, xp: 30, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:ender_pearl", emerald_cost: 5, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
        ]),
        (5, &[
            Listing::EmeraldForItems { item: "minecraft:nether_wart", count: 22, max_uses: 12, xp: 30, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:experience_bottle", emerald_cost: 3, count: 1, max_uses: 12, xp: 30, mult: 0.05, potion: None },
        ]),
    ]),
    ("minecraft:armorer", &[
        (1, &[
            Listing::EmeraldForItems { item: "minecraft:coal", count: 15, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:iron_leggings", emerald_cost: 7, count: 1, max_uses: 12, xp: 1, mult: 0.2, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:iron_boots", emerald_cost: 4, count: 1, max_uses: 12, xp: 1, mult: 0.2, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:iron_helmet", emerald_cost: 5, count: 1, max_uses: 12, xp: 1, mult: 0.2, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:iron_chestplate", emerald_cost: 9, count: 1, max_uses: 12, xp: 1, mult: 0.2, potion: None },
        ]),
        (2, &[
            Listing::EmeraldForItems { item: "minecraft:iron_ingot", count: 4, max_uses: 12, xp: 10, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:bell", emerald_cost: 36, count: 1, max_uses: 12, xp: 5, mult: 0.2, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:chainmail_boots", emerald_cost: 1, count: 1, max_uses: 12, xp: 5, mult: 0.2, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:chainmail_leggings", emerald_cost: 3, count: 1, max_uses: 12, xp: 5, mult: 0.2, potion: None },
        ]),
        (3, &[
            Listing::EmeraldForItems { item: "minecraft:lava_bucket", count: 1, max_uses: 12, xp: 20, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:diamond", count: 1, max_uses: 12, xp: 20, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:chainmail_helmet", emerald_cost: 1, count: 1, max_uses: 12, xp: 10, mult: 0.2, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:chainmail_chestplate", emerald_cost: 4, count: 1, max_uses: 12, xp: 10, mult: 0.2, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:shield", emerald_cost: 5, count: 1, max_uses: 12, xp: 10, mult: 0.2, potion: None },
        ]),
        (4, &[
            Listing::EnchantedItem { item: "minecraft:diamond_leggings", base_cost: 14, max_uses: 3, xp: 15, mult: 0.2 },
            Listing::EnchantedItem { item: "minecraft:diamond_boots", base_cost: 8, max_uses: 3, xp: 15, mult: 0.2 },
        ]),
        (5, &[
            Listing::EnchantedItem { item: "minecraft:diamond_helmet", base_cost: 8, max_uses: 3, xp: 30, mult: 0.2 },
            Listing::EnchantedItem { item: "minecraft:diamond_chestplate", base_cost: 16, max_uses: 3, xp: 30, mult: 0.2 },
        ]),
    ]),
    ("minecraft:weaponsmith", &[
        (1, &[
            Listing::EmeraldForItems { item: "minecraft:coal", count: 15, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:iron_axe", emerald_cost: 3, count: 1, max_uses: 12, xp: 1, mult: 0.2, potion: None },
            Listing::EnchantedItem { item: "minecraft:iron_sword", base_cost: 2, max_uses: 3, xp: 1, mult: 0.05 },
        ]),
        (2, &[
            Listing::EmeraldForItems { item: "minecraft:iron_ingot", count: 4, max_uses: 12, xp: 10, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:bell", emerald_cost: 36, count: 1, max_uses: 12, xp: 5, mult: 0.2, potion: None },
        ]),
        (3, &[
            Listing::EmeraldForItems { item: "minecraft:flint", count: 24, max_uses: 12, xp: 20, emeralds: 1, potion: None },
        ]),
        (4, &[
            Listing::EmeraldForItems { item: "minecraft:diamond", count: 1, max_uses: 12, xp: 30, emeralds: 1, potion: None },
            Listing::EnchantedItem { item: "minecraft:diamond_axe", base_cost: 12, max_uses: 3, xp: 15, mult: 0.2 },
        ]),
        (5, &[
            Listing::EnchantedItem { item: "minecraft:diamond_sword", base_cost: 8, max_uses: 3, xp: 30, mult: 0.2 },
        ]),
    ]),
    ("minecraft:toolsmith", &[
        (1, &[
            Listing::EmeraldForItems { item: "minecraft:coal", count: 15, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:stone_axe", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.2, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:stone_shovel", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.2, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:stone_pickaxe", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.2, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:stone_hoe", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.2, potion: None },
        ]),
        (2, &[
            Listing::EmeraldForItems { item: "minecraft:iron_ingot", count: 4, max_uses: 12, xp: 10, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:bell", emerald_cost: 36, count: 1, max_uses: 12, xp: 5, mult: 0.2, potion: None },
        ]),
        (3, &[
            Listing::EmeraldForItems { item: "minecraft:flint", count: 30, max_uses: 12, xp: 20, emeralds: 1, potion: None },
            Listing::EnchantedItem { item: "minecraft:iron_axe", base_cost: 1, max_uses: 3, xp: 10, mult: 0.2 },
            Listing::EnchantedItem { item: "minecraft:iron_shovel", base_cost: 2, max_uses: 3, xp: 10, mult: 0.2 },
            Listing::EnchantedItem { item: "minecraft:iron_pickaxe", base_cost: 3, max_uses: 3, xp: 10, mult: 0.2 },
            Listing::ItemsForEmeralds { item: "minecraft:diamond_hoe", emerald_cost: 4, count: 1, max_uses: 3, xp: 10, mult: 0.2, potion: None },
        ]),
        (4, &[
            Listing::EmeraldForItems { item: "minecraft:diamond", count: 1, max_uses: 12, xp: 30, emeralds: 1, potion: None },
            Listing::EnchantedItem { item: "minecraft:diamond_axe", base_cost: 12, max_uses: 3, xp: 15, mult: 0.2 },
            Listing::EnchantedItem { item: "minecraft:diamond_shovel", base_cost: 5, max_uses: 3, xp: 15, mult: 0.2 },
        ]),
        (5, &[
            Listing::EnchantedItem { item: "minecraft:diamond_pickaxe", base_cost: 13, max_uses: 3, xp: 30, mult: 0.2 },
        ]),
    ]),
    ("minecraft:butcher", &[
        (1, &[
            Listing::EmeraldForItems { item: "minecraft:chicken", count: 14, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:porkchop", count: 7, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:rabbit", count: 4, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:rabbit_stew", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        ]),
        (2, &[
            Listing::EmeraldForItems { item: "minecraft:coal", count: 15, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:cooked_porkchop", emerald_cost: 1, count: 5, max_uses: 16, xp: 5, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:cooked_chicken", emerald_cost: 1, count: 8, max_uses: 16, xp: 5, mult: 0.05, potion: None },
        ]),
        (3, &[
            Listing::EmeraldForItems { item: "minecraft:mutton", count: 7, max_uses: 16, xp: 20, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:beef", count: 10, max_uses: 16, xp: 20, emeralds: 1, potion: None },
        ]),
        (4, &[
            Listing::EmeraldForItems { item: "minecraft:dried_kelp_block", count: 10, max_uses: 12, xp: 30, emeralds: 1, potion: None },
        ]),
        (5, &[
            Listing::EmeraldForItems { item: "minecraft:sweet_berries", count: 10, max_uses: 12, xp: 30, emeralds: 1, potion: None },
        ]),
    ]),
    ("minecraft:leatherworker", &[
        (1, &[
            Listing::EmeraldForItems { item: "minecraft:leather", count: 6, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::DyedArmor { item: "minecraft:leather_leggings", value: 3, max_uses: 12, xp: 1 },
            Listing::DyedArmor { item: "minecraft:leather_chestplate", value: 7, max_uses: 12, xp: 1 },
        ]),
        (2, &[
            Listing::EmeraldForItems { item: "minecraft:flint", count: 26, max_uses: 12, xp: 10, emeralds: 1, potion: None },
            Listing::DyedArmor { item: "minecraft:leather_helmet", value: 5, max_uses: 12, xp: 5 },
            Listing::DyedArmor { item: "minecraft:leather_boots", value: 4, max_uses: 12, xp: 5 },
        ]),
        (3, &[
            Listing::EmeraldForItems { item: "minecraft:rabbit_hide", count: 9, max_uses: 12, xp: 20, emeralds: 1, potion: None },
            Listing::DyedArmor { item: "minecraft:leather_chestplate", value: 7, max_uses: 12, xp: 1 },
        ]),
        (4, &[
            Listing::EmeraldForItems { item: "minecraft:turtle_scute", count: 4, max_uses: 12, xp: 30, emeralds: 1, potion: None },
            Listing::DyedArmor { item: "minecraft:leather_horse_armor", value: 6, max_uses: 12, xp: 15 },
        ]),
        (5, &[
            Listing::ItemsForEmeralds { item: "minecraft:saddle", emerald_cost: 6, count: 1, max_uses: 12, xp: 30, mult: 0.2, potion: None },
            Listing::DyedArmor { item: "minecraft:leather_helmet", value: 5, max_uses: 12, xp: 30 },
        ]),
    ]),
    ("minecraft:mason", &[
        (1, &[
            Listing::EmeraldForItems { item: "minecraft:clay_ball", count: 10, max_uses: 16, xp: 2, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:brick", emerald_cost: 1, count: 10, max_uses: 16, xp: 1, mult: 0.05, potion: None },
        ]),
        (2, &[
            Listing::EmeraldForItems { item: "minecraft:stone", count: 20, max_uses: 16, xp: 10, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:chiseled_stone_bricks", emerald_cost: 1, count: 4, max_uses: 16, xp: 5, mult: 0.05, potion: None },
        ]),
        (3, &[
            Listing::EmeraldForItems { item: "minecraft:granite", count: 16, max_uses: 16, xp: 20, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:andesite", count: 16, max_uses: 16, xp: 20, emeralds: 1, potion: None },
            Listing::EmeraldForItems { item: "minecraft:diorite", count: 16, max_uses: 16, xp: 20, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:dripstone_block", emerald_cost: 1, count: 4, max_uses: 16, xp: 10, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:polished_andesite", emerald_cost: 1, count: 4, max_uses: 16, xp: 10, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:polished_diorite", emerald_cost: 1, count: 4, max_uses: 16, xp: 10, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:polished_granite", emerald_cost: 1, count: 4, max_uses: 16, xp: 10, mult: 0.05, potion: None },
        ]),
        (4, &[
            Listing::EmeraldForItems { item: "minecraft:quartz", count: 12, max_uses: 12, xp: 30, emeralds: 1, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:orange_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:white_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:blue_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:light_blue_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:gray_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:light_gray_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:black_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:red_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:pink_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:magenta_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:lime_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:green_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:cyan_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:purple_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:yellow_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:brown_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:orange_glazed_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:white_glazed_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:blue_glazed_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:light_blue_glazed_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:gray_glazed_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:light_gray_glazed_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:black_glazed_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:red_glazed_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:pink_glazed_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:magenta_glazed_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:lime_glazed_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:green_glazed_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:cyan_glazed_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:purple_glazed_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:yellow_glazed_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:brown_glazed_terracotta", emerald_cost: 1, count: 1, max_uses: 12, xp: 15, mult: 0.05, potion: None },
        ]),
        (5, &[
            Listing::ItemsForEmeralds { item: "minecraft:quartz_pillar", emerald_cost: 1, count: 1, max_uses: 12, xp: 30, mult: 0.05, potion: None },
            Listing::ItemsForEmeralds { item: "minecraft:quartz_block", emerald_cost: 1, count: 1, max_uses: 12, xp: 30, mult: 0.05, potion: None },
        ]),
    ]),
];

/// `VillagerTrades.WANDERING_TRADER_TRADES`: (listings, how many to draw).
pub static WANDERING_TRADER_TRADES: &[(&[Listing], i32)] = &[
    (&[
        Listing::EmeraldForItems { item: "minecraft:potion", count: 1, max_uses: 2, xp: 1, emeralds: 1, potion: Some("minecraft:water") },
        Listing::EmeraldForItems { item: "minecraft:water_bucket", count: 1, max_uses: 2, xp: 1, emeralds: 2, potion: None },
        Listing::EmeraldForItems { item: "minecraft:milk_bucket", count: 1, max_uses: 2, xp: 1, emeralds: 2, potion: None },
        Listing::EmeraldForItems { item: "minecraft:fermented_spider_eye", count: 1, max_uses: 2, xp: 1, emeralds: 3, potion: None },
        Listing::EmeraldForItems { item: "minecraft:baked_potato", count: 4, max_uses: 2, xp: 1, emeralds: 1, potion: None },
        Listing::EmeraldForItems { item: "minecraft:hay_block", count: 1, max_uses: 2, xp: 1, emeralds: 1, potion: None },
    ], 2),
    (&[
        Listing::ItemsForEmeralds { item: "minecraft:packed_ice", emerald_cost: 1, count: 1, max_uses: 6, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:blue_ice", emerald_cost: 6, count: 1, max_uses: 6, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:gunpowder", emerald_cost: 1, count: 4, max_uses: 2, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:podzol", emerald_cost: 3, count: 3, max_uses: 6, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:acacia_log", emerald_cost: 1, count: 8, max_uses: 4, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:birch_log", emerald_cost: 1, count: 8, max_uses: 4, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:dark_oak_log", emerald_cost: 1, count: 8, max_uses: 4, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:jungle_log", emerald_cost: 1, count: 8, max_uses: 4, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:oak_log", emerald_cost: 1, count: 8, max_uses: 4, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:spruce_log", emerald_cost: 1, count: 8, max_uses: 4, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:cherry_log", emerald_cost: 1, count: 8, max_uses: 4, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:mangrove_log", emerald_cost: 1, count: 8, max_uses: 4, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:pale_oak_log", emerald_cost: 1, count: 8, max_uses: 4, xp: 1, mult: 0.05, potion: None },
        Listing::EnchantedItem { item: "minecraft:iron_pickaxe", base_cost: 1, max_uses: 1, xp: 1, mult: 0.2 },
        Listing::ItemsForEmeralds { item: "minecraft:potion", emerald_cost: 5, count: 1, max_uses: 1, xp: 1, mult: 0.05, potion: Some("minecraft:long_invisibility") },
    ], 2),
    (&[
        Listing::ItemsForEmeralds { item: "minecraft:tropical_fish_bucket", emerald_cost: 3, count: 1, max_uses: 4, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:pufferfish_bucket", emerald_cost: 3, count: 1, max_uses: 4, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:sea_pickle", emerald_cost: 2, count: 1, max_uses: 5, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:slime_ball", emerald_cost: 4, count: 1, max_uses: 5, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:glowstone", emerald_cost: 2, count: 1, max_uses: 5, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:nautilus_shell", emerald_cost: 5, count: 1, max_uses: 5, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:fern", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:sugar_cane", emerald_cost: 1, count: 1, max_uses: 8, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:pumpkin", emerald_cost: 1, count: 1, max_uses: 4, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:kelp", emerald_cost: 3, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:cactus", emerald_cost: 3, count: 1, max_uses: 8, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:dandelion", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:poppy", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:blue_orchid", emerald_cost: 1, count: 1, max_uses: 8, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:allium", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:azure_bluet", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:red_tulip", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:orange_tulip", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:white_tulip", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:pink_tulip", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:oxeye_daisy", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:cornflower", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:lily_of_the_valley", emerald_cost: 1, count: 1, max_uses: 7, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:open_eyeblossom", emerald_cost: 1, count: 1, max_uses: 7, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:wheat_seeds", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:beetroot_seeds", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:pumpkin_seeds", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:melon_seeds", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:acacia_sapling", emerald_cost: 5, count: 1, max_uses: 8, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:birch_sapling", emerald_cost: 5, count: 1, max_uses: 8, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:dark_oak_sapling", emerald_cost: 5, count: 1, max_uses: 8, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:jungle_sapling", emerald_cost: 5, count: 1, max_uses: 8, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:oak_sapling", emerald_cost: 5, count: 1, max_uses: 8, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:spruce_sapling", emerald_cost: 5, count: 1, max_uses: 8, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:cherry_sapling", emerald_cost: 5, count: 1, max_uses: 8, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:pale_oak_sapling", emerald_cost: 5, count: 1, max_uses: 8, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:mangrove_propagule", emerald_cost: 5, count: 1, max_uses: 8, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:red_dye", emerald_cost: 1, count: 3, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:white_dye", emerald_cost: 1, count: 3, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:blue_dye", emerald_cost: 1, count: 3, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:pink_dye", emerald_cost: 1, count: 3, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:black_dye", emerald_cost: 1, count: 3, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:green_dye", emerald_cost: 1, count: 3, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:light_gray_dye", emerald_cost: 1, count: 3, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:magenta_dye", emerald_cost: 1, count: 3, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:yellow_dye", emerald_cost: 1, count: 3, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:gray_dye", emerald_cost: 1, count: 3, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:purple_dye", emerald_cost: 1, count: 3, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:light_blue_dye", emerald_cost: 1, count: 3, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:lime_dye", emerald_cost: 1, count: 3, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:orange_dye", emerald_cost: 1, count: 3, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:brown_dye", emerald_cost: 1, count: 3, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:cyan_dye", emerald_cost: 1, count: 3, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:brain_coral_block", emerald_cost: 3, count: 1, max_uses: 8, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:bubble_coral_block", emerald_cost: 3, count: 1, max_uses: 8, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:fire_coral_block", emerald_cost: 3, count: 1, max_uses: 8, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:horn_coral_block", emerald_cost: 3, count: 1, max_uses: 8, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:tube_coral_block", emerald_cost: 3, count: 1, max_uses: 8, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:vine", emerald_cost: 1, count: 3, max_uses: 4, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:pale_hanging_moss", emerald_cost: 1, count: 3, max_uses: 4, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:brown_mushroom", emerald_cost: 1, count: 3, max_uses: 4, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:red_mushroom", emerald_cost: 1, count: 3, max_uses: 4, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:lily_pad", emerald_cost: 1, count: 5, max_uses: 2, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:small_dripleaf", emerald_cost: 1, count: 2, max_uses: 5, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:sand", emerald_cost: 1, count: 8, max_uses: 8, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:red_sand", emerald_cost: 1, count: 4, max_uses: 6, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:pointed_dripstone", emerald_cost: 1, count: 2, max_uses: 5, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:rooted_dirt", emerald_cost: 1, count: 2, max_uses: 5, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:moss_block", emerald_cost: 1, count: 2, max_uses: 5, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:pale_moss_block", emerald_cost: 1, count: 2, max_uses: 5, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:wildflowers", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:dry_tall_grass", emerald_cost: 1, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
        Listing::ItemsForEmeralds { item: "minecraft:firefly_bush", emerald_cost: 3, count: 1, max_uses: 12, xp: 1, mult: 0.05, potion: None },
    ], 5),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::Items;
    use crate::merchant::TradeContext;
    use horizon_core::random::LegacyRandom;
    use horizon_items::enchant::Enchantments;
    use std::collections::{HashMap, HashSet};

    fn ctx<'a>(items: &'a Items, enchantments: &'a Enchantments) -> TradeContext<'a> {
        TradeContext {
            items,
            enchantments,
            variant: "minecraft:plains",
            origin: [0.0; 3],
            maps: None,
        }
    }

    fn items() -> Items {
        Items::for_tests(HashMap::new())
    }

    #[test]
    fn a_farmer_gets_two_level_one_offers() {
        let items = items();
        let e = Enchantments::default();
        let ctx = ctx(&items, &e);
        let mut random = LegacyRandom::new(42);
        let offers = offers_for("minecraft:farmer", 1, &ctx, &mut random);
        assert_eq!(offers.len(), 2);
        // Every level-1 farmer offer costs wheat/potato/carrot/beetroot
        // or 1 emerald for bread.
        for o in &offers {
            assert!(o.max_uses == 16);
            assert!(o.xp == 1 || o.xp == 2);
        }
    }

    #[test]
    fn a_wandering_trader_gets_nine_offers() {
        let items = items();
        let e = Enchantments::default();
        let ctx = ctx(&items, &e);
        let mut random = LegacyRandom::new(7);
        let offers = wandering_trader_offers(&ctx, &mut random);
        assert_eq!(offers.len(), 9);
    }

    #[test]
    fn unknown_professions_and_levels_have_no_offers() {
        let items = items();
        let e = Enchantments::default();
        let ctx = ctx(&items, &e);
        let mut random = LegacyRandom::new(1);
        assert!(offers_for("minecraft:nitwit", 1, &ctx, &mut random).is_empty());
        assert!(offers_for("minecraft:farmer", 6, &ctx, &mut random).is_empty());
    }

    #[test]
    fn dyed_armor_comes_out_dyed() {
        let mut tags = HashMap::new();
        tags.insert(
            "minecraft:dyeable".to_string(),
            HashSet::from(["minecraft:leather_chestplate".to_string()]),
        );
        let items = Items::for_tests(tags);
        let e = Enchantments::default();
        let ctx = ctx(&items, &e);
        let mut random = LegacyRandom::new(3);
        let listing = Listing::DyedArmor {
            item: "minecraft:leather_chestplate",
            value: 7,
            max_uses: 12,
            xp: 1,
        };
        let o = listing.offer(&ctx, &mut random).expect("an offer");
        assert_eq!(o.base_cost_a.stack.count, 7);
        assert!(o.result.data().and_then(|d| d.dyed_color).is_some());
    }

    #[test]
    fn a_book_trade_without_enchantments_is_a_plain_book_for_one() {
        let items = items();
        let e = Enchantments::default();
        let ctx = ctx(&items, &e);
        let mut random = LegacyRandom::new(3);
        let listing = Listing::EnchantBook {
            xp: 5,
            min_level: 0,
            max_level: i32::MAX,
            tag: "minecraft:tradeable",
        };
        let o = listing.offer(&ctx, &mut random).expect("an offer");
        assert_eq!(&*o.result.item, "minecraft:book");
        assert_eq!(o.base_cost_a.stack.count, 1);
        assert_eq!(o.cost_b.as_ref().map(|c| c.stack.count), Some(1));
    }

    #[test]
    fn book_price_doubles_and_caps() {
        let mut random = LegacyRandom::new(0);
        // Level 1, undoubled: 2 + [0, 15) + 3, so 5..=19.
        let p = book_price(&mut random, 1, false);
        assert!((5..=19).contains(&p), "{p}");
        // Level 5 doubled would exceed a stack without the cap.
        let mut high = LegacyRandom::new(0);
        let mut seen_high = false;
        for _ in 0..200 {
            if book_price(&mut high, 5, true) == 64 {
                seen_high = true;
            }
        }
        assert!(seen_high);
    }

    #[test]
    fn a_type_specific_trade_only_fires_for_its_types() {
        static INNER: Listing = Listing::ItemsForEmeralds {
            item: "minecraft:map",
            emerald_cost: 7,
            count: 1,
            max_uses: 12,
            xp: 1,
            mult: 0.05,
            potion: None,
        };
        let items = items();
        let e = Enchantments::default();
        let listing = Listing::TypeSpecific {
            types: &["minecraft:desert"],
            listing: &INNER,
        };
        let mut random = LegacyRandom::new(3);
        let mut ctx = ctx(&items, &e);
        assert!(listing.offer(&ctx, &mut random).is_none());
        ctx.variant = "minecraft:desert";
        assert!(listing.offer(&ctx, &mut random).is_some());
    }

    #[test]
    fn a_treasure_map_without_a_map_maker_has_no_offer() {
        let items = items();
        let e = Enchantments::default();
        let ctx = ctx(&items, &e);
        let mut random = LegacyRandom::new(3);
        let listing = Listing::TreasureMap {
            emerald_cost: 13,
            destination: "minecraft:on_ocean_explorer_maps",
            name: "filled_map.monument",
            decoration: "minecraft:monument",
            max_uses: 12,
            xp: 10,
        };
        assert!(listing.offer(&ctx, &mut random).is_none());
    }

    #[test]
    fn a_failure_listing_has_no_offer() {
        let items = items();
        let e = Enchantments::default();
        let ctx = ctx(&items, &e);
        let mut random = LegacyRandom::new(3);
        assert!(Listing::Failure.offer(&ctx, &mut random).is_none());
    }

    #[test]
    fn suspicious_stew_carries_its_effect_in_ticks() {
        let items = items();
        let e = Enchantments::default();
        let ctx = ctx(&items, &e);
        let mut random = LegacyRandom::new(3);
        let listing = Listing::SuspiciousStew {
            effect: "minecraft:night_vision",
            duration: 100,
            xp: 15,
            mult: 0.05,
        };
        let o = listing.offer(&ctx, &mut random).expect("an offer");
        assert_eq!(&*o.result.item, "minecraft:suspicious_stew");
        let effects = &o.result.data().expect("data").stew_effects;
        assert_eq!(effects.len(), 1);
        assert_eq!(&*effects[0].0, "minecraft:night_vision");
        assert_eq!(effects[0].1, 100);
    }
}
