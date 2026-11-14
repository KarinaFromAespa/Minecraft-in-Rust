//! Brewing parity (tools/test_brewing_parity.py): reads "input reagent
//! candidate" lines of item SNBT and prints the three brewing-slot filters
//! and the matching mix's assembled result; "tick" lines build a
//! BrewingStand, run it for their tick count, and print the same state the
//! real BrewingStandBlockEntity.serverTick oracle prints.
//!
//! The first input line is `#resources RESOURCES`; usage: brewing_fixture INPUT OUTPUT
use horizon_client::{
    brewing::{Brewing, BrewingStand, brewing_fuel},
    item_data::{canon_stack, parse_stack},
    items::{ItemStack, Items},
};
use horizon_world::loot::PackOverlay;
use std::sync::Arc;
use std::{fmt::Write as _, fs};

pub fn main() -> horizon_client::Result<()> {
    let args: Vec<String> = crate::args().collect();
    let input = fs::read_to_string(args.get(1).ok_or("missing INPUT")?)?;
    let root = input
        .lines()
        .find_map(|line| line.strip_prefix("#resources "))
        .map(std::path::PathBuf::from)
        .ok_or("no #resources line")?;
    let items = Items::catalog(&root, &PackOverlay::NONE)?;
    let brewing = Brewing::load(&root, &items, &PackOverlay::NONE);
    let mut out = String::new();
    for (n, line) in input
        .lines()
        .filter(|line| !line.starts_with('#'))
        .enumerate()
    {
        let f: Vec<&str> = line.split(' ').collect();
        if f[0] == "tick" {
            let ticks: u32 = f[1].parse()?;
            let mut slots: [Option<ItemStack>; 5] = Default::default();
            for i in 0..5 {
                slots[i] = parse_stack(f[2 + i])?;
            }
            let mut stand = BrewingStand::new();
            stand.items = slots;
            stand.brew_time = f[7].parse()?;
            stand.fuel = f[8].parse()?;
            stand.ingredient = (f[9] != "-").then(|| Arc::<str>::from(f[9]));
            let mut brews = 0;
            for _ in 0..ticks {
                brews += stand.tick(&brewing, &items).brewed as u32;
            }
            let bits: String = stand.items[..3]
                .iter()
                .map(|s| if s.is_some() { '1' } else { '0' })
                .collect();
            writeln!(
                out,
                "{n} s0={} s1={} s2={} s3={} s4={} time={} fuel={} bits={bits} ing={} brews={brews}",
                canon_stack(stand.items[0].as_ref()),
                canon_stack(stand.items[1].as_ref()),
                canon_stack(stand.items[2].as_ref()),
                canon_stack(stand.items[3].as_ref()),
                canon_stack(stand.items[4].as_ref()),
                stand.brew_time,
                stand.fuel,
                stand.ingredient.as_deref().unwrap_or("-"),
            )?;
            continue;
        }
        let input = parse_stack(f[0])?;
        let reagent = parse_stack(f[1])?;
        let candidate = parse_stack(f[2])?;
        let potion_slot = candidate.as_ref().is_some_and(|s| brewing.is_bottle(s));
        let reagent_slot = candidate.as_ref().is_some_and(|s| brewing.is_reagent(s));
        let fuel_slot = candidate.as_ref().is_some_and(|s| brewing_fuel(&items, s));
        let result = match (input.as_ref(), reagent.as_ref()) {
            (Some(input), Some(reagent)) => brewing
                .find(input, reagent)
                .map_or_else(|| "-".to_owned(), |r| canon_stack(Some(&r.apply(input)))),
            _ => "-".to_owned(),
        };
        writeln!(out, "{n} {potion_slot} {reagent_slot} {fuel_slot} {result}")?;
    }
    fs::write(args.get(2).ok_or("missing OUTPUT")?, out)?;
    Ok(())
}
