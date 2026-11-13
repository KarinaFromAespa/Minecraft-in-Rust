//! The axe, hoe and shovel block transforms (26.2): the hardcoded tables of
//! `AxeItem`, `HoeItem`, `ShovelItem` and `HoneycombItem`, evaluated the way
//! `BlockTransformer.transformBlock` would pick them. 26.2 has no
//! `minecraft:block_transformer` registry; the tables live in the item
//! classes. `tools/test_block_transformer_parity.py` compares every block
//! state with vanilla.
use crate::{Result, items::Items};
use horizon_world::{chunk::BlockState, registry::BlockRegistry};
use std::{collections::HashMap, path::Path};

enum Predicate {
    Blocks(Vec<String>),
    Tag {
        tag: String,
        offset: [i32; 3],
    },
    AllOf(Vec<Predicate>),
    /// The clicked state's property holds this value (a lit campfire).
    Property {
        key: String,
        value: String,
    },
}

enum Provider {
    Rules(Vec<(Predicate, Provider)>),
    State(BlockState),
    /// `CopyPropertiesProvider`: the source with the clicked block's properties.
    Copy(Box<Provider>),
    /// The clicked state with one property set (dowsing a campfire).
    Set {
        key: String,
        value: String,
    },
}

/// One transform: what the tool does to the clicked block.
pub struct Transform {
    provider: Provider,
    pub disallowed_faces: Vec<String>,
    pub sound: String,
    pub particle: String,
    pub loot: Option<String>,
    pub drop_strategy: String,
    pub update_from_neighbors: bool,
    pub transform_type: String,
    pub consume_on_use: bool,
    pub damage: u32,
}

pub struct Transformers {
    transformers: HashMap<String, Vec<Transform>>,
}

/// What the transformer reads of the world: the block state at an offset from
/// the clicked position.
pub trait Around {
    fn state(&self, offset: [i32; 3]) -> BlockState;
}

fn named(registry: &BlockRegistry, name: &str) -> Option<BlockState> {
    registry
        .default_state(name)
        .and_then(|id| registry.state(id))
        .cloned()
}

/// `BlockState.withPropertiesOf`: the properties both blocks have.
fn with_properties_of(
    registry: &BlockRegistry,
    mut state: BlockState,
    old: &BlockState,
) -> Option<BlockState> {
    let defaults = registry
        .default_state(&state.name)
        .and_then(|id| registry.state(id))?;
    for (k, v) in &old.properties {
        if defaults.properties.contains_key(k) {
            let mut candidate = state.clone();
            candidate.properties.insert(k.clone(), v.clone());
            if registry.resolve(&candidate).is_ok() {
                state = candidate;
            }
        }
    }
    Some(state)
}

impl Predicate {
    fn test(&self, items: &Items, around: &dyn Around) -> bool {
        match self {
            Self::Blocks(names) => {
                let state = around.state([0, 0, 0]);
                names.contains(&state.name)
            }
            Self::Tag { tag, offset } => {
                let state = around.state(*offset);
                items
                    .block_tags
                    .get(tag)
                    .is_some_and(|members| members.contains(&state.name))
            }
            Self::AllOf(all) => all.iter().all(|p| p.test(items, around)),
            Self::Property { key, value } => around
                .state([0, 0, 0])
                .properties
                .get(key)
                .is_some_and(|v| v == value),
        }
    }
}

impl Provider {
    fn state(
        &self,
        items: &Items,
        registry: &BlockRegistry,
        around: &dyn Around,
    ) -> Option<BlockState> {
        match self {
            Self::Rules(rules) => rules
                .iter()
                .find(|(p, _)| p.test(items, around))
                .and_then(|(_, then)| then.state(items, registry, around)),
            Self::State(state) => Some(state.clone()),
            Self::Copy(source) => {
                let state = source.state(items, registry, around)?;
                with_properties_of(registry, state, &around.state([0, 0, 0]))
            }
            Self::Set { key, value } => {
                let mut state = around.state([0, 0, 0]);
                state.properties.insert(key.clone(), value.clone());
                registry.resolve(&state).ok()?;
                Some(state)
            }
        }
    }
}

/// One block-to-block rule: the clicked block becomes the target, keeping the
/// properties both have (`withPropertiesOf`).
fn map_rule(registry: &BlockRegistry, from: &str, to: &str) -> Option<(Predicate, Provider)> {
    let target = named(registry, to)?;
    Some((
        Predicate::Blocks(vec![from.to_owned()]),
        Provider::Copy(Box::new(Provider::State(target))),
    ))
}

/// The same rule, only when the block above is air (`onlyIfAirAbove`).
fn air_rule(registry: &BlockRegistry, from: &str, to: &str) -> Option<(Predicate, Provider)> {
    let (p, then) = map_rule(registry, from, to)?;
    Some((
        Predicate::AllOf(vec![
            p,
            Predicate::Tag {
                tag: "minecraft:air".to_owned(),
                offset: [0, 1, 0],
            },
        ]),
        then,
    ))
}

fn transform(provider: Provider, sound: &str, particle: &str, transform_type: &str) -> Transform {
    Transform {
        provider,
        disallowed_faces: Vec::new(),
        sound: sound.to_owned(),
        particle: particle.to_owned(),
        loot: None,
        drop_strategy: "from_middle".to_owned(),
        update_from_neighbors: true,
        transform_type: transform_type.to_owned(),
        consume_on_use: true,
        damage: 1,
    }
}

/// The copper families `WeatheringCopper` and `HoneycombItem` build their
/// maps from, each with its four oxidation stages in order. The block of
/// copper names its stages `exposed_copper` etc., not `exposed_copper_block`.
fn copper_stages() -> Vec<[String; 4]> {
    let mut out = vec![[
        "minecraft:copper_block".to_owned(),
        "minecraft:exposed_copper".to_owned(),
        "minecraft:weathered_copper".to_owned(),
        "minecraft:oxidized_copper".to_owned(),
    ]];
    for base in [
        "cut_copper",
        "chiseled_copper",
        "cut_copper_slab",
        "cut_copper_stairs",
        "copper_door",
        "copper_trapdoor",
        "copper_bars",
        "copper_grate",
        "copper_bulb",
        "copper_lantern",
        "copper_chest",
        "copper_golem_statue",
        "lightning_rod",
        "copper_chain",
    ] {
        out.push([
            format!("minecraft:{base}"),
            format!("minecraft:exposed_{base}"),
            format!("minecraft:weathered_{base}"),
            format!("minecraft:oxidized_{base}"),
        ]);
    }
    out
}

/// `waxed_` on a stage name (`waxed_copper_block`, `waxed_exposed_copper`).
fn waxed(stage: &str) -> String {
    format!(
        "minecraft:waxed_{}",
        stage.strip_prefix("minecraft:").unwrap_or(stage)
    )
}

impl Transformers {
    /// The hardcoded 26.2 tables (`AxeItem.STRIPPABLES`,
    /// `WeatheringCopper.PREVIOUS_BY_BLOCK`, `HoneycombItem.WAX_OFF_BY_BLOCK`,
    /// `HoeItem.TILLABLES`, `ShovelItem.FLATTENABLES`). The root is unused:
    /// 26.2 has no `block_transformer` data.
    pub fn load(_root: &Path, registry: &BlockRegistry) -> Result<Self> {
        let rules = |pairs: &[(&str, &str)]| -> Vec<(Predicate, Provider)> {
            pairs
                .iter()
                .filter_map(|(from, to)| map_rule(registry, from, to))
                .collect()
        };
        // `AxeItem.evaluateNewBlockState`: strip first, then scrape, then
        // wax off — the transform order is that order.
        let strip = rules(&[
            ("minecraft:oak_wood", "minecraft:stripped_oak_wood"),
            ("minecraft:oak_log", "minecraft:stripped_oak_log"),
            (
                "minecraft:dark_oak_wood",
                "minecraft:stripped_dark_oak_wood",
            ),
            ("minecraft:dark_oak_log", "minecraft:stripped_dark_oak_log"),
            (
                "minecraft:pale_oak_wood",
                "minecraft:stripped_pale_oak_wood",
            ),
            ("minecraft:pale_oak_log", "minecraft:stripped_pale_oak_log"),
            ("minecraft:acacia_wood", "minecraft:stripped_acacia_wood"),
            ("minecraft:acacia_log", "minecraft:stripped_acacia_log"),
            ("minecraft:cherry_wood", "minecraft:stripped_cherry_wood"),
            ("minecraft:cherry_log", "minecraft:stripped_cherry_log"),
            ("minecraft:birch_wood", "minecraft:stripped_birch_wood"),
            ("minecraft:birch_log", "minecraft:stripped_birch_log"),
            ("minecraft:jungle_wood", "minecraft:stripped_jungle_wood"),
            ("minecraft:jungle_log", "minecraft:stripped_jungle_log"),
            ("minecraft:spruce_wood", "minecraft:stripped_spruce_wood"),
            ("minecraft:spruce_log", "minecraft:stripped_spruce_log"),
            ("minecraft:warped_stem", "minecraft:stripped_warped_stem"),
            (
                "minecraft:warped_hyphae",
                "minecraft:stripped_warped_hyphae",
            ),
            ("minecraft:crimson_stem", "minecraft:stripped_crimson_stem"),
            (
                "minecraft:crimson_hyphae",
                "minecraft:stripped_crimson_hyphae",
            ),
            (
                "minecraft:mangrove_wood",
                "minecraft:stripped_mangrove_wood",
            ),
            ("minecraft:mangrove_log", "minecraft:stripped_mangrove_log"),
            ("minecraft:bamboo_block", "minecraft:stripped_bamboo_block"),
        ]);
        // `WeatheringCopper.PREVIOUS_BY_BLOCK`: each stage to the one before
        // it, waxed and unwaxed. `HoneycombItem.WAX_OFF_BY_BLOCK`: each waxed
        // stage to its unwaxed twin.
        let mut scrape = Vec::new();
        let mut wax_off = Vec::new();
        for stages in copper_stages() {
            for stage in &stages {
                if let Some(rule) = map_rule(registry, &waxed(stage), stage) {
                    wax_off.push(rule);
                }
            }
            for pair in stages.windows(2) {
                let (from, to) = (&pair[1], &pair[0]);
                if let Some(rule) = map_rule(registry, from, to) {
                    scrape.push(rule);
                }
                let (from, to) = (waxed(from), waxed(to));
                if let Some(rule) = map_rule(registry, &from, &to) {
                    scrape.push(rule);
                }
            }
        }
        let axe = vec![
            transform(
                Provider::Rules(strip),
                "minecraft:item.axe.strip",
                "none",
                "single_block",
            ),
            transform(
                Provider::Rules(scrape),
                "minecraft:item.axe.scrape",
                "scrape",
                "copper_chest",
            ),
            transform(
                Provider::Rules(wax_off),
                "minecraft:item.axe.wax_off",
                "wax_off",
                "copper_chest",
            ),
        ];
        // `HoeItem.TILLABLES`: farmland (or dirt, for coarse dirt) only when
        // the face is not down and the block above is air; rooted dirt turns
        // to dirt unconditionally and drops hanging roots.
        let mut till = Vec::new();
        for (from, to) in [
            ("minecraft:grass_block", "minecraft:farmland"),
            ("minecraft:dirt_path", "minecraft:farmland"),
            ("minecraft:dirt", "minecraft:farmland"),
            ("minecraft:coarse_dirt", "minecraft:dirt"),
        ] {
            if let Some(rule) = air_rule(registry, from, to) {
                till.push(rule);
            }
        }
        let mut till_transform = transform(
            Provider::Rules(till),
            "minecraft:item.hoe.till",
            "none",
            "single_block",
        );
        till_transform.disallowed_faces = vec!["down".to_owned()];
        let rooted = rules(&[("minecraft:rooted_dirt", "minecraft:dirt")]);
        let mut rooted_transform = transform(
            Provider::Rules(rooted),
            "minecraft:item.hoe.till",
            "none",
            "single_block",
        );
        rooted_transform.loot = Some("minecraft:till/rooted_dirt".to_owned());
        let hoe = vec![till_transform, rooted_transform];
        // `ShovelItem.FLATTENABLES`: a dirt path only when the face is not
        // down and the block above is air; a lit campfire is dowsed (level
        // event 1009, no direct sound).
        let mut flatten = Vec::new();
        for from in [
            "minecraft:grass_block",
            "minecraft:dirt",
            "minecraft:podzol",
            "minecraft:coarse_dirt",
            "minecraft:mycelium",
            "minecraft:rooted_dirt",
        ] {
            if let Some(rule) = air_rule(registry, from, "minecraft:dirt_path") {
                flatten.push(rule);
            }
        }
        let mut flatten_transform = transform(
            Provider::Rules(flatten),
            "minecraft:item.shovel.flatten",
            "none",
            "single_block",
        );
        flatten_transform.disallowed_faces = vec!["down".to_owned()];
        let dowse = ["minecraft:campfire", "minecraft:soul_campfire"]
            .iter()
            .map(|name| {
                (
                    Predicate::AllOf(vec![
                        Predicate::Blocks(vec![(*name).to_owned()]),
                        Predicate::Property {
                            key: "lit".to_owned(),
                            value: "true".to_owned(),
                        },
                    ]),
                    Provider::Set {
                        key: "lit".to_owned(),
                        value: "false".to_owned(),
                    },
                )
            })
            .collect();
        let mut dowse_transform = transform(
            Provider::Rules(dowse),
            "minecraft:empty",
            "dowse",
            "single_block",
        );
        dowse_transform.disallowed_faces = vec!["down".to_owned()];
        let shovel = vec![flatten_transform, dowse_transform];
        Ok(Self {
            transformers: HashMap::from([
                ("minecraft:axe".to_owned(), axe),
                ("minecraft:hoe".to_owned(), hoe),
                ("minecraft:shovel".to_owned(), shovel),
            ]),
        })
    }

    /// The first transform that allows the clicked face and provides a state,
    /// with its index.
    pub fn apply<'a>(
        &'a self,
        items: &Items,
        registry: &BlockRegistry,
        id: &str,
        face: &str,
        around: &dyn Around,
    ) -> Option<(usize, &'a Transform, BlockState)> {
        self.transformers
            .get(id)?
            .iter()
            .enumerate()
            .filter(|(_, t)| !t.disallowed_faces.iter().any(|f| f == face))
            .find_map(|(i, t)| Some((i, t, t.provider.state(items, registry, around)?)))
    }

    /// `HoneycombItem.getWaxed`: the waxed twin of a copper block, found from
    /// the axe's wax-off transforms (`WAX_OFF_BY_BLOCK` is the inverse of
    /// `WAXABLES`).
    pub fn waxed(&self, registry: &BlockRegistry, old: &BlockState) -> Option<BlockState> {
        for transform in self.transformers.get("minecraft:axe")? {
            if transform.particle != "wax_off" {
                continue;
            }
            let Provider::Rules(rules) = &transform.provider else {
                continue;
            };
            for (predicate, then) in rules {
                if let (Predicate::Blocks(names), Provider::Copy(source)) = (predicate, then)
                    && let Provider::State(unwaxed) = &**source
                    && unwaxed.name == old.name
                    && let Some(waxed) = names.first()
                {
                    let target = registry
                        .default_state(waxed)
                        .and_then(|id| registry.state(id))?
                        .clone();
                    return with_properties_of(registry, target, old);
                }
            }
        }
        None
    }
}
