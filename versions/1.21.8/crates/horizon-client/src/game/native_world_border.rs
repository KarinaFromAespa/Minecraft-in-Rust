//! The world border for native server mods (docs/native-mods.md, "Server
//! hooks for plugin ports: world"): a web map draws it, a pre-generator
//! fills it.
use super::Game;

// Native mods use these; nothing in Horizon does.
impl Game<'_> {
    /// The world border of the level `dimension` (`minecraft:the_nether`) as
    /// the server enforces it: `[min_x, min_z, max_x, max_z]` in blocks
    /// (`WorldBorder.getMinX` and the others, the previous size while it
    /// moves). Vanilla keeps one border, the Overworld's in level.dat, and
    /// each other level derives its own from it, its centre divided by its
    /// coordinate scale (`Level$1`); a level not opened yet answers the
    /// derived border. None off a server and for a dimension other than
    /// the Overworld, the Nether and the End. API
    /// @API:world_border@.
    pub(super) fn native_world_border(&self, dimension: &str) -> Option<[f64; 4]> {
        let dim = match dimension {
            "minecraft:overworld" => super::Dim::Overworld,
            "minecraft:the_nether" => super::Dim::Nether,
            "minecraft:the_end" => super::Dim::End,
            _ => return None,
        };
        if !self.native_authoritative() {
            return None;
        }
        let border = if dim == self.dim {
            self.border.clone()
        } else if let Some(b) = self.levels.parked_border(dim) {
            b.clone()
        } else {
            self.load_border(dim, self.overworld_border())
        };
        Some(border.bounds())
    }
}
