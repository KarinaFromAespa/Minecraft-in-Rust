//! Game rules (1.21.10 `GameRules`): the registered rules with their category,
//! type, default and bounds, a value map, and persistence in level.dat's
//! `Data.GameRules` (camelCase keys, string values).
use horizon_world::{
    compression::Compression,
    nbt::{self, Limits, NamedTag, Tag},
};
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Value {
    Bool(bool),
    Int(i32),
}
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{b}"),
            Value::Int(i) => write!(f, "{i}"),
        }
    }
}
pub struct Rule {
    pub name: &'static str,
    pub category: &'static str,
    pub default: Value,
    /// 1.21.11's range for the rule, carried for the table: 1.21.10's
    /// `IntegerValue.set` has no range check, so `set` does not enforce it.
    pub min: i32,
    pub max: i32,
}
const fn b(name: &'static str, category: &'static str, default: bool) -> Rule {
    Rule {
        name,
        category,
        default: Value::Bool(default),
        min: 0,
        max: 0,
    }
}
const fn i(name: &'static str, category: &'static str, default: i32, min: i32, max: i32) -> Rule {
    Rule {
        name,
        category,
        default: Value::Int(default),
        min,
        max,
    }
}
/// Every rule available without experimental features, in registration order.
pub const RULES: &[Rule] = &[
    b("advance_time", "updates", true),
    b("advance_weather", "updates", true),
    b("allow_entering_nether_using_portals", "misc", true),
    b("block_drops", "drops", true),
    b("block_explosion_drop_decay", "drops", true),
    b("command_blocks_work", "misc", true),
    b("command_block_output", "chat", true),
    b("drowning_damage", "player", true),
    b("elytra_movement_check", "player", true),
    b("ender_pearls_vanish_on_death", "player", true),
    b("entity_drops", "drops", true),
    b("fall_damage", "player", true),
    b("fire_damage", "player", true),
    i(
        "fire_spread_radius_around_player",
        "updates",
        128,
        -1,
        i32::MAX,
    ),
    b("forgive_dead_players", "mobs", true),
    b("freeze_damage", "player", true),
    b("global_sound_events", "misc", true),
    b("immediate_respawn", "player", false),
    b("keep_inventory", "player", false),
    b("lava_source_conversion", "updates", false),
    b("limited_crafting", "player", false),
    b("locator_bar", "player", true),
    b("log_admin_commands", "chat", true),
    i("max_block_modifications", "misc", 32768, 1, i32::MAX),
    i("max_command_forks", "misc", 65536, 0, i32::MAX),
    i("max_command_sequence_length", "misc", 65536, 0, i32::MAX),
    i("max_entity_cramming", "mobs", 24, 0, i32::MAX),
    i("max_snow_accumulation_height", "updates", 1, 0, 8),
    b("mob_drops", "drops", true),
    b("mob_explosion_drop_decay", "drops", true),
    b("mob_griefing", "mobs", true),
    b("natural_health_regeneration", "player", true),
    b("player_movement_check", "player", true),
    i(
        "players_nether_portal_creative_delay",
        "player",
        0,
        0,
        i32::MAX,
    ),
    i(
        "players_nether_portal_default_delay",
        "player",
        80,
        0,
        i32::MAX,
    ),
    i("players_sleeping_percentage", "player", 100, 0, i32::MAX),
    b("projectiles_can_break_blocks", "drops", true),
    b("pvp", "player", true),
    b("raids", "mobs", true),
    i("random_tick_speed", "updates", 3, 0, i32::MAX),
    b("reduced_debug_info", "misc", false),
    i("respawn_radius", "player", 10, 0, i32::MAX),
    b("send_command_feedback", "chat", true),
    b("show_advancement_messages", "chat", true),
    b("show_death_messages", "chat", true),
    b("spawner_blocks_work", "misc", true),
    b("spawn_mobs", "spawning", true),
    b("spawn_monsters", "spawning", true),
    b("spawn_patrols", "spawning", true),
    b("spawn_phantoms", "spawning", true),
    b("spawn_wandering_traders", "spawning", true),
    b("spawn_wardens", "spawning", true),
    b("spectators_generate_chunks", "player", true),
    b("spread_vines", "updates", true),
    b("tnt_explodes", "misc", true),
    b("tnt_explosion_drop_decay", "drops", false),
    b("universal_anger", "mobs", false),
    b("water_source_conversion", "updates", true),
];
/// GameRules.MAX_MINECART_SPEED, available with minecart improvements.
pub const MAX_MINECART_SPEED: Rule = i("max_minecart_speed", "misc", 8, 1, 1000);
pub const MINECART_IMPROVEMENTS: &str = "minecraft:minecart_improvements";

pub fn available_rules(minecart_improvements: bool) -> impl Iterator<Item = &'static Rule> {
    RULES
        .iter()
        .chain(minecart_improvements.then_some(&MAX_MINECART_SPEED))
}

pub fn rule(name: &str) -> Option<&'static Rule> {
    let name = name.strip_prefix("minecraft:").unwrap_or(name);
    available_rules(true).find(|r| r.name == name)
}

/// Current values (defaults unless set).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GameRules {
    values: BTreeMap<&'static str, Value>,
    minecart_improvements: bool,
}
/// The 1.21.10 vanilla key for an engine rule name: most rules are the
/// snake_case name in camelCase (`command_block_output` →
/// `commandBlockOutput`); the renamed ones are listed here. `None` for
/// `fire_spread_radius_around_player`, which 1.21.10 stores as the
/// `doFireTick` / `allowFireTicksAwayFromPlayer` pair instead (see
/// [`fire_keys`]).
pub fn vanilla_key(name: &str) -> Option<String> {
    let renamed = match name {
        "advance_time" => "doDaylightCycle",
        "advance_weather" => "doWeatherCycle",
        "spawn_mobs" => "doMobSpawning",
        "spawn_patrols" => "doPatrolSpawning",
        "spawn_phantoms" => "doInsomnia",
        "spawn_wardens" => "doWardenSpawning",
        "spawn_wandering_traders" => "doTraderSpawning",
        "block_drops" => "doTileDrops",
        "mob_drops" => "doMobLoot",
        "entity_drops" => "doEntityDrops",
        "spread_vines" => "doVinesSpread",
        "raids" => "disableRaids",
        "elytra_movement_check" => "disableElytraMovementCheck",
        "player_movement_check" => "disablePlayerMovementCheck",
        "immediate_respawn" => "doImmediateRespawn",
        "natural_health_regeneration" => "naturalRegeneration",
        "limited_crafting" => "doLimitedCrafting",
        "command_blocks_work" => "commandBlocksEnabled",
        "spawner_blocks_work" => "spawnerBlocksEnabled",
        "max_command_sequence_length" => "maxCommandChainLength",
        "max_block_modifications" => "commandModificationBlockLimit",
        "max_command_forks" => "maxCommandForkCount",
        "max_minecart_speed" => "minecartMaxSpeed",
        "max_snow_accumulation_height" => "snowAccumulationHeight",
        "respawn_radius" => "spawnRadius",
        "show_advancement_messages" => "announceAdvancements",
        "fire_spread_radius_around_player" => return None,
        _ => return Some(camel_case(name)),
    };
    Some(renamed.to_owned())
}

/// The engine rule name for a 1.21.10 vanilla key, or `None` for the
/// fire pair (handled by [`fire_radius`]) and unknown keys.
pub fn from_vanilla_key(key: &str) -> Option<String> {
    let renamed = match key {
        "doDaylightCycle" => "advance_time",
        "doWeatherCycle" => "advance_weather",
        "doMobSpawning" => "spawn_mobs",
        "doPatrolSpawning" => "spawn_patrols",
        "doInsomnia" => "spawn_phantoms",
        "doWardenSpawning" => "spawn_wardens",
        "doTraderSpawning" => "spawn_wandering_traders",
        "doTileDrops" => "block_drops",
        "doMobLoot" => "mob_drops",
        "doEntityDrops" => "entity_drops",
        "doVinesSpread" => "spread_vines",
        "disableRaids" => "raids",
        "disableElytraMovementCheck" => "elytra_movement_check",
        "disablePlayerMovementCheck" => "player_movement_check",
        "doImmediateRespawn" => "immediate_respawn",
        "naturalRegeneration" => "natural_health_regeneration",
        "doLimitedCrafting" => "limited_crafting",
        "commandBlocksEnabled" => "command_blocks_work",
        "spawnerBlocksEnabled" => "spawner_blocks_work",
        "maxCommandChainLength" => "max_command_sequence_length",
        "commandModificationBlockLimit" => "max_block_modifications",
        "maxCommandForkCount" => "max_command_forks",
        "minecartMaxSpeed" => "max_minecart_speed",
        "snowAccumulationHeight" => "max_snow_accumulation_height",
        "spawnRadius" => "respawn_radius",
        "announceAdvancements" => "show_advancement_messages",
        "doFireTick" | "allowFireTicksAwayFromPlayer" => return None,
        _ => return Some(snake_case(key)),
    };
    Some(renamed.to_owned())
}

/// Rules whose vanilla key is the negation of the engine value:
/// `disableRaids` / `disableElytraMovementCheck` /
/// `disablePlayerMovementCheck` are true when the engine rule is false.
pub fn vanilla_inverted(name: &str) -> bool {
    matches!(
        name,
        "raids" | "elytra_movement_check" | "player_movement_check"
    )
}

/// The 1.21.10 fire pair for a `fire_spread_radius_around_player` value,
/// mirroring the 1.21.11 datafix in reverse: 0 is fire tick off, -1 is
/// fire ticks away from the player, anything else is the default
/// (on, near the player only — the radius itself is 1.21.11-only).
pub fn fire_keys(radius: i32) -> [(&'static str, bool); 2] {
    if radius == 0 {
        [("doFireTick", false), ("allowFireTicksAwayFromPlayer", false)]
    } else if radius == -1 {
        [("doFireTick", true), ("allowFireTicksAwayFromPlayer", true)]
    } else {
        [("doFireTick", true), ("allowFireTicksAwayFromPlayer", false)]
    }
}

/// The `fire_spread_radius_around_player` value for a 1.21.10 fire pair.
pub fn fire_radius(do_fire_tick: bool, away_from_player: bool) -> i32 {
    if !do_fire_tick {
        0
    } else if away_from_player {
        -1
    } else {
        128
    }
}

/// `Boolean.parseBoolean`, lenient as vanilla's rule parsing is: only
/// `"true"` is true, anything else parses as false when it parses at
/// all — `None` for an empty string.
fn parse_bool(s: &str) -> Option<bool> {
    if s.is_empty() {
        None
    } else {
        Some(s == "true")
    }
}

/// camelCase for a snake_case rule name (`command_block_output` →
/// `commandBlockOutput`); the renamed rules go through [`vanilla_key`].
fn camel_case(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut upper = false;
    for c in name.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.extend(c.to_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// snake_case for a camelCase vanilla key (`commandBlockOutput` →
/// `command_block_output`); the renamed keys go through
/// [`from_vanilla_key`].
fn snake_case(key: &str) -> String {
    let mut out = String::with_capacity(key.len() + 4);
    for c in key.chars() {
        if c.is_ascii_uppercase() {
            out.push('_');
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

impl GameRules {
    pub fn with_features(features: &[String]) -> Self {
        Self {
            minecart_improvements: features.iter().any(|f| f == MINECART_IMPROVEMENTS),
            ..Self::default()
        }
    }
    /// GameRules.copy(enabledFeatures): keep available values, add defaults for
    /// newly enabled rules and discard rules absent from the target feature set.
    pub fn copy_with_features(&self, features: &[String]) -> Self {
        let mut rules = Self::with_features(features);
        for (&name, &value) in &self.values {
            rules.set(name, value);
        }
        rules
    }
    pub fn available_rules(&self) -> impl Iterator<Item = &'static Rule> + use<> {
        available_rules(self.minecart_improvements)
    }
    pub fn get(&self, name: &str) -> Option<Value> {
        let r = rule(name)?;
        if r.name == MAX_MINECART_SPEED.name && !self.minecart_improvements {
            return None;
        }
        Some(self.values.get(r.name).copied().unwrap_or(r.default))
    }
    pub fn bool(&self, name: &str) -> bool {
        matches!(self.get(name), Some(Value::Bool(true)))
    }
    /// Parse a network rule value, retaining type validation (1.21.10 has
    /// no range validation to retain).
    pub fn set_string(&mut self, name: &str, text: &str) -> bool {
        let value = match self.get(name) {
            Some(Value::Bool(_)) => text.parse::<bool>().ok().map(Value::Bool),
            Some(Value::Int(_)) => text.parse::<i32>().ok().map(Value::Int),
            None => None,
        };
        value.is_some_and(|v| self.set(name, v))
    }
    pub fn int(&self, name: &str) -> i32 {
        match self.get(name) {
            Some(Value::Int(v)) => v,
            _ => 0,
        }
    }
    /// Set a rule; false if the name or value type is wrong. Any integer
    /// sets: 1.21.10's `IntegerValue.set` has no range check (the 1.21.11
    /// range on `Rule` is unenforced here).
    pub fn set(&mut self, name: &str, value: Value) -> bool {
        let Some(r) = rule(name).filter(|r| self.get(r.name).is_some()) else {
            return false;
        };
        let ok = match (r.default, value) {
            (Value::Bool(_), Value::Bool(_)) => true,
            (Value::Int(_), Value::Int(_)) => true,
            _ => false,
        };
        if ok && value == r.default {
            self.values.remove(r.name);
        } else if ok {
            self.values.insert(r.name, value);
        }
        ok
    }
    /// Every rule's value by name without namespace, as `save::Level` holds
    /// them (booleans as 0 or 1).
    pub fn by_name(&self) -> BTreeMap<String, i32> {
        self.available_rules()
            .map(|r| {
                let v = match self.values.get(r.name).copied().unwrap_or(r.default) {
                    Value::Bool(b) => i32::from(b),
                    Value::Int(i) => i,
                };
                (r.name.to_owned(), v)
            })
            .collect()
    }
    /// The rules as level.dat's `Data.GameRules` holds them: camelCase
    /// keys → string values (`"true"` / `"false"` / `"123"`), every
    /// available rule. The fire radius travels as the `doFireTick` /
    /// `allowFireTicksAwayFromPlayer` pair; the `disable*` rules are
    /// written inverted.
    fn rules_compound(&self) -> Tag {
        let mut data: Vec<(horizon_world::nbt::JavaString, Tag)> = Vec::new();
        for r in self.available_rules() {
            let v = self.values.get(r.name).copied().unwrap_or(r.default);
            if r.name == "fire_spread_radius_around_player" {
                let Value::Int(radius) = v else {
                    continue;
                };
                for (key, b) in fire_keys(radius) {
                    data.push((
                        horizon_world::nbt::JavaString::from(key),
                        crate::nbt_value::string(if b { "true" } else { "false" }),
                    ));
                }
                continue;
            }
            let Some(key) = vanilla_key(r.name) else {
                continue;
            };
            let text = match v {
                Value::Bool(b) => {
                    let b = if vanilla_inverted(r.name) { !b } else { b };
                    if b { "true" } else { "false" }.to_owned()
                }
                Value::Int(i) => i.to_string(),
            };
            data.push((
                horizon_world::nbt::JavaString::from(key.as_str()),
                crate::nbt_value::string(&text),
            ));
        }
        Tag::compound(data)
    }
    /// The rules compound applied to this map: a rule's value when the compound
    /// holds it as a string in the rule's type, its default otherwise.
    fn apply_compound(&mut self, data: &Tag) {
        // The fire pair first: it maps to one engine rule.
        let fire_tick = crate::nbt_value::text(data.get("doFireTick"))
            .map(|s| s == "true")
            .unwrap_or(true);
        let away = crate::nbt_value::text(data.get("allowFireTicksAwayFromPlayer"))
            .map(|s| s == "true")
            .unwrap_or(false);
        if data.get("doFireTick").is_some() || data.get("allowFireTicksAwayFromPlayer").is_some()
        {
            self.set(
                "fire_spread_radius_around_player",
                Value::Int(fire_radius(fire_tick, away)),
            );
        }
        for r in self.available_rules() {
            if r.name == "fire_spread_radius_around_player" {
                continue;
            }
            let Some(key) = vanilla_key(r.name) else {
                continue;
            };
            let Some(text) = crate::nbt_value::text(data.get(&key)) else {
                continue;
            };
            match r.default {
                Value::Bool(_) => {
                    if let Some(b) = parse_bool(&text) {
                        let b = if vanilla_inverted(r.name) { !b } else { b };
                        self.set(r.name, Value::Bool(b));
                    }
                }
                Value::Int(_) => {
                    if let Ok(i) = text.parse::<i32>() {
                        self.set(r.name, Value::Int(i));
                    }
                }
            }
        }
    }
    /// The rules as level.dat's `Data.GameRules` holds them, as a named tag.
    pub fn to_nbt(&self) -> NamedTag {
        NamedTag {
            name: "".into(),
            tag: self.rules_compound(),
        }
    }
    pub fn from_nbt(root: &Tag) -> Self {
        Self::from_nbt_with_features(root, &[])
    }
    pub fn from_nbt_with_features(root: &Tag, features: &[String]) -> Self {
        let mut rules = Self::with_features(features);
        rules.apply_compound(root);
        rules
    }
    pub fn load(world: &Path) -> Self {
        Self::load_with_features(world, &[])
    }
    /// `PrimaryLevelData.parse`: the rules are level.dat's `Data.GameRules`.
    pub fn load_with_features(world: &Path, features: &[String]) -> Self {
        let mut rules = Self::with_features(features);
        let Ok(bytes) = fs::read(world.join("level.dat")) else {
            return rules;
        };
        let Ok(raw) = Compression::Gzip.decode(&bytes, 16 << 20) else {
            return rules;
        };
        let Ok((root, _)) = nbt::read_named(&raw, Limits::default()) else {
            return rules;
        };
        if let Some(data) = root.tag.get("Data").and_then(|d| d.get("GameRules")) {
            rules.apply_compound(data);
        }
        rules
    }
    /// `PrimaryLevelData.setTagData`: patch level.dat's `Data.GameRules`,
    /// keeping the rest of the level data as it is.
    pub fn save(&self, world: &Path) -> crate::Result<()> {
        fs::create_dir_all(world)?;
        let path = world.join("level.dat");
        let mut root = fs::read(&path)
            .ok()
            .and_then(|bytes| Compression::Gzip.decode(&bytes, 16 << 20).ok())
            .and_then(|raw| {
                nbt::read_named(&raw, Limits::default())
                    .ok()
                    .map(|(r, _)| r.tag)
            })
            .unwrap_or_else(|| Tag::Compound(Default::default()));
        let mut data = match root.get("Data").cloned() {
            Some(t @ Tag::Compound(_)) => t,
            _ => Tag::Compound(Default::default()),
        };
        if let Tag::Compound(map) = &mut data {
            map.insert("GameRules".into(), self.rules_compound());
        }
        if let Tag::Compound(map) = &mut root {
            map.insert("Data".into(), data);
        }
        let bytes = nbt::write_named(
            &NamedTag {
                name: "".into(),
                tag: root,
            },
            Limits::default(),
        )
        .map_err(|e| e.to_string())?;
        fs::write(path, Compression::Gzip.encode(&bytes)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_values_validate_rule_names_and_types() {
        let mut rules = GameRules::default();
        assert!(rules.set_string("minecraft:keep_inventory", "true"));
        assert!(rules.bool("keep_inventory"));
        assert!(!rules.set_string("minecraft:keep_inventory", "1"));
        assert!(rules.bool("keep_inventory"));
        assert!(rules.set_string("minecraft:random_tick_speed", "7"));
        assert_eq!(rules.int("random_tick_speed"), 7);
        // 1.21.10 has no range check: -1 sets (only an unparseable
        // integer fails).
        assert!(rules.set_string("minecraft:random_tick_speed", "-1"));
        assert_eq!(rules.int("random_tick_speed"), -1);
        assert!(!rules.set_string("minecraft:random_tick_speed", "999999999999"));
        assert!(!rules.set_string("minecraft:unknown", "true"));
        assert!(!rules.set_string("other:keep_inventory", "false"));
        assert_eq!(rules.int("random_tick_speed"), -1);
    }
    #[test]
    fn defaults_and_round_trip() {
        let mut rules = GameRules::default();
        assert!(rules.bool("advance_time"));
        assert_eq!(rules.int("random_tick_speed"), 3);
        assert!(!rules.set("random_tick_speed", Value::Bool(true)));
        // 1.21.10 has no range check: 9 sets on a rule whose 1.21.11
        // range ends at 8.
        assert!(rules.set("max_snow_accumulation_height", Value::Int(9)));
        assert!(rules.set("keep_inventory", Value::Bool(true)));
        assert!(rules.set("minecraft:random_tick_speed", Value::Int(10)));
        let back = GameRules::from_nbt(&rules.to_nbt().tag);
        assert_eq!(back, rules);
        assert!(back.bool("keep_inventory"));
        let dir = std::env::temp_dir().join(format!("horizon-gamerules-{}", std::process::id()));
        rules.save(&dir).unwrap();
        assert_eq!(GameRules::load(&dir), rules);
        let _ = fs::remove_dir_all(dir);
    }
    #[test]
    fn the_minecart_speed_rule_renames_both_ways() {
        assert_eq!(
            vanilla_key("max_minecart_speed").as_deref(),
            Some("minecartMaxSpeed")
        );
        assert_eq!(
            from_vanilla_key("minecartMaxSpeed").as_deref(),
            Some("max_minecart_speed")
        );
    }
}
