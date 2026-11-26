//! The down/click state of `KeyMapping` and `ToggleKeyMapping` (sneak and
//! sprint can be toggled instead of held; attack and use are plain mappings
//! in 1.21.8 — their toggle options are 1.21.9-only).

/// One mapping's runtime state. `toggle` is true for a `ToggleKeyMapping`,
/// false for a plain one.
#[derive(Clone, Debug)]
pub struct KeyState {
    pub is_down: bool,
    click_count: i32,
    toggle: bool,
    /// Held down by a controller's toggle (Horizon's pad toggle-sneak, which
    /// vanilla has no option for): until it lets go, the mapping toggles as
    /// if its vanilla option were on.
    pad_toggled: bool,
}

impl KeyState {
    pub fn plain() -> Self {
        Self {
            is_down: false,
            click_count: 0,
            toggle: false,
            pad_toggled: false,
        }
    }
    /// A `ToggleKeyMapping`. Always a keyboard key in 1.21.8 (the
    /// constructor takes no key type), and there is no `shouldRestore`
    /// flag: 1.21.9's screen-restore machinery does not exist here.
    pub fn toggle() -> Self {
        Self {
            toggle: true,
            ..Self::plain()
        }
    }
    /// `KeyMapping.click`: one more pending click.
    pub fn click(&mut self) {
        self.click_count += 1;
    }
    /// `consumeClick`.
    pub fn consume_click(&mut self) -> bool {
        if self.click_count == 0 {
            false
        } else {
            self.click_count -= 1;
            true
        }
    }
    /// The mapping toggles: its option is on, or a pad toggled it down.
    fn needs_toggle(&self, option: bool) -> bool {
        option || self.pad_toggled
    }
    /// `setDown`; `needs_toggle` is the mapping's option value.
    pub fn set_down(&mut self, down: bool, needs_toggle: bool) {
        if self.toggle && self.needs_toggle(needs_toggle) {
            if down {
                self.is_down = !self.is_down;
                // Toggled back up: the pad's toggle is over.
                self.pad_toggled &= self.is_down;
            }
        } else {
            self.is_down = down;
        }
    }
    /// A controller's toggle button: flips the mapping like `setDown(true)`
    /// with its option on, whatever the option says.
    pub fn pad_toggle(&mut self) {
        self.is_down = !self.is_down;
        self.pad_toggled = self.is_down;
    }
    /// `KeyMapping.release` (a screen opened): drops pending clicks, then
    /// `setDown(false)` — which a toggle with its option on ignores, so a
    /// toggled sneak or sprint stays down through the screen (1.21.9's
    /// `ToggleKeyMapping.release` override, which drops it and restores it
    /// on close, does not exist in 1.21.8).
    pub fn release(&mut self, needs_toggle: bool) {
        self.click_count = 0;
        self.set_down(false, needs_toggle);
    }
    /// `ToggleKeyMapping.reset`: a toggle mapping up, its pad toggle over.
    /// A plain mapping has no `reset` and is left alone.
    pub fn reset(&mut self) {
        if self.toggle {
            self.is_down = false;
            self.pad_toggled = false;
        }
    }
    /// Clicks not yet consumed.
    pub fn pending_clicks(&self) -> i32 {
        self.click_count
    }
}

/// A keyboard event (`KeyEvent.Action`): SDL reports a held key's repeats
/// as their own action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyAction {
    Release,
    Press,
    Repeat,
}

/// Sneak.
pub const SNEAK: usize = 0;
/// Sprint.
pub const SPRINT: usize = 1;
/// Attack (mouse).
pub const ATTACK: usize = 2;

/// The `ToggleKeyMapping`s `Options` builds: sneak and sprint (keyboard),
/// and the plain attack and use mappings (mouse), with the handlers that
/// drive them. `needs_toggle` is each mapping's toggle option, read at
/// every event as vanilla does; attack and use ignore it (plain mappings
/// have no toggle option in 1.21.8).
#[derive(Clone, Debug)]
pub struct ToggleKeys(pub [KeyState; 4]);
impl Default for ToggleKeys {
    fn default() -> Self {
        Self([
            KeyState::toggle(),
            KeyState::toggle(),
            KeyState::plain(),
            KeyState::plain(),
        ])
    }
}
impl ToggleKeys {
    pub fn is_down(&self, slot: usize) -> bool {
        self.0[slot].is_down
    }
    /// `KeyboardHandler.keyPress` for a key bound to `slot`: a release sets
    /// the mapping up (`KeyMapping.set(key, false)`); a press or a repeat
    /// with no screen open sets it down and clicks it.
    pub fn key(&mut self, slot: usize, action: KeyAction, screen_open: bool, needs_toggle: bool) {
        match action {
            KeyAction::Release => self.0[slot].set_down(false, needs_toggle),
            KeyAction::Press | KeyAction::Repeat if !screen_open => {
                self.0[slot].set_down(true, needs_toggle);
                self.0[slot].click();
            }
            _ => {}
        }
    }
    /// A controller's toggle button bound to `slot` (the pad's toggle-sneak):
    /// a press with no screen open flips the mapping and clicks it, as a
    /// key press does with the mapping's toggle option on.
    pub fn pad_toggle(&mut self, slot: usize, screen_open: bool) {
        if !screen_open {
            self.0[slot].pad_toggle();
            self.0[slot].click();
        }
    }
    /// `MouseHandler.onButton` for a button bound to `slot`: only without a
    /// screen or overlay.
    pub fn button(&mut self, slot: usize, pressed: bool, screen_open: bool, needs_toggle: bool) {
        if !screen_open {
            self.0[slot].set_down(pressed, needs_toggle);
            if pressed {
                self.0[slot].click();
            }
        }
    }
    /// `Gui.setScreen` with a screen: `KeyMapping.releaseAll`. Closing a
    /// screen restores nothing in 1.21.8 (`Minecraft.setScreen(null)` only
    /// resumes sound and grabs the mouse); a toggle stayed down through it.
    pub fn screen_opened(&mut self, needs_toggle: [bool; 4]) {
        for (key, option) in self.0.iter_mut().zip(needs_toggle) {
            key.release(option);
        }
    }
    /// `LocalPlayer.respawn`: `KeyMapping.resetToggleKeys`.
    pub fn respawn(&mut self) {
        for key in &mut self.0 {
            key.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    /// `KeyMapping.resetToggleKeys` (a new `LocalPlayer`): a sneak and a sprint toggled on in one
    /// world are off in the next.
    #[test]
    fn resetting_the_toggle_keys_puts_every_mapping_up() {
        let mut keys = ToggleKeys::default();
        keys.key(0, KeyAction::Press, false, true);
        keys.key(1, KeyAction::Press, false, true);
        assert!(keys.is_down(0) && keys.is_down(1));
        for key in &mut keys.0 {
            key.reset();
        }
        assert!(!keys.is_down(0) && !keys.is_down(1));
    }

    use super::*;
    #[test]
    fn toggle_flips_on_press_and_ignores_release() {
        let mut k = KeyState::toggle();
        k.set_down(true, true);
        assert!(k.is_down);
        k.set_down(false, true);
        assert!(k.is_down);
        k.set_down(true, true);
        assert!(!k.is_down);
    }
    #[test]
    fn screens_block_presses_but_not_releases() {
        let mut keys = ToggleKeys::default();
        keys.key(SNEAK, KeyAction::Press, true, false);
        assert!(!keys.is_down(SNEAK));
        keys.key(SNEAK, KeyAction::Press, false, false);
        assert!(keys.is_down(SNEAK));
        keys.key(SNEAK, KeyAction::Release, true, false);
        assert!(!keys.is_down(SNEAK));
        keys.key(SPRINT, KeyAction::Press, false, true);
        keys.respawn();
        assert!(!keys.is_down(SPRINT));
    }
    /// #3061: the pad's toggle-sneak (vanilla's toggle crouch off) stays
    /// down through a screen — 1.21.8's `release` is `setDown(false)`,
    /// which a toggle ignores — and a keyboard shift-click in the
    /// inventory does not drop it.
    #[test]
    fn pad_toggle_stays_down_through_a_screen() {
        let off = [false; 4];
        let mut keys = ToggleKeys::default();
        keys.pad_toggle(SNEAK, false);
        assert!(keys.is_down(SNEAK));
        keys.screen_opened(off);
        assert!(keys.is_down(SNEAK), "the pad's toggle holds through a screen");
        keys.key(SNEAK, KeyAction::Press, true, false);
        keys.key(SNEAK, KeyAction::Release, true, false);
        assert!(keys.is_down(SNEAK));
        // Toggled back up, it stays up through the next screen.
        keys.pad_toggle(SNEAK, false);
        assert!(!keys.is_down(SNEAK));
        keys.screen_opened(off);
        assert!(!keys.is_down(SNEAK));
        // A held key is released by a screen, as before.
        keys.key(SNEAK, KeyAction::Press, false, false);
        assert!(keys.is_down(SNEAK));
        keys.screen_opened(off);
        assert!(!keys.is_down(SNEAK));
    }
    /// After the pad's toggle is over (toggled up, or reset by a respawn),
    /// the shift key holds again; a respawn ends it even with a screen
    /// open (1.21.8 restores nothing on close).
    #[test]
    fn pad_toggle_ends_with_its_release() {
        let off = [false; 4];
        let mut keys = ToggleKeys::default();
        keys.pad_toggle(SNEAK, false);
        keys.respawn();
        assert!(!keys.is_down(SNEAK));
        keys.key(SNEAK, KeyAction::Press, false, false);
        keys.key(SNEAK, KeyAction::Release, false, false);
        assert!(!keys.is_down(SNEAK), "held, not toggled");
        // The death screen holds it and the respawn ends the toggle: it
        // does not come back.
        keys.pad_toggle(SNEAK, false);
        keys.screen_opened(off);
        assert!(keys.is_down(SNEAK), "held through the screen");
        keys.respawn();
        assert!(!keys.is_down(SNEAK));
        // A press with a screen open does nothing.
        keys.pad_toggle(SNEAK, true);
        assert!(!keys.is_down(SNEAK));
    }
    #[test]
    fn screen_release_keeps_a_toggled_key_down() {
        let mut k = KeyState::toggle();
        k.set_down(true, true);
        k.click();
        k.release(true);
        assert!(k.is_down, "a toggle ignores setDown(false)");
        assert_eq!(k.pending_clicks(), 0, "release still drops pending clicks");
        // A held key (option off) is released, clicks and all.
        let mut h = KeyState::toggle();
        h.set_down(true, false);
        h.click();
        h.release(false);
        assert!(!h.is_down);
        assert_eq!(h.pending_clicks(), 0);
        // So is a plain mapping.
        let mut p = KeyState::plain();
        p.set_down(true, false);
        p.release(false);
        assert!(!p.is_down);
    }
}
