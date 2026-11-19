//! ShareToLanScreen: the pause menu's "Open to LAN" — the game mode and
//! Allow Cheats for joining players, the port, Start LAN World and Cancel.
//! 1.21.11 has no WorldOptionsScreen or RestrictionsScreen (both are 26.x),
//! and the in-world game rules screen went with them: vanilla 1.21.11
//! edits rules with /gamerule. The screen only opens while the world is
//! not yet published (PauseScreen hides the button once it is).
//! Parity reference: net/minecraft/client/gui/screens/ShareToLanScreen
use super::{Game, Screen, lan::LanSettings};
use horizon_client::{
    hud::{HudBatch, rgba},
    text::Text,
    ui::{self, EditBox},
    widgets::{self, Narratable},
};
use winit::{event::KeyEvent, keyboard::PhysicalKey};

pub(super) struct ShareToLan {
    game_mode: i32,
    commands: bool,
    port: EditBox,
    /// The free port suggested while the box is empty.
    hint: u16,
    /// tryParsePort's error, while the typed port is invalid.
    port_error: Option<Text>,
    focused_button: Option<Button>,
}

/// ShareToLanScreen's game-mode cycle order: survival, spectator,
/// creative, adventure.
const MODE_ORDER: [i32; 4] = [0, 3, 1, 2];
const MODES: [&str; 4] = ["survival", "creative", "adventure", "spectator"];
/// PORT_LOWER_BOUND / PORT_HIGHER_BOUND.
const PORT_MIN: i32 = 1024;
const PORT_MAX: i32 = 65535;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Button {
    GameMode,
    Commands,
    Start,
    Cancel,
}

fn on_off(on: bool) -> Text {
    Text::translate(if on { "options.on" } else { "options.off" }, &[])
}
fn labelled(key: &str, value: Text) -> Text {
    Text::translate("options.generic_value", &[Text::translate(key, &[]), value])
}

impl ShareToLan {
    /// No button keeps the focus a click gave it.
    pub(super) fn clear_click_focus(&mut self) {
        self.focused_button = None;
    }
    /// The four buttons: the two cycles at y 100, Start and Cancel at the
    /// bottom (ShareToLanScreen.init).
    fn buttons(w: f32, h: f32) -> [(Button, f32, f32, f32); 4] {
        let cx = (w / 2.0).floor();
        [
            (Button::GameMode, cx - 155.0, 100.0, 150.0),
            (Button::Commands, cx - 5.0, 100.0, 150.0),
            (Button::Start, cx - 155.0, h - 28.0, 150.0),
            (Button::Cancel, cx - 5.0, h - 28.0, 150.0),
        ]
    }
    /// The port box, centred at y 160.
    fn port_box(w: f32) -> (f32, f32) {
        ((w / 2.0).floor() - 75.0, 160.0)
    }
    fn next_mode(mode: i32) -> i32 {
        let pos = MODE_ORDER.iter().position(|&m| m == mode).unwrap_or(0);
        MODE_ORDER[(pos + 1) % MODE_ORDER.len()]
    }
    /// ShareToLanScreen.tryParsePort: blank takes the suggested port; a
    /// number outside 1024..=65535 or one already in use is an error.
    fn try_parse_port(&mut self) {
        let value = self.port.value.trim();
        if value.is_empty() {
            self.port_error = None;
            return;
        }
        self.port_error = match value.parse::<i32>() {
            Ok(p) if (PORT_MIN..=PORT_MAX).contains(&p) => {
                if horizon_client::lan::is_port_available(p as u16) {
                    None
                } else {
                    Some(Text::translate("lanServer.port.unavailable", &[]))
                }
            }
            _ => Some(Text::translate("lanServer.port.invalid", &[])),
        };
    }
    /// The port Start publishes on: the typed one, or the suggestion.
    fn parsed_port(&self) -> u16 {
        let value = self.port.value.trim();
        if value.is_empty() {
            return self.hint;
        }
        value.parse::<i32>().map_or(self.hint, |p| p as u16)
    }
    fn hit(&self, (x, y): (f32, f32), (w, h): (f32, f32)) -> Option<Button> {
        Self::buttons(w, h)
            .into_iter()
            .find(|(b, bx, by, bw)| self.active(*b) && ui::hovered((x, y), *bx, *by, *bw, 20.0))
            .map(|b| b.0)
    }
    fn label(&self, b: Button) -> Text {
        match b {
            Button::GameMode => labelled(
                "selectWorld.gameMode",
                Text::translate(
                    &format!("selectWorld.gameMode.{}", MODES[self.game_mode as usize & 3]),
                    &[],
                ),
            ),
            Button::Commands => labelled("selectWorld.allowCommands", on_off(self.commands)),
            Button::Start => Text::translate("lanServer.start", &[]),
            Button::Cancel => Text::translate("gui.cancel", &[]),
        }
    }
    /// What button `b` narrates: the switches are `CycleButton`s ("<name>:
    /// <value> button" and the value a click moves to).
    fn narration(&self, b: Button, rect: [f32; 4], tr: super::screen_info::Tr<'_>) -> Narratable {
        let active = self.active(b);
        let mode = |m: i32| tr(&format!("selectWorld.gameMode.{}", MODES[m as usize & 3]), &[]);
        let on_off = |on: bool| tr(if on { "options.on" } else { "options.off" }, &[]);
        let labelled =
            |key: &str, value: String| tr("options.generic_value", &[&tr(key, &[]), &value]);
        let cycle = |message: String, next: String| Narratable::cycle(message, next, rect, active);
        match b {
            Button::GameMode => {
                let key = "selectWorld.gameMode";
                cycle(
                    labelled(key, mode(self.game_mode)),
                    labelled(key, mode(Self::next_mode(self.game_mode))),
                )
            }
            Button::Commands => {
                let key = "selectWorld.allowCommands";
                cycle(
                    labelled(key, on_off(self.commands)),
                    labelled(key, on_off(!self.commands)),
                )
            }
            _ => Narratable::button(self.label(b).plain(), rect, active),
        }
    }
    /// Start stays inactive while the typed port is invalid (the port
    /// box's responder); the rest are always active.
    fn active(&self, b: Button) -> bool {
        match b {
            Button::Start => self.port_error.is_none(),
            _ => true,
        }
    }
}

impl Game<'_> {
    pub(super) fn open_share_to_lan(&mut self) {
        let hint = horizon_client::lan::available_port();
        let mut port = EditBox::new(0.0, 0.0, 150.0, 20.0);
        port.message = super::screen_info::tr("lanServer.port", &[]);
        port.max_length = 5;
        // The world is the server thread's when there is one.
        let (game_mode, commands) = match &self.integrated {
            Some(server) => match server.call(|g| g.server_options()) {
                Some(state) => (state.default_mode, state.cheats),
                None => return,
            },
            None => (self.level.mode(), self.allow_commands()),
        };
        self.share_to_lan = Some(ShareToLan {
            game_mode,
            commands,
            port,
            hint,
            port_error: None,
            focused_button: None,
        });
        self.screen = Screen::ShareToLan;
    }

    pub(super) fn close_share_to_lan(&mut self) {
        self.share_to_lan = None;
        self.screen = Screen::Paused;
    }

    /// Escape: back to the pause menu (ShareToLanScreen.onClose).
    pub(super) fn share_to_lan_escape(&mut self) {
        self.close_share_to_lan();
    }

    /// ShareToLanScreen's publish message: a client system message the
    /// narrator says.
    fn send_publish_message(&mut self, message: Text) {
        self.say_system_queued(&message);
        self.system_message(message);
    }

    /// The Start button: close, then publish with the chosen game mode,
    /// commands and port (IntegratedServer.publishServer).
    fn start_share_to_lan(&mut self, mode: i32, commands: bool, port: u16) {
        self.close_share_to_lan();
        if let Some(server) = &self.integrated {
            server.call(move |g| {
                g.set_default_game_mode(mode);
                g.set_allow_commands(commands);
                g.owner_allow_commands(commands);
            });
        } else {
            self.set_default_game_mode(mode);
            self.set_allow_commands(commands);
            self.owner_allow_commands(commands);
        }
        // 1.21.11's publish has no guest-command or force settings: a fresh
        // publish takes the defaults (guests need op; the default mode binds).
        let settings = LanSettings {
            port,
            guest_commands: false,
            force_game_mode: true,
        };
        if self.lan_open(settings).is_err() {
            self.send_publish_message(Text::translate("commands.publish.failed", &[]));
            return;
        }
        // lan_open sent the started message to chat; the narrator says it too.
        if let Some(port) = self.lan_port() {
            let message = Text::translate(
                "commands.publish.started",
                &[Text::literal(port.to_string())],
            );
            self.say_system_queued(&message);
        }
    }

    pub(super) fn draw_share_to_lan(&mut self, batch: &mut HudBatch) {
        let Some(mut s) = self.share_to_lan.take() else {
            return;
        };
        widgets::begin_screen(|| {
            widgets::ScreenInfo::new(
                "share_to_lan",
                super::screen_info::tr("lanServer.title", &[]),
            )
        });
        let atlas = &self.gui.atlas;
        let (w, h) = (batch.width, batch.height);
        ui::menu_background(batch, atlas, 0.0, 0.0, w, h, true);
        let tr = &super::screen_info::tr;
        // ShareToLanScreen.render: the title and the two info lines, centred.
        for (key, y) in [
            ("lanServer.title", 50.0),
            ("lanServer.otherPlayers", 82.0),
            ("lanServer.port", 142.0),
        ] {
            let t = Text::translate(key, &[]);
            let tw = atlas.font.width_text(&t);
            batch.text_styled(atlas, &t, ((w - tw) / 2.0).floor(), y, u32::MAX, true);
        }
        for (b, bx, by, bw) in ShareToLan::buttons(w, h) {
            let active = s.active(b);
            let is_focused = s.focused_button == Some(b);
            let hover = active && (ui::hovered(self.cursor, bx, by, bw, 20.0) || is_focused);
            ui::button_as(batch, atlas, bx, by, bw, 20.0, &s.label(b), active, hover, |rect| {
                s.narration(b, rect, tr)
            });
        }
        // The port box: its hint the port to use, red while invalid.
        let (px, py) = ShareToLan::port_box(w);
        s.port.x = px;
        s.port.y = py;
        s.port.text_color = if s.port_error.is_some() {
            rgba(0xDF, 0x50, 0x50, 255)
        } else {
            rgba(0xE0, 0xE0, 0xE0, 255)
        };
        s.port.draw(batch, atlas, None);
        if s.port.value.is_empty() && !s.port.focused {
            // EditBox.DEFAULT_HINT_STYLE: dark gray.
            let hint = Text::literal(s.hint.to_string());
            batch.text_styled(
                atlas,
                &hint,
                s.port.text_x(),
                s.port.text_y(),
                rgba(0x55, 0x55, 0x55, 255),
                true,
            );
        }
        // An invalid port explains itself on hover (the EditBox tooltip).
        if let Some(error) = &s.port_error
            && ui::hovered(self.cursor, px, py, 150.0, 20.0)
        {
            let lines: Vec<Text> = error
                .plain()
                .split('\n')
                .map(|l| Text::literal(l.to_string()))
                .collect();
            ui::tooltip(batch, atlas, &lines, self.cursor);
        }
        self.share_to_lan = Some(s);
    }

    pub(super) fn share_to_lan_click(&mut self) {
        let Some(mut s) = self.share_to_lan.take() else {
            return;
        };
        let (w, h) = self.gui_size();
        let (px, py) = ShareToLan::port_box(w);
        s.port.focused = ui::hovered(self.cursor, px, py, 150.0, 20.0);
        let hit = s.hit(self.cursor, (w, h));
        if let Some(b) = hit {
            s.focused_button = Some(b);
            self.audio.play_button_click();
        }
        match hit {
            Some(Button::GameMode) => s.game_mode = ShareToLan::next_mode(s.game_mode),
            Some(Button::Commands) => s.commands = !s.commands,
            Some(Button::Cancel) => {
                self.share_to_lan = Some(s);
                self.close_share_to_lan();
                return;
            }
            Some(Button::Start) => {
                let (mode, commands, port) = (s.game_mode, s.commands, s.parsed_port());
                return self.start_share_to_lan(mode, commands, port);
            }
            None => {}
        }
        self.share_to_lan = Some(s);
    }

    pub(super) fn share_to_lan_key(&mut self, event: &KeyEvent) {
        if event.state != winit::event::ElementState::Pressed {
            return;
        }
        let PhysicalKey::Code(key) = event.physical_key else {
            return;
        };
        let ctrl = self.has_control_down_with_quirk();
        let shortcut = self.edit_shortcut(event);
        let types_text = self.types_text();
        let shift = self.has_shift_down();
        let (w, h) = self.gui_size();
        let Some(s) = self.share_to_lan.as_mut() else {
            return;
        };
        if s.port.focused {
            // Tab leaves the box: forward to Start, back to Allow Cheats.
            if key == winit::keyboard::KeyCode::Tab {
                s.port.focused = false;
                s.focused_button = Some(if shift {
                    Button::Commands
                } else {
                    Button::Start
                });
                return;
            }
            if let Some(shortcut) = shortcut {
                s.port.shortcut(shortcut, &mut self.clipboard);
                s.try_parse_port();
                return;
            }
            // Only a key with text types (a `None` is a backspace to
            // `share_to_lan_text`, the control socket's `\b`).
            if !s.port.key(key, ctrl, false)
                && types_text
                && let Some(text) = event.text.as_deref()
            {
                self.share_to_lan_text(Some(text));
            } else {
                s.try_parse_port();
            }
            return;
        }
        let active_buttons: Vec<Button> = ShareToLan::buttons(w, h)
            .iter()
            .filter(|b| s.active(b.0))
            .map(|b| b.0)
            .collect();
        if active_buttons.is_empty() {
            return;
        }
        match key {
            winit::keyboard::KeyCode::Tab => {
                let reverse = shift;
                let next = match s.focused_button {
                    // Tab steps through the port box between Allow Cheats and Start.
                    Some(Button::Commands) if !reverse => {
                        s.port.focused = true;
                        s.focused_button = None;
                        return;
                    }
                    Some(Button::Start) if reverse => {
                        s.port.focused = true;
                        s.focused_button = None;
                        return;
                    }
                    Some(cur) => {
                        if let Some(pos) = active_buttons.iter().position(|&b| b == cur) {
                            if reverse {
                                if pos == 0 {
                                    active_buttons.last().copied()
                                } else {
                                    Some(active_buttons[pos - 1])
                                }
                            } else if pos + 1 >= active_buttons.len() {
                                active_buttons.first().copied()
                            } else {
                                Some(active_buttons[pos + 1])
                            }
                        } else if reverse {
                            active_buttons.last().copied()
                        } else {
                            active_buttons.first().copied()
                        }
                    }
                    None => {
                        if reverse {
                            active_buttons.last().copied()
                        } else {
                            active_buttons.first().copied()
                        }
                    }
                };
                s.focused_button = next;
            }
            winit::keyboard::KeyCode::ArrowUp
            | winit::keyboard::KeyCode::ArrowDown
            | winit::keyboard::KeyCode::ArrowLeft
            | winit::keyboard::KeyCode::ArrowRight => {
                let rects: Vec<(Button, f32, f32, f32)> = ShareToLan::buttons(w, h)
                    .into_iter()
                    .filter(|b| s.active(b.0))
                    .collect();
                let cur_b = s.focused_button.unwrap_or(active_buttons[0]);
                if let Some(cur) = rects.iter().find(|r| r.0 == cur_b) {
                    let (cx, cy) = (cur.1 + cur.3 / 2.0, cur.2 + 10.0);
                    let mut best: Option<(Button, f32)> = None;
                    for r in &rects {
                        if r.0 == cur_b {
                            continue;
                        }
                        let (rx, ry) = (r.1 + r.3 / 2.0, r.2 + 10.0);
                        let (dx, dy) = (rx - cx, ry - cy);
                        let valid = match key {
                            winit::keyboard::KeyCode::ArrowUp => dy < -4.0,
                            winit::keyboard::KeyCode::ArrowDown => dy > 4.0,
                            winit::keyboard::KeyCode::ArrowLeft => dx < -4.0,
                            winit::keyboard::KeyCode::ArrowRight => dx > 4.0,
                            _ => false,
                        };
                        if !valid {
                            continue;
                        }
                        let dist = match key {
                            winit::keyboard::KeyCode::ArrowUp
                            | winit::keyboard::KeyCode::ArrowDown => dy.abs() + dx.abs() * 2.0,
                            winit::keyboard::KeyCode::ArrowLeft
                            | winit::keyboard::KeyCode::ArrowRight => dx.abs() + dy.abs() * 2.0,
                            _ => f32::MAX,
                        };
                        if best.as_ref().is_none_or(|(_, d)| dist < *d) {
                            best = Some((r.0, dist));
                        }
                    }
                    if let Some((next_b, _)) = best {
                        s.focused_button = Some(next_b);
                    }
                } else {
                    s.focused_button = Some(active_buttons[0]);
                }
            }
            winit::keyboard::KeyCode::Enter
            | winit::keyboard::KeyCode::NumpadEnter
            | winit::keyboard::KeyCode::Space => {
                if let Some(btn) = s.focused_button {
                    self.audio.play_button_click();
                    let mut s = self.share_to_lan.take().unwrap();
                    match btn {
                        Button::GameMode => s.game_mode = ShareToLan::next_mode(s.game_mode),
                        Button::Commands => s.commands = !s.commands,
                        Button::Cancel => {
                            self.share_to_lan = Some(s);
                            self.close_share_to_lan();
                            return;
                        }
                        Button::Start => {
                            let (mode, commands, port) =
                                (s.game_mode, s.commands, s.parsed_port());
                            return self.start_share_to_lan(mode, commands, port);
                        }
                    }
                    self.share_to_lan = Some(s);
                }
            }
            _ => {}
        }
    }
    /// Whether the Share to LAN port box has focus.
    pub(super) fn share_to_lan_typing(&self) -> bool {
        self.share_to_lan.as_ref().is_some_and(|s| s.port.focused)
    }
    /// Digits typed into the port field (the keyboard, or a controller's
    /// on-screen keyboard), and its backspace.
    pub(super) fn share_to_lan_text(&mut self, text: Option<&str>) {
        let Some(s) = self.share_to_lan.as_mut().filter(|s| s.port.focused) else {
            return;
        };
        match text {
            Some(text) => {
                let digits: String = text.chars().filter(char::is_ascii_digit).collect();
                if !digits.is_empty() {
                    s.port.insert_text(&digits);
                }
            }
            None => {
                s.port
                    .key(winit::keyboard::KeyCode::Backspace, false, false);
            }
        }
        s.try_parse_port();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(key: &str, args: &[&str]) -> String {
        if args.is_empty() {
            key.to_owned()
        } else {
            format!("{key}({})", args.join(", "))
        }
    }

    fn share_to_lan() -> ShareToLan {
        ShareToLan {
            game_mode: 0,
            commands: false,
            port: EditBox::new(0.0, 0.0, 150.0, 20.0),
            hint: 25565,
            port_error: None,
            focused_button: None,
        }
    }

    #[test]
    fn game_mode_cycles_survival_spectator_creative_adventure() {
        let mut mode = 0;
        let mut seen = vec![mode];
        for _ in 0..4 {
            mode = ShareToLan::next_mode(mode);
            seen.push(mode);
        }
        assert_eq!(seen, vec![0, 3, 1, 2, 0]);
    }

    #[test]
    fn layout_follows_share_to_lan_screen() {
        // 427x240: the cycles at y 100, the port box at y 160, Start and
        // Cancel at the bottom.
        let buttons = ShareToLan::buttons(427.0, 240.0);
        assert_eq!(buttons[0], (Button::GameMode, 58.0, 100.0, 150.0));
        assert_eq!(buttons[1], (Button::Commands, 208.0, 100.0, 150.0));
        assert_eq!(buttons[2], (Button::Start, 58.0, 212.0, 150.0));
        assert_eq!(buttons[3], (Button::Cancel, 208.0, 212.0, 150.0));
        assert_eq!(ShareToLan::port_box(427.0), (138.0, 160.0));
    }

    #[test]
    fn port_validation_follows_try_parse_port() {
        let mut s = share_to_lan();
        // Blank takes the suggestion.
        s.try_parse_port();
        assert!(s.port_error.is_none());
        assert_eq!(s.parsed_port(), 25565);
        // Below the lower bound, above the higher bound, not a number.
        for bad in ["1023", "65536", "abc"] {
            s.port.set_value(bad);
            s.try_parse_port();
            assert!(s.port_error.is_some(), "{bad} should be invalid");
            assert!(!s.active(Button::Start));
        }
        // A port already in use is unavailable: a different error than an
        // out-of-range one.
        let invalid = s.port_error.as_ref().map(|t| t.plain());
        let listener =
            std::net::TcpListener::bind(("127.0.0.1", 0)).expect("a free test port");
        let taken = listener.local_addr().expect("the bound port").port();
        s.port.set_value(&taken.to_string());
        s.try_parse_port();
        assert!(s.port_error.is_some());
        assert_ne!(s.port_error.as_ref().map(|t| t.plain()), invalid);
        assert!(!s.active(Button::Start));
        // A free port in range is fine.
        drop(listener);
        s.try_parse_port();
        assert!(s.port_error.is_none());
        assert_eq!(s.parsed_port(), taken);
        assert!(s.active(Button::Start));
    }

    #[test]
    fn the_switches_narrate_as_cycle_buttons() {
        use horizon_client::widgets::What;
        let s = share_to_lan();
        let what = |b| s.narration(b, [0.0; 4], &keys).what;
        let generic = |name: &str, value: &str| format!("options.generic_value({name}, {value})");
        assert_eq!(
            what(Button::GameMode),
            What::Cycle {
                message: generic("selectWorld.gameMode", "selectWorld.gameMode.survival"),
                next: generic("selectWorld.gameMode", "selectWorld.gameMode.spectator"),
                active: true,
            }
        );
        assert_eq!(
            what(Button::Commands),
            What::Cycle {
                message: generic("selectWorld.allowCommands", "options.off"),
                next: generic("selectWorld.allowCommands", "options.on"),
                active: true,
            }
        );
        assert!(matches!(what(Button::Start), What::Button { .. }));
    }
}
