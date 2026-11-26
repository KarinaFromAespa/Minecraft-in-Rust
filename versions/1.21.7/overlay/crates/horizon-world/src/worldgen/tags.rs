//! Version-pinned tag codecs, resource-stack merging and dependency resolution.
use super::{DecodeError, Object, optional, required};
use horizon_core::random::java_string_hash;
use serde_json::value::RawValue;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagEntry {
    pub id: String,
    pub tag: bool,
    pub required: bool,
}
impl TagEntry {
    pub fn from_json(json: &str) -> Result<Self, DecodeError> {
        let raw: Box<RawValue> = serde_json::from_str(json)?;
        let (name, mandatory) = if raw.get().starts_with('"') {
            (serde_json::from_str::<String>(raw.get())?, true)
        } else {
            let object: Object = serde_json::from_str(raw.get())?;
            (
                serde_json::from_str(required(&object, "id")?.get())?,
                optional(&object, "required")
                    .map(|v| serde_json::from_str(v.get()))
                    .transpose()?
                    .unwrap_or(true),
            )
        };
        let (tag, name) = name
            .strip_prefix('#')
            .map_or((false, name.as_str()), |s| (true, s));
        Ok(Self {
            id: identifier(name)?,
            tag,
            required: mandatory,
        })
    }
    pub fn display(&self) -> String {
        format!(
            "{}{}{}",
            if self.tag { "#" } else { "" },
            self.id,
            if self.required { "" } else { "?" }
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagFile {
    pub entries: Vec<TagEntry>,
    pub replace: bool,
}
impl TagFile {
    pub fn from_json(json: &str) -> Result<Self, DecodeError> {
        let object: Object = serde_json::from_str(json)?;
        let values: Vec<Box<RawValue>> = serde_json::from_str(required(&object, "values")?.get())?;
        let entries = values
            .iter()
            .map(|v| TagEntry::from_json(v.get()))
            .collect::<Result<_, _>>()?;
        let replace = optional(&object, "replace")
            .map(|v| serde_json::from_str(v.get()))
            .transpose()?
            .unwrap_or(false);
        Ok(Self { entries, replace })
    }
}
// Identifier.CODEC permits the empty path as well as an omitted namespace.
pub(crate) fn identifier(name: &str) -> Result<String, DecodeError> {
    let (namespace, path) = name.split_once(':').unwrap_or(("minecraft", name));
    let namespace = if namespace.is_empty() {
        "minecraft"
    } else {
        namespace
    };
    let valid = |c: u8| c.is_ascii_lowercase() || c.is_ascii_digit() || b"_.-".contains(&c);
    if !namespace.bytes().all(valid) || !path.bytes().all(|c| valid(c) || c == b'/') {
        return Err(DecodeError::Registry(format!(
            "invalid tag identifier: {name}"
        )));
    }
    Ok(format!("{namespace}:{path}"))
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagEntryWithSource {
    pub entry: TagEntry,
    pub source: String,
}
#[derive(Clone, Debug, Default)]
pub struct TagDefinitions {
    entries: BTreeMap<String, Vec<TagEntryWithSource>>,
    insertion_order: Vec<String>,
}
#[derive(Clone, Debug)]
pub struct BuiltTags<T> {
    pub tags: BTreeMap<String, Vec<T>>,
    pub missing: BTreeMap<String, Vec<TagEntryWithSource>>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TagBuildError {
    TooManyTags,
    DependencyDepth,
    HashTreeBucket,
}
impl std::fmt::Display for TagBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::TooManyTags => "tag definitions exceed 8192 entries",
            Self::DependencyDepth => {
                "tag dependency depth exceeds 256 or cyclic reachability does not terminate"
            }
            Self::HashTreeBucket => "tag identifiers require an unsupported Java hash tree bucket",
        })
    }
}
impl std::error::Error for TagBuildError {}
impl TagDefinitions {
    /// Apply resources in their pack priority order. A successfully decoded
    /// replace file clears earlier entries; a rejected file changes nothing.
    pub fn add(&mut self, name: &str, source: &str, file: TagFile) -> Result<(), DecodeError> {
        let name = identifier(name)?;
        if !self.entries.contains_key(&name) {
            self.insertion_order.push(name.clone());
        }
        let entries = self.entries.entry(name).or_default();
        if file.replace {
            entries.clear();
        }
        entries.extend(file.entries.into_iter().map(|entry| TagEntryWithSource {
            entry,
            source: source.into(),
        }));
        Ok(())
    }
    /// A valid JSON resource with a codec error still creates an empty tag in
    /// vanilla's resource loader; existing entries are retained on that error.
    pub fn apply_json(&mut self, name: &str, source: &str, json: &str) -> Result<(), DecodeError> {
        let _: Box<RawValue> = serde_json::from_str(json)?;
        let name = identifier(name)?;
        if !self.entries.contains_key(&name) {
            self.insertion_order.push(name.clone());
            self.entries.insert(name.clone(), Vec::new());
        }
        self.add(&name, source, TagFile::from_json(json)?)
    }
    pub fn definitions(&self) -> &BTreeMap<String, Vec<TagEntryWithSource>> {
        &self.entries
    }
    /// Missing required references invalidate their own tags, not unrelated tags.
    /// Element lookup receives the entry's required flag, as in vanilla.
    pub fn build<T: Clone + Ord>(
        &self,
        mut lookup: impl FnMut(&str, bool) -> Option<T>,
    ) -> Result<BuiltTags<T>, TagBuildError> {
        if self.entries.len() > 8192 {
            return Err(TagBuildError::TooManyTags);
        }
        // DependencySorter owns a fresh default-sized HashMap. Required edges
        // are inserted for every tag before any optional edge is considered.
        // The resource loader creates keys with HashMap.computeIfAbsent,
        // which inserts at the head of a bucket. DependencySorter then copies
        // that map with put (tail insertion), retaining the reversed order of
        // colliding resource keys. Resizing preserves each split chain's order.
        let order = java_bucket_order(self.insertion_order.iter().rev().map(String::as_str), 16)?;
        let mut edges: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for mandatory in [true, false] {
            for &name in &order {
                for entry in &self.entries[name] {
                    let e = &entry.entry;
                    if e.tag && e.required == mandatory && !reaches(&edges, &e.id, name, 0)? {
                        let children = edges.entry(name).or_default();
                        if !children.contains(&e.id.as_str()) {
                            children.push(&e.id);
                        }
                    }
                }
            }
        }
        // Guava HashMultimap value sets start with capacity four (two expected
        // values). Their native bucket order matters when cycles are pruned.
        let edges = edges
            .into_iter()
            .map(|(key, children)| Ok((key, java_bucket_order(children, 4)?)))
            .collect::<Result<BTreeMap<_, _>, TagBuildError>>()?;
        let mut visited = BTreeSet::new();
        let mut sorted = Vec::new();
        for name in order {
            visit(name, &edges, &mut visited, &mut sorted, 0)?;
        }
        let mut result: BuiltTags<T> = BuiltTags {
            tags: BTreeMap::new(),
            missing: BTreeMap::new(),
        };
        for name in sorted {
            let Some(entries) = self.entries.get(name) else {
                continue;
            };
            let mut seen = BTreeSet::new();
            let mut values = Vec::new();
            let mut missing = Vec::new();
            for sourced in entries {
                let entry = &sourced.entry;
                if entry.tag {
                    if let Some(children) = result.tags.get(&entry.id) {
                        for value in children {
                            if seen.insert(value.clone()) {
                                values.push(value.clone());
                            }
                        }
                    } else if entry.required {
                        missing.push(sourced.clone());
                    }
                } else if let Some(value) = lookup(&entry.id, entry.required) {
                    if seen.insert(value.clone()) {
                        values.push(value);
                    }
                } else if entry.required {
                    missing.push(sourced.clone());
                }
            }
            if missing.is_empty() {
                result.tags.insert(name.into(), values);
            } else {
                result.missing.insert(name.into(), missing);
            }
        }
        Ok(result)
    }
}
fn reaches(
    edges: &BTreeMap<&str, Vec<&str>>,
    from: &str,
    target: &str,
    depth: usize,
) -> Result<bool, TagBuildError> {
    if depth > 256 {
        return Err(TagBuildError::DependencyDepth);
    }
    if let Some(children) = edges.get(from) {
        if children.contains(&target) {
            return Ok(true);
        }
        for child in children {
            if reaches(edges, child, target, depth + 1)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
fn visit<'a>(
    name: &'a str,
    edges: &BTreeMap<&'a str, Vec<&'a str>>,
    visited: &mut BTreeSet<&'a str>,
    sorted: &mut Vec<&'a str>,
    depth: usize,
) -> Result<(), TagBuildError> {
    if !visited.insert(name) {
        return Ok(());
    }
    if depth > 256 {
        return Err(TagBuildError::DependencyDepth);
    }
    if let Some(children) = edges.get(name) {
        for child in children {
            visit(child, edges, visited, sorted, depth + 1)?;
        }
    }
    sorted.push(name);
    Ok(())
}
fn hash(name: &str) -> u32 {
    let (namespace, path) = name.split_once(':').expect("canonical tag identifier");
    let raw = java_string_hash(namespace.encode_utf16())
        .wrapping_mul(31)
        .wrapping_add(java_string_hash(path.encode_utf16())) as u32;
    raw ^ (raw >> 16)
}
fn java_bucket_order<'a>(
    names: impl IntoIterator<Item = &'a str>,
    initial: usize,
) -> Result<Vec<&'a str>, TagBuildError> {
    let names: Vec<_> = names.into_iter().collect();
    let mut capacity = initial;
    for count in 1..=names.len() {
        let bucket = hash(names[count - 1]) as usize & (capacity - 1);
        let collisions = names[..count]
            .iter()
            .filter(|name| hash(name) as usize & (capacity - 1) == bucket)
            .count();
        if collisions >= 9 {
            if capacity < 64 {
                capacity *= 2;
            } else {
                return Err(TagBuildError::HashTreeBucket);
            }
        }
        if count > capacity * 3 / 4 {
            capacity *= 2;
        }
    }
    let mut ordered = names;
    ordered.sort_by_key(|name| hash(name) as usize & (capacity - 1));
    Ok(ordered)
}

/// The 1.21.11-era `supports_*` fluid tags 1.21.7 lacks, with what fills
/// them: `true` = the whole `#water` tag (still + flowing),
/// `false` = still water only (`FluidState.getType() == Fluids.WATER`).
/// Verified against the 1.21.7 jar's bytecode.
pub(crate) const LEGACY_SUPPORT_FLUID_TAGS: [(&str, bool); 3] = [
    ("minecraft:supports_sugar_cane_adjacently", true),
    ("minecraft:supports_lily_pad", false),
    ("minecraft:supports_frogspawn", false),
];

/// Synthesize the 1.21.11 `supports_*` block tags 1.21.7 lacks, from their
/// 1.21.7 equivalents, verified against the 1.21.7 jar's bytecode. Only
/// fills tags the bundle lacks, so a data pack (or a version that has the
/// tags) keeps its own. An empty source leaves the tag absent rather than
/// empty.
///
/// `supports_frogspawn` is deliberately NOT synthesized: vanilla 1.21.7
/// `FrogspawnBlock.mayPlaceOn` is fluid-only (water below, empty above),
/// so the block tag is correctly empty and the fluid tag does the work.
pub(crate) fn synthesize_legacy_support_block_tags(block_tags: &mut BTreeMap<String, Vec<String>>) {
    let members = |tags: &BTreeMap<String, Vec<String>>, name: &str| -> BTreeSet<String> {
        tags.get(name).into_iter().flatten().cloned().collect()
    };
    let fill = |tags: &mut BTreeMap<String, Vec<String>>, name: &str, values: BTreeSet<String>| {
        if !values.is_empty() {
            tags.entry(name.to_owned())
                .or_insert_with(|| values.into_iter().collect());
        }
    };
    let dirt = members(block_tags, "minecraft:dirt");
    let sand = members(block_tags, "minecraft:sand");
    let nylium = members(block_tags, "minecraft:nylium");
    let dry = members(block_tags, "minecraft:dry_vegetation_may_place_on");
    // VegetationBlock.mayPlaceOn: dirt or farmland.
    let vegetation: BTreeSet<String> = dirt
        .iter()
        .cloned()
        .chain(["minecraft:farmland".to_owned()])
        .collect();
    fill(
        block_tags,
        "minecraft:supports_vegetation",
        vegetation.clone(),
    );
    // AzaleaBlock: clay or VegetationBlock.
    fill(
        block_tags,
        "minecraft:supports_azalea",
        vegetation
            .iter()
            .cloned()
            .chain(["minecraft:clay".to_owned()])
            .collect(),
    );
    // MangrovePropaguleBlock: clay or SaplingBlock; hanging: mangrove leaves.
    fill(
        block_tags,
        "minecraft:supports_mangrove_propagule",
        vegetation
            .iter()
            .cloned()
            .chain(["minecraft:clay".to_owned()])
            .collect(),
    );
    fill(
        block_tags,
        "minecraft:supports_hanging_mangrove_propagule",
        BTreeSet::from(["minecraft:mangrove_leaves".to_owned()]),
    );
    // DryVegetationBlock: the dry_vegetation_may_place_on tag.
    fill(block_tags, "minecraft:supports_dry_vegetation", dry);
    // NetherSproutsBlock: nylium, soul soil, or VegetationBlock.
    fill(
        block_tags,
        "minecraft:supports_nether_sprouts",
        nylium
            .iter()
            .cloned()
            .chain(["minecraft:soul_soil".to_owned()])
            .chain(vegetation.iter().cloned())
            .collect(),
    );
    // NetherWartBlock: soul sand.
    fill(
        block_tags,
        "minecraft:supports_nether_wart",
        BTreeSet::from(["minecraft:soul_sand".to_owned()]),
    );
    // WitherRoseBlock: FlowerBlock plus netherrack, soul sand, soul soil.
    fill(
        block_tags,
        "minecraft:supports_wither_rose",
        vegetation
            .iter()
            .cloned()
            .chain([
                "minecraft:netherrack".to_owned(),
                "minecraft:soul_sand".to_owned(),
                "minecraft:soul_soil".to_owned(),
            ])
            .collect(),
    );
    // CactusBlock: sand (cactus-on-cactus is the rule's own check).
    fill(block_tags, "minecraft:supports_cactus", sand.clone());
    // SugarCaneBlock: dirt or sand below; water or frosted ice beside.
    fill(
        block_tags,
        "minecraft:supports_sugar_cane",
        dirt.iter().cloned().chain(sand.iter().cloned()).collect(),
    );
    fill(
        block_tags,
        "minecraft:supports_sugar_cane_adjacently",
        BTreeSet::from(["minecraft:frosted_ice".to_owned()]),
    );
    // BambooStalkBlock/BambooSaplingBlock: the bamboo_plantable_on tag
    // (which itself includes bamboo and bamboo saplings, so stalks stack).
    fill(
        block_tags,
        "minecraft:supports_bamboo",
        members(block_tags, "minecraft:bamboo_plantable_on"),
    );
    // CocoaBlock: the jungle_logs tag (log, wood, stripped forms).
    fill(
        block_tags,
        "minecraft:supports_cocoa",
        members(block_tags, "minecraft:jungle_logs"),
    );
    // CropBlock/PitcherCropBlock: farmland.
    fill(
        block_tags,
        "minecraft:supports_crops",
        BTreeSet::from(["minecraft:farmland".to_owned()]),
    );
    // ChorusPlantBlock/ChorusFlowerBlock: end stone (chorus plant itself
    // is the rule's own check).
    fill(
        block_tags,
        "minecraft:supports_chorus_plant",
        BTreeSet::from(["minecraft:end_stone".to_owned()]),
    );
    fill(
        block_tags,
        "minecraft:supports_chorus_flower",
        BTreeSet::from(["minecraft:end_stone".to_owned()]),
    );
    // BigDripleafBlock: the big_dripleaf_placeable tag (leaf/stem itself
    // is the rule's own check).
    fill(
        block_tags,
        "minecraft:supports_big_dripleaf",
        members(block_tags, "minecraft:big_dripleaf_placeable"),
    );
    // SmallDripleafBlock: the small_dripleaf_placeable tag (the
    // water-above + VegetationBlock alternative is the rule's own check).
    fill(
        block_tags,
        "minecraft:supports_small_dripleaf",
        members(block_tags, "minecraft:small_dripleaf_placeable"),
    );
    // WaterlilyBlock: the ground block is an IceBlock — ice and frosted
    // ice only (packed ice is a plain Block and blue ice a
    // HalfTransparentBlock in 1.21.7, so neither counts).
    fill(
        block_tags,
        "minecraft:supports_lily_pad",
        BTreeSet::from([
            "minecraft:ice".to_owned(),
            "minecraft:frosted_ice".to_owned(),
        ]),
    );
}

/// Synthesize the 1.21.11 `supports_*` fluid tags 1.21.7 lacks. Sugar
/// cane takes the whole `#water` tag (`FluidState.is(FluidTags.WATER)`);
/// lily pad and frogspawn take still water only
/// (`FluidState.getType() == Fluids.WATER`). Only fills tags the bundle
/// lacks. See [`LEGACY_SUPPORT_FLUID_TAGS`].
pub(crate) fn synthesize_legacy_support_fluid_tags(fluid_tags: &mut BTreeMap<String, Vec<String>>) {
    let water: BTreeSet<String> = fluid_tags
        .get("minecraft:water")
        .into_iter()
        .flatten()
        .cloned()
        .collect();
    for (name, whole_water) in LEGACY_SUPPORT_FLUID_TAGS {
        let values: BTreeSet<String> = if whole_water {
            water.clone()
        } else {
            BTreeSet::from(["minecraft:water".to_owned()])
        };
        if !values.is_empty() {
            fluid_tags
                .entry(name.to_owned())
                .or_insert_with(|| values.into_iter().collect());
        }
    }
}

/// Synthesize the 1.21.11 `supports_*` tags 1.21.7 lacks, from their
/// 1.21.7 equivalents, verified against the 1.21.7 jar's bytecode. Only
/// fills tags the bundle lacks, so a data pack (or a version that has
/// the tags) keeps its own. An empty source leaves the tag absent rather
/// than empty.
pub(crate) fn synthesize_legacy_support_tags(
    block_tags: &mut BTreeMap<String, Vec<String>>,
    fluid_tags: &mut BTreeMap<String, Vec<String>>,
) {
    synthesize_legacy_support_block_tags(block_tags);
    synthesize_legacy_support_fluid_tags(fluid_tags);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_replacement_keeps_entries_and_codec_error_can_create_empty_tag() {
        let mut definitions = TagDefinitions::default();
        definitions
            .apply_json("a", "first", r##"{"values":["one","two","one"]}"##)
            .unwrap();
        assert!(
            definitions
                .apply_json("a", "bad", r##"{"replace":true,"values":[false]}"##)
                .is_err()
        );
        assert!(definitions.apply_json("empty", "bad", "{}").is_err());
        assert!(definitions.apply_json("syntax", "bad", "{").is_err());
        let built = definitions.build(|id, _| Some(id.to_owned())).unwrap();
        assert_eq!(
            built.tags["minecraft:a"],
            vec!["minecraft:one", "minecraft:two"]
        );
        assert_eq!(built.tags["minecraft:empty"], Vec::<String>::new());
        assert!(!built.tags.contains_key("minecraft:syntax"));
        definitions
            .apply_json(
                "a",
                "replacement",
                r##"{"replace":true,"values":["three"]}"##,
            )
            .unwrap();
        let built = definitions.build(|id, _| Some(id.to_owned())).unwrap();
        assert_eq!(built.tags["minecraft:a"], vec!["minecraft:three"]);
    }
    #[test]
    fn required_edges_precede_optional_edges_and_missing_tags_do_not_poison_siblings() {
        let mut definitions = TagDefinitions::default();
        for (name, json) in [
            ("a", r##"{"values":[{"id":"#b","required":false},"one"]}"##),
            ("b", r##"{"values":["#a","two"]}"##),
            ("bad", r##"{"values":["absent","also_absent"]}"##),
            (
                "c",
                r##"{"values":[{"id":"#bad","required":false},"two"]}"##,
            ),
        ] {
            definitions.apply_json(name, "fixture", json).unwrap();
        }
        let built = definitions
            .build(|id, _| (!id.ends_with("absent")).then(|| id.to_owned()))
            .unwrap();
        assert_eq!(built.tags["minecraft:a"], vec!["minecraft:one"]);
        assert_eq!(
            built.tags["minecraft:b"],
            vec!["minecraft:one", "minecraft:two"]
        );
        assert_eq!(built.tags["minecraft:c"], vec!["minecraft:two"]);
        assert_eq!(
            built.missing["minecraft:bad"]
                .iter()
                .map(|e| e.entry.id.as_str())
                .collect::<Vec<_>>(),
            vec!["minecraft:absent", "minecraft:also_absent"]
        );
    }

    /// The 1.21.11 `supports_*` tags don't exist in 1.21.7 data, so the
    /// survival rules' lookups resolved to empty sets: worldgen trees
    /// vanished, and hand-planted saplings, crops, cacti and sugar cane
    /// failed survival at runtime. Both loaders synthesize the tags from
    /// their 1.21.7 equivalents; this pins the synthesis itself.
    #[test]
    fn legacy_support_tags_synthesize_from_equivalents() {
        let mut block_tags = BTreeMap::from([
            (
                "minecraft:dirt".to_owned(),
                vec![
                    "minecraft:dirt".to_owned(),
                    "minecraft:grass_block".to_owned(),
                ],
            ),
            (
                "minecraft:sand".to_owned(),
                vec!["minecraft:sand".to_owned(), "minecraft:red_sand".to_owned()],
            ),
            (
                "minecraft:nylium".to_owned(),
                vec!["minecraft:crimson_nylium".to_owned()],
            ),
            (
                "minecraft:dry_vegetation_may_place_on".to_owned(),
                vec!["minecraft:sand".to_owned()],
            ),
            (
                "minecraft:bamboo_plantable_on".to_owned(),
                vec![
                    "minecraft:dirt".to_owned(),
                    "minecraft:gravel".to_owned(),
                    "minecraft:bamboo".to_owned(),
                ],
            ),
            (
                "minecraft:jungle_logs".to_owned(),
                vec![
                    "minecraft:jungle_log".to_owned(),
                    "minecraft:stripped_jungle_wood".to_owned(),
                ],
            ),
            (
                "minecraft:big_dripleaf_placeable".to_owned(),
                vec!["minecraft:clay".to_owned(), "minecraft:dirt".to_owned()],
            ),
            (
                "minecraft:small_dripleaf_placeable".to_owned(),
                vec![
                    "minecraft:clay".to_owned(),
                    "minecraft:moss_block".to_owned(),
                ],
            ),
            // A tag the bundle already has keeps its own members.
            (
                "minecraft:supports_cactus".to_owned(),
                vec!["minecraft:sand".to_owned()],
            ),
        ]);
        let mut fluid_tags = BTreeMap::from([(
            "minecraft:water".to_owned(),
            vec![
                "minecraft:water".to_owned(),
                "minecraft:flowing_water".to_owned(),
            ],
        )]);
        synthesize_legacy_support_tags(&mut block_tags, &mut fluid_tags);
        let has = |tag: &str, member: &str| {
            block_tags
                .get(tag)
                .is_some_and(|members| members.iter().any(|m| m == member))
        };
        let fluid_has = |tag: &str, member: &str| {
            fluid_tags
                .get(tag)
                .is_some_and(|members| members.iter().any(|m| m == member))
        };
        assert!(has("minecraft:supports_vegetation", "minecraft:dirt"));
        assert!(has(
            "minecraft:supports_vegetation",
            "minecraft:grass_block"
        ));
        assert!(has("minecraft:supports_vegetation", "minecraft:farmland"));
        assert!(!has("minecraft:supports_vegetation", "minecraft:sand"));
        assert!(has("minecraft:supports_azalea", "minecraft:clay"));
        assert!(has("minecraft:supports_azalea", "minecraft:farmland"));
        assert!(has(
            "minecraft:supports_mangrove_propagule",
            "minecraft:clay"
        ));
        assert!(has(
            "minecraft:supports_hanging_mangrove_propagule",
            "minecraft:mangrove_leaves"
        ));
        assert!(has("minecraft:supports_dry_vegetation", "minecraft:sand"));
        assert!(has(
            "minecraft:supports_nether_sprouts",
            "minecraft:crimson_nylium"
        ));
        assert!(has(
            "minecraft:supports_nether_sprouts",
            "minecraft:soul_soil"
        ));
        assert!(has("minecraft:supports_nether_sprouts", "minecraft:dirt"));
        assert!(has("minecraft:supports_nether_wart", "minecraft:soul_sand"));
        assert!(has(
            "minecraft:supports_wither_rose",
            "minecraft:netherrack"
        ));
        assert!(has("minecraft:supports_wither_rose", "minecraft:soul_sand"));
        assert!(has("minecraft:supports_wither_rose", "minecraft:dirt"));
        assert_eq!(
            block_tags["minecraft:supports_cactus"],
            vec!["minecraft:sand".to_owned()],
            "an existing tag keeps its own members"
        );
        assert!(has("minecraft:supports_sugar_cane", "minecraft:dirt"));
        assert!(has("minecraft:supports_sugar_cane", "minecraft:red_sand"));
        assert!(has(
            "minecraft:supports_sugar_cane_adjacently",
            "minecraft:frosted_ice"
        ));
        assert_eq!(
            fluid_tags["minecraft:supports_sugar_cane_adjacently"],
            // Members come out of a BTreeSet, so alphabetically sorted.
            vec![
                "minecraft:flowing_water".to_owned(),
                "minecraft:water".to_owned()
            ],
        );
        // The runtime-only tags (gameplay survival in level/survive.rs).
        assert!(has("minecraft:supports_bamboo", "minecraft:dirt"));
        assert!(has("minecraft:supports_bamboo", "minecraft:gravel"));
        assert!(has("minecraft:supports_bamboo", "minecraft:bamboo"));
        assert!(!has("minecraft:supports_bamboo", "minecraft:red_sand"));
        assert!(has("minecraft:supports_cocoa", "minecraft:jungle_log"));
        assert!(has(
            "minecraft:supports_cocoa",
            "minecraft:stripped_jungle_wood"
        ));
        assert!(!has("minecraft:supports_cocoa", "minecraft:oak_log"));
        assert_eq!(
            block_tags["minecraft:supports_crops"],
            vec!["minecraft:farmland".to_owned()]
        );
        assert_eq!(
            block_tags["minecraft:supports_chorus_plant"],
            vec!["minecraft:end_stone".to_owned()]
        );
        assert_eq!(
            block_tags["minecraft:supports_chorus_flower"],
            vec!["minecraft:end_stone".to_owned()]
        );
        assert!(has("minecraft:supports_big_dripleaf", "minecraft:clay"));
        assert!(has("minecraft:supports_big_dripleaf", "minecraft:dirt"));
        assert!(has("minecraft:supports_small_dripleaf", "minecraft:clay"));
        assert!(has(
            "minecraft:supports_small_dripleaf",
            "minecraft:moss_block"
        ));
        assert!(!has("minecraft:supports_small_dripleaf", "minecraft:dirt"));
        assert!(has("minecraft:supports_lily_pad", "minecraft:ice"));
        assert!(has("minecraft:supports_lily_pad", "minecraft:frosted_ice"));
        // Packed ice is a plain Block and blue ice a HalfTransparentBlock
        // in 1.21.7, so neither is an IceBlock and neither holds a lily pad.
        assert!(!has("minecraft:supports_lily_pad", "minecraft:packed_ice"));
        assert!(!has("minecraft:supports_lily_pad", "minecraft:blue_ice"));
        // Frogspawn is fluid-only in vanilla 1.21.7: no block tag.
        assert!(!block_tags.contains_key("minecraft:supports_frogspawn"));
        // Lily pad and frogspawn take still water only, not flowing water.
        assert_eq!(
            fluid_tags["minecraft:supports_lily_pad"],
            vec!["minecraft:water".to_owned()]
        );
        assert_eq!(
            fluid_tags["minecraft:supports_frogspawn"],
            vec!["minecraft:water".to_owned()]
        );
        assert!(!fluid_has(
            "minecraft:supports_lily_pad",
            "minecraft:flowing_water"
        ));
        assert!(!fluid_has(
            "minecraft:supports_frogspawn",
            "minecraft:flowing_water"
        ));
    }
}
