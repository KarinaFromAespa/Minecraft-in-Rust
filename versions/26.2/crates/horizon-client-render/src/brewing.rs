//! Potions and the brewing stand: the `Potions` / `MobEffects` registries (code
//! in vanilla, ported as tables), 26.2's hardcoded `PotionBrewing` mixes
//! (26.2 has no `minecraft:brewing` recipe files — `BrewingRecipe` is a
//! 26.3-only class), potion colours and tooltips, and
//! `BrewingStandBlockEntity.serverTick`.
pub use crate::chunk::block_entity::BrewingStand;
use crate::items::{ItemStack, Items};
use horizon_world::loot::PackOverlay;
use std::{collections::HashSet, path::Path};

pub use horizon_client_core::potions::{
    BASE_POTION_COLOR, Category, EFFECTS, EffectDef, POTIONS, PotionDef, all_effects,
    description_id, effect, is_mod_effect, mod_effects, namespaced, potion, potion_color,
    register_mod_effects, set_pack_colors,
};
/// `MobEffectUtil.formatDuration` at 20 TPS.
pub fn format_duration(ticks: i32) -> String {
    format_tick_duration(ticks, 20.0)
}
/// `StringUtil.formatTickDuration`: mm:ss, or hh:mm:ss from an hour up, every
/// field two digits.
pub fn format_tick_duration(ticks: i32, tick_rate: f32) -> String {
    let secs = (ticks as f32 / tick_rate).floor() as i32;
    let (h, m, s) = (secs / 3600, secs / 60 % 60, secs % 60);
    if h > 0 {
        format!("{h:02}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}
/// `item.minecraft.potion.effect.<name>` style translation key for a potion item.
pub fn potion_name_key(item: &str, potion: Option<&str>) -> String {
    let prefix = item.trim_start_matches("minecraft:");
    let suffix = potion.and_then(self::potion).map_or("empty", |p| p.name);
    format!("item.minecraft.{prefix}.effect.{suffix}")
}
/// `PotionItem.getName` and `TippedArrowItem.getName`: a potion or a tipped
/// arrow is named by its contents (`PotionContents.getName`: their own
/// `custom_name`, else the potion's name, else `empty`), whatever its
/// `item_name`. None for other items, and for a stack whose contents were
/// taken off (`!minecraft:potion_contents`), which takes the item's name.
pub fn contents_name_key(stack: &ItemStack) -> Option<String> {
    use horizon_world::nbt::Tag;
    let item = &*stack.item;
    let named = matches!(
        item,
        "minecraft:potion"
            | "minecraft:splash_potion"
            | "minecraft:lingering_potion"
            | "minecraft:tipped_arrow"
    );
    let data = stack.data();
    if !named || data.is_some_and(|d| d.other("!minecraft:potion_contents").is_some()) {
        return None;
    }
    let prefix = item.trim_start_matches("minecraft:");
    let own = data
        .and_then(|d| d.potion_extra.iter().find(|(k, _)| k == "custom_name"))
        .and_then(|(_, name)| match name {
            Tag::String(s) => Some(s.to_string_lossy()),
            _ => None,
        });
    Some(match own {
        Some(suffix) => format!("item.minecraft.{prefix}.effect.{suffix}"),
        None => potion_name_key(item, stack.potion()),
    })
}
/// `PotionContents.getAllEffects`: the potion's effects, then the stack's
/// `custom_effects` (id, duration, amplifier). Unknown effect ids are dropped.
pub fn contents_effects(stack: &ItemStack) -> Vec<(&'static str, i32, i32)> {
    let potion = stack.potion().and_then(self::potion);
    let custom = stack.custom_potion_effects();
    potion
        .map_or(&[][..], |d| d.effects)
        .iter()
        .copied()
        .chain(
            custom
                .iter()
                .filter_map(|(id, duration, amp)| Some((effect(id)?.id, *duration, *amp))),
        )
        .collect()
}
/// Roman amplifier suffix (`potion.potency.N`) and duration lines of a potion.
pub fn tooltip(
    potion: Option<&str>,
    scale: f32,
    translate: &dyn Fn(&str) -> Option<String>,
) -> Vec<(String, Category)> {
    let Some(def) = potion.and_then(self::potion) else {
        return vec![];
    };
    if def.effects.is_empty() {
        return vec![(
            translate("effect.none").unwrap_or_else(|| "No Effects".into()),
            Category::Neutral,
        )];
    }
    effect_lines(def.effects.iter().copied(), scale, translate)
}
/// `PotionContents.addPotionTooltip`'s line per effect: its name, potency and
/// scaled duration, for (effect id, duration, amplifier) entries.
pub fn effect_lines<'a>(
    effects: impl IntoIterator<Item = (&'a str, i32, i32)>,
    scale: f32,
    translate: &dyn Fn(&str) -> Option<String>,
) -> Vec<(String, Category)> {
    effects
        .into_iter()
        .map(|(e, duration, amp)| {
            let e = e.trim_start_matches("minecraft:");
            let mut line = translate(&description_id(e)).unwrap_or_else(|| e.to_owned());
            if amp > 0 {
                line.push(' ');
                line.push_str(
                    &translate(&format!("potion.potency.{amp}"))
                        .unwrap_or_else(|| crate::enchant::level_name(amp as u32 + 1)),
                );
            }
            let instant = effect(e).is_some_and(|d| d.instant);
            if !instant {
                if duration == -1 {
                    // MobEffectInstance.isInfiniteDuration: formatDuration says
                    // the infinite sign, not a time.
                    line = format!(
                        "{line} ({})",
                        translate("effect.duration.infinite").unwrap_or_else(|| "∞".to_owned())
                    );
                } else {
                    let scaled = (duration as f32 * scale) as i32;
                    if scaled > 20 {
                        line = format!("{line} ({})", format_duration(scaled));
                    }
                }
            }
            (line, effect(e).map_or(Category::Neutral, |d| d.category))
        })
        .collect()
}

/// A `PotionIngredient`: item ids plus an optional potion filter. (Horizon's
/// own matcher type: vanilla 26.2 has no `PotionIngredient` class — it is
/// 26.3-only, where it backs the data-driven `minecraft:brewing` recipes.)
#[derive(Clone, Debug)]
pub struct PotionIngredient {
    pub items: HashSet<String>,
    pub potions: Option<HashSet<String>>,
}
impl PotionIngredient {
    pub fn test(&self, stack: &ItemStack) -> bool {
        self.items.contains(&*stack.item)
            && self
                .potions
                .as_ref()
                .is_none_or(|p| stack.potion().is_some_and(|x| p.contains(x)))
    }
}
/// What a 26.2 brewing mix produces: it preserves one half of the input.
/// A potion mix keeps the container and changes the potion; a container mix
/// keeps the potion and changes the container (`PotionBrewing.mix` builds
/// the result with `PotionContents.createItemStack`, count 1).
#[derive(Clone, Debug)]
pub enum BrewOutput {
    Potion(String),
    Container(String),
}
#[derive(Clone, Debug)]
pub struct BrewingRecipe {
    pub id: String,
    pub input: PotionIngredient,
    pub reagent: PotionIngredient,
    pub output: BrewOutput,
}
impl BrewingRecipe {
    /// `PotionBrewing.mix`'s result for this mix: a new count-1 stack.
    /// Custom effects and other components do not carry over — vanilla
    /// builds the result from the item and the potion alone.
    pub fn apply(&self, input: &ItemStack) -> ItemStack {
        match &self.output {
            BrewOutput::Potion(potion) => ItemStack::new(&input.item, 1).with_potion(potion),
            BrewOutput::Container(item) => match input.potion() {
                Some(potion) => ItemStack::new(item, 1).with_potion(potion),
                None => ItemStack::new(item, 1),
            },
        }
    }
}
pub struct Brewing {
    pub recipes: Vec<BrewingRecipe>,
    /// `PotionBrewing.isIngredient`: every mix's reagent item.
    pub reagents: HashSet<String>,
    /// Items accepted in the bottle slots (`BrewingStandBlockEntity`:
    /// potion, splash potion, lingering potion, glass bottle).
    pub inputs: HashSet<String>,
}
/// `PotionBrewing.addVanillaMixes` (26.2): the container mixes, in the order
/// `mix` checks them — (from item, reagent item, to item).
const CONTAINER_MIXES: &[(&str, &str, &str)] = &[
    (
        "minecraft:potion",
        "minecraft:gunpowder",
        "minecraft:splash_potion",
    ),
    (
        "minecraft:splash_potion",
        "minecraft:dragon_breath",
        "minecraft:lingering_potion",
    ),
];
/// `PotionBrewing.addVanillaMixes` (26.2): the potion mixes, in registration
/// order — (from potion, reagent item, to potion). `addStartMix` contributes
/// its water→mundane pair first, then the awkward→potion pair. All vanilla
/// mixes are feature-flag gated and enabled by default.
const POTION_MIXES: &[(&str, &str, &str)] = &[
    (
        "minecraft:water",
        "minecraft:glowstone_dust",
        "minecraft:thick",
    ),
    ("minecraft:water", "minecraft:redstone", "minecraft:mundane"),
    (
        "minecraft:water",
        "minecraft:nether_wart",
        "minecraft:awkward",
    ),
    (
        "minecraft:water",
        "minecraft:breeze_rod",
        "minecraft:mundane",
    ),
    (
        "minecraft:awkward",
        "minecraft:breeze_rod",
        "minecraft:wind_charged",
    ),
    (
        "minecraft:water",
        "minecraft:slime_block",
        "minecraft:mundane",
    ),
    (
        "minecraft:awkward",
        "minecraft:slime_block",
        "minecraft:oozing",
    ),
    ("minecraft:water", "minecraft:stone", "minecraft:mundane"),
    ("minecraft:awkward", "minecraft:stone", "minecraft:infested"),
    ("minecraft:water", "minecraft:cobweb", "minecraft:mundane"),
    ("minecraft:awkward", "minecraft:cobweb", "minecraft:weaving"),
    (
        "minecraft:awkward",
        "minecraft:golden_carrot",
        "minecraft:night_vision",
    ),
    (
        "minecraft:night_vision",
        "minecraft:redstone",
        "minecraft:long_night_vision",
    ),
    (
        "minecraft:night_vision",
        "minecraft:fermented_spider_eye",
        "minecraft:invisibility",
    ),
    (
        "minecraft:long_night_vision",
        "minecraft:fermented_spider_eye",
        "minecraft:long_invisibility",
    ),
    (
        "minecraft:invisibility",
        "minecraft:redstone",
        "minecraft:long_invisibility",
    ),
    (
        "minecraft:water",
        "minecraft:magma_cream",
        "minecraft:mundane",
    ),
    (
        "minecraft:awkward",
        "minecraft:magma_cream",
        "minecraft:fire_resistance",
    ),
    (
        "minecraft:fire_resistance",
        "minecraft:redstone",
        "minecraft:long_fire_resistance",
    ),
    (
        "minecraft:water",
        "minecraft:rabbit_foot",
        "minecraft:mundane",
    ),
    (
        "minecraft:awkward",
        "minecraft:rabbit_foot",
        "minecraft:leaping",
    ),
    (
        "minecraft:leaping",
        "minecraft:redstone",
        "minecraft:long_leaping",
    ),
    (
        "minecraft:leaping",
        "minecraft:glowstone_dust",
        "minecraft:strong_leaping",
    ),
    (
        "minecraft:leaping",
        "minecraft:fermented_spider_eye",
        "minecraft:slowness",
    ),
    (
        "minecraft:long_leaping",
        "minecraft:fermented_spider_eye",
        "minecraft:long_slowness",
    ),
    (
        "minecraft:slowness",
        "minecraft:redstone",
        "minecraft:long_slowness",
    ),
    (
        "minecraft:slowness",
        "minecraft:glowstone_dust",
        "minecraft:strong_slowness",
    ),
    (
        "minecraft:awkward",
        "minecraft:turtle_helmet",
        "minecraft:turtle_master",
    ),
    (
        "minecraft:turtle_master",
        "minecraft:redstone",
        "minecraft:long_turtle_master",
    ),
    (
        "minecraft:turtle_master",
        "minecraft:glowstone_dust",
        "minecraft:strong_turtle_master",
    ),
    (
        "minecraft:swiftness",
        "minecraft:fermented_spider_eye",
        "minecraft:slowness",
    ),
    (
        "minecraft:long_swiftness",
        "minecraft:fermented_spider_eye",
        "minecraft:long_slowness",
    ),
    ("minecraft:water", "minecraft:sugar", "minecraft:mundane"),
    (
        "minecraft:awkward",
        "minecraft:sugar",
        "minecraft:swiftness",
    ),
    (
        "minecraft:swiftness",
        "minecraft:redstone",
        "minecraft:long_swiftness",
    ),
    (
        "minecraft:swiftness",
        "minecraft:glowstone_dust",
        "minecraft:strong_swiftness",
    ),
    (
        "minecraft:awkward",
        "minecraft:pufferfish",
        "minecraft:water_breathing",
    ),
    (
        "minecraft:water_breathing",
        "minecraft:redstone",
        "minecraft:long_water_breathing",
    ),
    (
        "minecraft:water",
        "minecraft:glistering_melon_slice",
        "minecraft:mundane",
    ),
    (
        "minecraft:awkward",
        "minecraft:glistering_melon_slice",
        "minecraft:healing",
    ),
    (
        "minecraft:healing",
        "minecraft:glowstone_dust",
        "minecraft:strong_healing",
    ),
    (
        "minecraft:healing",
        "minecraft:fermented_spider_eye",
        "minecraft:harming",
    ),
    (
        "minecraft:strong_healing",
        "minecraft:fermented_spider_eye",
        "minecraft:strong_harming",
    ),
    (
        "minecraft:harming",
        "minecraft:glowstone_dust",
        "minecraft:strong_harming",
    ),
    (
        "minecraft:poison",
        "minecraft:fermented_spider_eye",
        "minecraft:harming",
    ),
    (
        "minecraft:long_poison",
        "minecraft:fermented_spider_eye",
        "minecraft:harming",
    ),
    (
        "minecraft:strong_poison",
        "minecraft:fermented_spider_eye",
        "minecraft:strong_harming",
    ),
    (
        "minecraft:water",
        "minecraft:spider_eye",
        "minecraft:mundane",
    ),
    (
        "minecraft:awkward",
        "minecraft:spider_eye",
        "minecraft:poison",
    ),
    (
        "minecraft:poison",
        "minecraft:redstone",
        "minecraft:long_poison",
    ),
    (
        "minecraft:poison",
        "minecraft:glowstone_dust",
        "minecraft:strong_poison",
    ),
    (
        "minecraft:water",
        "minecraft:ghast_tear",
        "minecraft:mundane",
    ),
    (
        "minecraft:awkward",
        "minecraft:ghast_tear",
        "minecraft:regeneration",
    ),
    (
        "minecraft:regeneration",
        "minecraft:redstone",
        "minecraft:long_regeneration",
    ),
    (
        "minecraft:regeneration",
        "minecraft:glowstone_dust",
        "minecraft:strong_regeneration",
    ),
    (
        "minecraft:water",
        "minecraft:blaze_powder",
        "minecraft:mundane",
    ),
    (
        "minecraft:awkward",
        "minecraft:blaze_powder",
        "minecraft:strength",
    ),
    (
        "minecraft:strength",
        "minecraft:redstone",
        "minecraft:long_strength",
    ),
    (
        "minecraft:strength",
        "minecraft:glowstone_dust",
        "minecraft:strong_strength",
    ),
    (
        "minecraft:water",
        "minecraft:fermented_spider_eye",
        "minecraft:weakness",
    ),
    (
        "minecraft:weakness",
        "minecraft:redstone",
        "minecraft:long_weakness",
    ),
    (
        "minecraft:awkward",
        "minecraft:phantom_membrane",
        "minecraft:slow_falling",
    ),
    (
        "minecraft:slow_falling",
        "minecraft:redstone",
        "minecraft:long_slow_falling",
    ),
];
/// The three potion containers `PotionBrewing` accepts (`containers`).
const CONTAINERS: [&str; 3] = [
    "minecraft:potion",
    "minecraft:splash_potion",
    "minecraft:lingering_potion",
];
impl Brewing {
    /// 26.2's brewing mixes: `PotionBrewing`'s hardcoded vanilla table.
    /// 26.2 has no brewing recipe type, so no data pack can add mixes —
    /// the resource parameters are unused and only kept so the call sites
    /// (fixtures, stations, the remote menu) do not churn.
    pub fn load(_root: &Path, _items: &Items, _overlay: &PackOverlay) -> Self {
        let one = |s: &str| [s.to_owned()].into_iter().collect();
        let mut recipes = Vec::with_capacity(CONTAINER_MIXES.len() + POTION_MIXES.len());
        // Container mixes first: `PotionBrewing.mix` checks them before the
        // potion mixes, and `find` takes the first match.
        for (from, reagent, to) in CONTAINER_MIXES {
            recipes.push(BrewingRecipe {
                id: format!(
                    "minecraft:brewing/{}_{}_{}",
                    from.trim_start_matches("minecraft:"),
                    reagent.trim_start_matches("minecraft:"),
                    to.trim_start_matches("minecraft:")
                ),
                input: PotionIngredient {
                    items: one(from),
                    potions: None,
                },
                reagent: PotionIngredient {
                    items: one(reagent),
                    potions: None,
                },
                output: BrewOutput::Container((*to).to_owned()),
            });
        }
        for (from, reagent, to) in POTION_MIXES {
            recipes.push(BrewingRecipe {
                id: format!(
                    "minecraft:brewing/{}_{}_{}",
                    from.trim_start_matches("minecraft:"),
                    reagent.trim_start_matches("minecraft:"),
                    to.trim_start_matches("minecraft:")
                ),
                input: PotionIngredient {
                    items: CONTAINERS.iter().map(|s| (*s).to_owned()).collect(),
                    potions: Some(one(from)),
                },
                reagent: PotionIngredient {
                    items: one(reagent),
                    potions: None,
                },
                output: BrewOutput::Potion((*to).to_owned()),
            });
        }
        let reagents = recipes
            .iter()
            .flat_map(|r| r.reagent.items.iter().cloned())
            .collect();
        let inputs = recipes
            .iter()
            .flat_map(|r| r.input.items.iter().cloned())
            .chain(std::iter::once("minecraft:glass_bottle".to_owned()))
            .collect();
        Self {
            recipes,
            reagents,
            inputs,
        }
    }
    /// `PotionBrewing.mix`'s match: the first mix whose input and reagent
    /// test, container mixes first. The source must carry potion contents
    /// — `mix` returns the source unchanged when its potion is empty, for
    /// both mix kinds. (Vanilla's `mix` does not check the item for potion
    /// mixes, but `hasMix` gates every brew on a container and
    /// `canPlaceItem` only admits the three containers and the glass bottle
    /// into the slots, so a non-container with potion contents can never
    /// reach `mix` through gameplay; the matcher restricts potion-mix
    /// inputs to the containers, which is exactly what is observable.)
    pub fn find(&self, input: &ItemStack, reagent: &ItemStack) -> Option<&BrewingRecipe> {
        if input.potion().is_none() {
            return None;
        }
        self.recipes
            .iter()
            .find(|r| r.input.test(input) && r.reagent.test(reagent))
    }
    /// `PotionBrewing.hasMix`: the source must be a container, then a
    /// container mix (no contents check) or a potion mix (contents check)
    /// matches. Note the asymmetry with `find`: a potion with no contents
    /// plus gunpowder IS brewable (the container mix matches), but `mix`
    /// then returns the source unchanged — the brew runs 400 ticks and
    /// consumes the ingredient without changing the bottle.
    pub fn has_mix(&self, input: &ItemStack, reagent: &ItemStack) -> bool {
        if !CONTAINERS.contains(&&*input.item) {
            return false;
        }
        self.recipes
            .iter()
            .any(|r| r.input.test(input) && r.reagent.test(reagent))
    }
    pub fn is_reagent(&self, stack: &ItemStack) -> bool {
        self.reagents.contains(&*stack.item)
    }
    pub fn is_bottle(&self, stack: &ItemStack) -> bool {
        self.inputs.contains(&*stack.item)
    }
}

/// What a brewing tick asks the world to do.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BrewTick {
    /// New `has_bottle_0..2` block state values.
    pub bottles: Option<[bool; 3]>,
    /// Level event 1035 (brew sound).
    pub brewed: bool,
    /// Remainders dropped at the stand.
    pub drops: Vec<ItemStack>,
}
impl BrewingStand {
    pub fn new() -> Self {
        Self::default()
    }
    fn brewable(&self, brewing: &Brewing) -> bool {
        let Some(ingredient) = &self.items[3] else {
            return false;
        };
        if !brewing.is_reagent(ingredient) {
            return false;
        }
        self.items[..3]
            .iter()
            .flatten()
            .any(|b| brewing.has_mix(b, ingredient))
    }
    /// `BrewingStandBlockEntity.serverTick` (26.2): fuel is 20 uses from any
    /// item in the `minecraft:brewing_fuel` tag, the brew takes a flat 400
    /// ticks, and the fuel stack shrinks with no remainder. 26.3's
    /// `BrewingFuel` component (uses/speed providers), the speed
    /// multiplier and the total-brew/total-fuel fields do not exist here.
    pub fn tick(&mut self, brewing: &Brewing, items: &Items) -> BrewTick {
        let mut out = BrewTick::default();
        if self.fuel <= 0
            && let Some(fuel) = self.items[4].as_mut()
            && brewing_fuel(items, fuel)
        {
            self.fuel = 20;
            fuel.count -= 1;
            if fuel.count == 0 {
                self.items[4] = None;
            }
        }
        let brewable = self.brewable(brewing);
        if self.brew_time > 0 {
            self.brew_time -= 1;
            let same = self.items[3].as_ref().map(|s| s.item.clone()) == self.ingredient;
            if self.brew_time == 0 && brewable {
                self.brew(brewing, items, &mut out);
            } else if !brewable || !same {
                self.brew_time = 0;
            }
        } else if brewable && self.fuel > 0 {
            self.fuel -= 1;
            self.brew_time = 400;
            self.ingredient = self.items[3].as_ref().map(|s| s.item.clone());
        }
        let bottles = [
            self.items[0].is_some(),
            self.items[1].is_some(),
            self.items[2].is_some(),
        ];
        if self.last_bottles != Some(bottles) {
            self.last_bottles = Some(bottles);
            out.bottles = Some(bottles);
        }
        out
    }
    pub fn idle(&self) -> bool {
        let bottles = [
            self.items[0].is_some(),
            self.items[1].is_some(),
            self.items[2].is_some(),
        ];
        self.brew_time <= 0
            && self.items[3].is_none()
            && (self.fuel > 0 || self.items[4].is_none())
            && self.last_bottles == Some(bottles)
    }
    fn brew(&mut self, brewing: &Brewing, items: &Items, out: &mut BrewTick) {
        let Some(ingredient) = self.items[3].clone() else {
            return;
        };
        for slot in 0..3 {
            if let Some(input) = &self.items[slot]
                && let Some(r) = brewing.find(input, &ingredient)
            {
                self.items[slot] = Some(r.apply(input));
            }
        }
        let remainder = items
            .get(&ingredient.item)
            .and_then(|i| i.remainder.clone());
        if let Some(i) = self.items[3].as_mut() {
            i.count -= 1;
            if i.count == 0 {
                self.items[3] = remainder.map(|r| ItemStack::new(&r, 1));
            } else if let Some(r) = remainder {
                out.drops.push(ItemStack::new(&r, 1));
            }
        }
        out.brewed = true;
    }
}
/// `BrewingStandBlockEntity` (26.2): fuel is any item in the
/// `minecraft:brewing_fuel` tag (blaze powder), 20 uses (`FUEL_USES`), brew
/// speed 1.0 (the brew time is a flat 400 ticks). 26.2 has no
/// `brewing_fuel` component.
pub fn brewing_fuel(items: &Items, stack: &ItemStack) -> bool {
    items.item_in_tag("minecraft:brewing_fuel", &stack.item)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `BrewingStandBlockEntity` (26.2): blaze powder — the only item in the
    /// `minecraft:brewing_fuel` tag — loads 20 uses and brews in a flat 400
    /// ticks; anything else is not fuel.
    #[test]
    fn blaze_powder_fuel_brews_in_400_ticks_for_20_uses() {
        let items = Items::for_tests(
            [(
                "minecraft:brewing_fuel".to_owned(),
                ["minecraft:blaze_powder".to_owned()].into_iter().collect(),
            )]
            .into_iter()
            .collect(),
        );
        let brewing = Brewing {
            reagents: ["minecraft:nether_wart".to_owned()].into_iter().collect(),
            inputs: HashSet::new(),
            recipes: vec![BrewingRecipe {
                id: "awkward".into(),
                input: PotionIngredient {
                    items: ["minecraft:potion".to_owned()].into_iter().collect(),
                    potions: Some(["minecraft:water".to_owned()].into_iter().collect()),
                },
                reagent: PotionIngredient {
                    items: ["minecraft:nether_wart".to_owned()].into_iter().collect(),
                    potions: None,
                },
                output: BrewOutput::Potion("minecraft:awkward".to_owned()),
            }],
        };
        let mut stand = BrewingStand::new();
        stand.items[0] = Some(ItemStack::new("minecraft:potion", 1).with_potion("minecraft:water"));
        stand.items[3] = Some(ItemStack::new("minecraft:nether_wart", 1));
        stand.items[4] = Some(ItemStack::new("minecraft:blaze_powder", 2));
        stand.tick(&brewing, &items);
        assert_eq!((stand.brew_time, stand.fuel), (400, 19));
        assert_eq!(stand.items[4].as_ref().unwrap().count, 1);
        for _ in 1..400 {
            assert!(!stand.tick(&brewing, &items).brewed);
        }
        assert!(stand.tick(&brewing, &items).brewed);
        assert_eq!(
            stand.items[0].as_ref().unwrap().potion(),
            Some("minecraft:awkward")
        );
        assert_eq!(stand.fuel, 19);
        assert_eq!(stand.items[4].as_ref().unwrap().count, 1);

        // Stone is not fuel: the stand never starts.
        let mut stand = BrewingStand::new();
        stand.items[0] = Some(ItemStack::new("minecraft:potion", 1).with_potion("minecraft:water"));
        stand.items[3] = Some(ItemStack::new("minecraft:nether_wart", 1));
        stand.items[4] = Some(ItemStack::new("minecraft:stone", 1));
        stand.tick(&brewing, &items);
        assert_eq!((stand.brew_time, stand.fuel), (0, 0));
    }

    /// `PotionBrewing.addVanillaMixes`: 65 mixes (2 container, 63 potion);
    /// a potion mix keeps the container, a container mix keeps the potion,
    /// and a potion with no contents is brewable with gunpowder (`hasMix`)
    /// but `mix` leaves it unchanged.
    #[test]
    fn vanilla_mixes_preserve_one_half() {
        let brewing = Brewing::load(
            std::path::Path::new(""),
            &Items::for_tests(std::collections::HashMap::new()),
            &PackOverlay::NONE,
        );
        assert_eq!(brewing.recipes.len(), 65);
        let water = ItemStack::new("minecraft:potion", 1).with_potion("minecraft:water");
        let wart = ItemStack::new("minecraft:nether_wart", 1);
        let awkward = brewing.find(&water, &wart).unwrap().apply(&water);
        assert_eq!(&*awkward.item, "minecraft:potion");
        assert_eq!(awkward.potion(), Some("minecraft:awkward"));
        // A splash water bottle stays a splash bottle through the mix.
        let splash = ItemStack::new("minecraft:splash_potion", 1).with_potion("minecraft:water");
        let awkward = brewing.find(&splash, &wart).unwrap().apply(&splash);
        assert_eq!(&*awkward.item, "minecraft:splash_potion");
        assert_eq!(awkward.potion(), Some("minecraft:awkward"));
        // Gunpowder turns a potion into a splash potion of the same potion.
        let gunpowder = ItemStack::new("minecraft:gunpowder", 1);
        let strong = ItemStack::new("minecraft:potion", 1).with_potion("minecraft:strong_healing");
        let out = brewing.find(&strong, &gunpowder).unwrap().apply(&strong);
        assert_eq!(&*out.item, "minecraft:splash_potion");
        assert_eq!(out.potion(), Some("minecraft:strong_healing"));
        // A potion with no contents: brewable, but the mix is a no-op.
        let empty = ItemStack::new("minecraft:potion", 1);
        assert!(brewing.has_mix(&empty, &gunpowder));
        assert!(brewing.find(&empty, &gunpowder).is_none());
        assert!(!brewing.has_mix(&empty, &wart));
        // A glass bottle is never a mix source.
        let bottle = ItemStack::new("minecraft:glass_bottle", 1);
        assert!(!brewing.has_mix(&bottle, &gunpowder));
        assert!(brewing.is_bottle(&bottle));
        assert!(brewing.is_reagent(&gunpowder));
        assert!(!brewing.is_reagent(&ItemStack::new("minecraft:stick", 1)));
    }

    #[test]
    fn potion_colours_and_names() {
        // Swiftness: speed colour 3402751 = 0x33EBFF.
        assert_eq!(potion_color(Some("minecraft:swiftness")), 0x33EBFF);
        assert_eq!(potion_color(Some("minecraft:water")), BASE_POTION_COLOR);
        // Turtle master mixes slowness x4 and resistance x3.
        let (s, r) = (9154528u32, 9520880u32);
        let mix = |sh: u32| ((s >> sh & 255) * 4 + (r >> sh & 255) * 3) / 7;
        assert_eq!(
            potion_color(Some("minecraft:turtle_master")),
            mix(16) << 16 | mix(8) << 8 | mix(0)
        );
        assert_eq!(
            potion_name_key("minecraft:splash_potion", Some("minecraft:long_swiftness")),
            "item.minecraft.splash_potion.effect.swiftness"
        );
        // A potion or a tipped arrow is named by its contents, over its
        // item_name; one whose contents were taken off is not.
        use horizon_world::nbt::Tag;
        let mut swift =
            ItemStack::new("minecraft:potion", 1).with_potion("minecraft:long_swiftness");
        swift
            .data_mut()
            .set_other("minecraft:item_name", Some(Tag::String("Bottle".into())));
        assert_eq!(
            contents_name_key(&swift).as_deref(),
            Some("item.minecraft.potion.effect.swiftness")
        );
        assert_eq!(
            contents_name_key(&ItemStack::new("minecraft:splash_potion", 1)).as_deref(),
            Some("item.minecraft.splash_potion.effect.empty")
        );
        let mut own = ItemStack::new("minecraft:tipped_arrow", 1).with_potion("minecraft:poison");
        own.data_mut()
            .potion_extra
            .push(("custom_name".into(), Tag::String("luck".into())));
        assert_eq!(
            contents_name_key(&own).as_deref(),
            Some("item.minecraft.tipped_arrow.effect.luck")
        );
        let mut taken = ItemStack::new("minecraft:potion", 1);
        taken.data_mut().set_other(
            "!minecraft:potion_contents",
            Some(Tag::compound(Vec::<(&str, Tag)>::new())),
        );
        assert_eq!(contents_name_key(&taken), None);
        assert_eq!(
            contents_name_key(&ItemStack::new("minecraft:stone", 1)),
            None
        );
        assert_eq!(format_duration(9600), "08:00");
        assert_eq!(format_duration(450), "00:22");
        assert_eq!(format_duration(140), "00:07");
        assert_eq!(format_duration(1800), "01:30");
        assert_eq!(format_duration(72000), "01:00:00");
        assert_eq!(format_tick_duration(9600, 40.0), "04:00");
        let t = tooltip(Some("minecraft:strong_poison"), 1.0, &|_| None);
        assert_eq!(t[0].0, "poison II (00:21)");
        assert_eq!(POTIONS.len(), 46);
    }

    #[test]
    fn brewing_stand_brews_after_400_ticks() {
        let mut item_tags = std::collections::HashMap::new();
        item_tags.insert("x".into(), HashSet::new());
        let items = Items::for_tests(item_tags);
        let recipe = BrewingRecipe {
            id: "awkward".into(),
            input: PotionIngredient {
                items: ["minecraft:potion".to_owned()].into_iter().collect(),
                potions: Some(["minecraft:water".to_owned()].into_iter().collect()),
            },
            reagent: PotionIngredient {
                items: ["minecraft:nether_wart".to_owned()].into_iter().collect(),
                potions: None,
            },
            output: BrewOutput::Potion("minecraft:awkward".to_owned()),
        };
        let brewing = Brewing {
            reagents: ["minecraft:nether_wart".to_owned()].into_iter().collect(),
            inputs: HashSet::new(),
            recipes: vec![recipe],
        };
        let mut stand = BrewingStand::new();
        stand.fuel = 1;
        stand.items[0] = Some(ItemStack::new("minecraft:potion", 1).with_potion("minecraft:water"));
        stand.items[2] =
            Some(ItemStack::new("minecraft:potion", 1).with_potion("minecraft:awkward"));
        stand.items[3] = Some(ItemStack::new("minecraft:nether_wart", 2));
        let first = stand.tick(&brewing, &items);
        assert_eq!(first.bottles, Some([true, false, true]));
        assert_eq!((stand.brew_time, stand.fuel), (400, 0));
        let mut brewed_at = 0;
        for t in 1..=400 {
            if stand.tick(&brewing, &items).brewed {
                brewed_at = t;
            }
        }
        assert_eq!(brewed_at, 400);
        assert_eq!(
            stand.items[0].as_ref().unwrap().potion(),
            Some("minecraft:awkward")
        );
        // The awkward potion had no recipe and stays.
        assert_eq!(
            stand.items[2].as_ref().unwrap().potion(),
            Some("minecraft:awkward")
        );
        assert_eq!(stand.items[3].as_ref().unwrap().count, 1);
        // No fuel left: nothing more brews.
        stand.tick(&brewing, &items);
        assert_eq!(stand.brew_time, 0);
    }

    #[test]
    fn a_brewing_stand_is_idle_only_while_its_tick_changes_nothing() {
        let mut stand = BrewingStand::new();
        assert!(!stand.idle());
        stand.last_bottles = Some([false; 3]);
        assert!(stand.idle());
        stand.items[0] = Some(ItemStack::new("minecraft:potion", 1));
        assert!(!stand.idle());
        stand.last_bottles = Some([true, false, false]);
        assert!(stand.idle());
        stand.items[4] = Some(ItemStack::new("minecraft:blaze_powder", 1));
        assert!(!stand.idle());
        stand.fuel = 3;
        assert!(stand.idle());
        stand.items[3] = Some(ItemStack::new("minecraft:nether_wart", 1));
        assert!(!stand.idle());
        stand.items[3] = None;
        stand.brew_time = 5;
        assert!(!stand.idle());
    }

    /// The stand sleeps while `idle`, so its tick must change nothing then: run the real
    /// tick on idle stands and compare, and check a stand with fuel to take wakes.
    #[test]
    fn an_idle_brewing_stand_tick_changes_nothing() {
        let items = Items::for_tests(std::collections::HashMap::new());
        let brewing = Brewing {
            reagents: ["minecraft:nether_wart".to_owned()].into_iter().collect(),
            inputs: HashSet::new(),
            recipes: vec![BrewingRecipe {
                id: "awkward".into(),
                input: PotionIngredient {
                    items: ["minecraft:potion".to_owned()].into_iter().collect(),
                    potions: Some(["minecraft:water".to_owned()].into_iter().collect()),
                },
                reagent: PotionIngredient {
                    items: ["minecraft:nether_wart".to_owned()].into_iter().collect(),
                    potions: None,
                },
                output: BrewOutput::Potion("minecraft:awkward".to_owned()),
            }],
        };
        let water = || Some(ItemStack::new("minecraft:potion", 1).with_potion("minecraft:water"));
        let stand = |bottle: bool, fuel: i32, powder: bool| {
            let mut s = BrewingStand::new();
            s.fuel = fuel;
            if bottle {
                s.items[1] = water();
            }
            if powder {
                s.items[4] = Some(ItemStack::new("minecraft:blaze_powder", 1));
            }
            s.last_bottles = Some([false, bottle, false]);
            s
        };
        for before in [
            stand(false, 0, false),
            stand(true, 0, false),
            stand(true, 5, false),
            stand(true, 5, true),
        ] {
            assert!(before.idle(), "{before:?}");
            let mut after = before.clone();
            assert_eq!(after.tick(&brewing, &items), BrewTick::default());
            assert_eq!(format!("{after:?}"), format!("{before:?}"));
        }
        // A brewable ingredient starts the brew: not idle, and the tick changes it.
        let mut brewing_now = stand(true, 5, false);
        brewing_now.items[3] = Some(ItemStack::new("minecraft:nether_wart", 1));
        assert!(!brewing_now.idle());
        brewing_now.tick(&brewing, &items);
        assert_eq!(brewing_now.brew_time, 400);
    }

    /// #3312: `PotionContents.getPotionDescription` names the effects with
    /// translatable components, so a Russian game reads them from ru_ru.json
    /// (the prepared one; skipped without it), as the tooltip's other lines.
    #[test]
    fn effect_lines_read_the_selected_language() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../.local/horizon-client")
            .join(horizon_core::VANILLA_VERSION);
        if !root.join("assets/minecraft/lang/ru_ru.json").exists() {
            return;
        }
        let ru = crate::lang::Language::load(&[&root], "ru_ru");
        let translate = |key: &str| ru.get(key).map(str::to_owned);
        let lines = tooltip(Some("minecraft:turtle_master"), 0.125, &translate);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].0.starts_with("Замедление IV ("), "{lines:?}");
        assert!(lines[1].0.starts_with("Сопротивление III ("), "{lines:?}");
    }

    #[test]
    fn infinite_effects_say_the_infinite_sign() {
        // MobEffectInstance.isInfiniteDuration: a -1 duration says the
        // infinite sign, not a time; a short one still says nothing.
        let translate = |key: &str| (key == "effect.duration.infinite").then(|| "∞".to_owned());
        let lines = effect_lines([("minecraft:resistance", -1, 0)], 1.0, &translate);
        assert_eq!(lines[0].0, "resistance (∞)");
        let lines = effect_lines([("minecraft:resistance", 15, 0)], 1.0, &translate);
        assert_eq!(lines[0].0, "resistance");
    }
}
