use horizon_client::Result;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};

pub fn main() -> Result<()> {
    let args: Vec<_> = crate::args().collect();
    let mut out = BufWriter::new(File::create(&args[2])?);
    // 1.21.4 has no `Item.canDestroyBlock` (a 1.21.5 `Tool` field), but
    // `Item.canAttackBlock` still gates the destroy: MaceItem, SwordItem and
    // TridentItem all return `!isCreative()` in 1.21.4, so a creative player
    // destroys no block with a sword, trident or mace in hand, and destroys
    // with anything else. The tool mode never matters in 1.21.4.
    for line in BufReader::new(File::open(&args[1])?).lines() {
        let row: serde_json::Value = serde_json::from_str(&line?)?;
        let item = row["item"].as_str().unwrap_or("");
        let creative = row["creative"].as_bool().unwrap_or(false);
        let weapon = matches!(
            item,
            "minecraft:mace"
                | "minecraft:trident"
                | "minecraft:wooden_sword"
                | "minecraft:stone_sword"
                | "minecraft:iron_sword"
                | "minecraft:golden_sword"
                | "minecraft:diamond_sword"
                | "minecraft:netherite_sword"
        );
        writeln!(out, "{}", !(creative && weapon))?;
    }
    Ok(())
}
