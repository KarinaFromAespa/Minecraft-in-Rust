//! `Gui.render`'s subtitle rule.

/// Where `Gui.render` draws the subtitles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Subtitles {
    /// Hidden with the rest of the HUD: F1 hides them, screen or not.
    Hidden,
    /// In the HUD's own pass, under any screen that draws later.
    Now,
}

/// `Gui.render`'s subtitle rule: the subtitles follow the HUD (`!hideGui`).
/// 1.21.8 has no deferral over an in-game screen — F1 takes them away with
/// the rest of the HUD, and an open screen draws over them.
pub(super) fn subtitles(show_hud: bool) -> Subtitles {
    if show_hud {
        Subtitles::Now
    } else {
        Subtitles::Hidden
    }
}
