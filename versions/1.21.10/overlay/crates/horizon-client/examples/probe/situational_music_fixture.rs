//! `getSituationalMusic`/`getMusicVolume` through Horizon's `Situation` for
//! `tools/test_situational_music_parity.py`:
//! `situational_music_fixture ROOT INPUT OUTPUT`.
use horizon_client::music::{BackgroundMusic, Music, MusicTables, Situation};
use horizon_client::sound_events::normalize;
use horizon_core::random::LegacyRandom;
use std::collections::HashSet;
use std::io::Write;

/// The biomes in `#minecraft:plays_underwater_music`, `#` references
/// resolved one level (oceans and rivers).
fn underwater_biomes(root: &std::path::Path) -> HashSet<String> {
    let tags = root.join("data/minecraft/tags/worldgen/biome");
    let read = |name: &str| -> Vec<String> {
        let path = tags.join(format!("{name}.json"));
        let Ok(text) = std::fs::read_to_string(path) else {
            return Vec::new();
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
            return Vec::new();
        };
        json["values"]
            .as_array()
            .map(|v| {
                v.iter()
                    .filter_map(|e| e.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default()
    };
    let mut out = HashSet::new();
    for value in read("plays_underwater_music") {
        if let Some(inner) = value.strip_prefix("#minecraft:") {
            out.extend(read(inner));
        } else {
            out.insert(value);
        }
    }
    out
}

pub fn main() {
    let args: Vec<String> = crate::args().collect();
    let root = std::path::Path::new(&args[1]);
    let tables = MusicTables::load(root);
    let underwater = underwater_biomes(root);
    let input = std::fs::read_to_string(&args[2]).expect("input");
    let mut out = std::io::BufWriter::new(std::fs::File::create(&args[3]).expect("output"));
    for line in input.lines() {
        let f: Vec<&str> = line.split(' ').collect();
        let in_world = f[1] == "1";
        let (background, probe_volume) = if in_world {
            (tables.background(f[2], f[3]), tables.volume(f[2], f[3]))
        } else {
            (BackgroundMusic::default(), 1.0)
        };
        // One pick per line, on a fixed seed: every vanilla list has a
        // single entry, so the pick is the list's track.
        let mut random = LegacyRandom::new(0);
        let situation = Situation {
            screen_music: (f[0] == "win").then(Music::credits),
            in_world,
            in_end: f[2] == "minecraft:the_end",
            in_nether: f[2] == "minecraft:the_nether",
            boss_music: f[4].contains("music"),
            biome_has_music: background.music.is_some(),
            biome_music: background.pick(&mut random),
            probe_volume,
            creative: f[5] == "1" && f[6] == "1",
            underwater: f[7] == "1",
            playing_underwater: false,
            plays_underwater_music: underwater.contains(f[3]),
        };
        let music = situation.music().map_or("-".to_owned(), |m| {
            format!(
                "{},{},{},{}",
                normalize(&m.sound),
                m.min_delay,
                m.max_delay,
                m.replace_current_music
            )
        });
        writeln!(
            out,
            "{line} music={music} volume={:08x}",
            situation.volume().to_bits()
        )
        .expect("write");
    }
}
