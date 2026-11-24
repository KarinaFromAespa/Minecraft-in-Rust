//! Audio environment attributes from Horizon's `MusicTables` and
//! `AmbientTables` for `tools/test_audio_attributes_parity.py`:
//! `audio_attributes_fixture ROOT INPUT OUTPUT`. 1.21.10 has no
//! per-situation music attributes: a biome carries one `effects.music`
//! list, and the selection follows `Situation`'s fallback chain.
use horizon_client::ambience::AmbientTables;
use horizon_client::music::{Music, MusicTables, Situation};
use horizon_client::sound_events::normalize;
use horizon_core::random::LegacyRandom;
use std::collections::HashSet;
use std::io::Write;

fn music(m: Option<&Music>) -> String {
    m.map_or("-".to_owned(), |m| {
        format!(
            "{},{},{},{}",
            normalize(&m.sound),
            m.min_delay,
            m.max_delay,
            m.replace_current_music
        )
    })
}

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
    let ambient = AmbientTables::load(root);
    let underwater_biomes = underwater_biomes(root);
    let input = std::fs::read_to_string(&args[2]).expect("input");
    let mut out = std::io::BufWriter::new(std::fs::File::create(&args[3]).expect("output"));
    for (name, m) in [
        ("MENU", Music::menu()),
        ("CREDITS", Music::credits()),
        ("END_BOSS", Music::end_boss()),
        ("CREATIVE", Music::creative()),
        ("END", Music::end()),
        ("UNDER_WATER", Music::under_water()),
        ("GAME", Music::game()),
    ] {
        writeln!(out, "{name} {}", music(Some(&m))).expect("write");
    }
    for line in input.lines() {
        let f: Vec<&str> = line.split(' ').collect();
        let (dimension, biome) = (f[0], f[1]);
        let bg = tables.background(dimension, biome);
        let list = match &bg.music {
            None => "-".to_owned(),
            Some(list) if list.is_empty() => "empty".to_owned(),
            Some(list) => list
                .iter()
                .map(|(m, w)| format!("{}:{w}", music(Some(m))))
                .collect::<Vec<_>>()
                .join(";"),
        };
        let mut s = format!("{line} biome_music={list}");
        // One pick per line, on a fixed seed: every vanilla list has a
        // single entry, so the pick is the list's track.
        let mut random = LegacyRandom::new(0);
        let picked = bg.pick(&mut random);
        for creative in [false, true] {
            for underwater in [false, true] {
                let situation = Situation {
                    screen_music: None,
                    in_world: true,
                    in_end: dimension == "minecraft:the_end",
                    in_nether: dimension == "minecraft:the_nether",
                    boss_music: false,
                    biome_has_music: bg.music.is_some(),
                    biome_music: picked.clone(),
                    probe_volume: tables.volume(dimension, biome),
                    creative,
                    underwater,
                    playing_underwater: false,
                    plays_underwater_music: underwater_biomes.contains(biome),
                };
                s.push_str(&format!(
                    " select={}",
                    music(situation.music().as_ref())
                ));
            }
        }
        s.push_str(&format!(
            " volume={:08x}",
            tables.volume(dimension, biome).to_bits()
        ));
        let a = ambient.get(dimension, biome);
        s.push_str(&format!(
            " loop={}",
            a.r#loop.as_deref().map_or("-".to_owned(), normalize)
        ));
        s.push_str(&format!(
            " mood={}",
            a.mood.as_ref().map_or("-".to_owned(), |m| format!(
                "{},{},{},{:016x}",
                normalize(&m.sound),
                m.tick_delay,
                m.block_search_extent,
                m.offset.to_bits()
            ))
        ));
        for (sound, chance) in &a.additions {
            s.push_str(&format!(
                " addition={},{:016x}",
                normalize(sound),
                chance.to_bits()
            ));
        }
        writeln!(out, "{s}").expect("write");
    }
}
