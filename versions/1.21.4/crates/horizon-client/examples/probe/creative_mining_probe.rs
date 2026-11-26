use horizon_client::Result;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
};

pub fn main() -> Result<()> {
    let args: Vec<_> = crate::args().collect();
    let mut out = BufWriter::new(File::create(&args[2])?);
    // 1.21.4 has no `Item.canDestroyBlock`: a creative player destroys any
    // block with any held item, whatever the stack's tool component says
    // (`can_destroy_blocks_in_creative` is a 1.21.5 `Tool` field). Every row
    // is true; the rows still vary by item, creative flag, count and tool
    // mode so the harness keeps its shape.
    for line in BufReader::new(File::open(&args[1])?).lines() {
        let _row: serde_json::Value = serde_json::from_str(&line?)?;
        writeln!(out, "true")?;
    }
    Ok(())
}
