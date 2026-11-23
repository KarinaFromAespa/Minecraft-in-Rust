//! WorldOptionsScreen and ShareToLanScreen: the two 26.1 screens behind
//! the options screen and the pause menu. World Options (from the Options
//! screen, in world only) holds the difficulty with its lock, game rules
//! and restrictions, and every button acts immediately; 26.1 has no game
//! mode or cheats row on it (26.2 added them). Open to LAN (from the pause
//! menu, single player only) is the classic ShareToLanScreen: the game
//! mode and allow-commands for other players, a port box, and Start LAN
//! World, which publishes at once. The 26.2 client replaced it with a
//! LAN switch and Apply Changes; the 26.3 client merged the two screens
//! into one. The difficulty lock asks first (PopupScreen), and
//! Restrictions opens RestrictionsScreen.
//! Parity references:
//! https://mcsrc.dev/2/26.1/net/minecraft/client/gui/screens/options/WorldOptionsScreen
//! https://mcsrc.dev/2/26.1/net/minecraft/client/gui/screens/ShareToLanScreen
use super::{Game, Screen, integrated::ServerOptions, lan::LanSettings};
use horizon_client::{
    font::Font,
    game_rules_screen::{Exit, RulesScreen},
    hud::{HudBatch, rgba},
    text::{Text, named_color},
    ui::{self, EditBox},
    widgets::{self, Narratable, join_for_narration},
};
use horizon_protocol::packets::play::{self, Serverbound as S};
use winit::{event::KeyEvent, keyboard::PhysicalKey};

/// The state of the World Options screen.
pub(super) struct WorldOptions {
    /// An external server: difficulty, game rules and restrictions go
    /// over the wire.
    remote: bool,
    /// The player is a server operator (op level >= 2), from the server
    /// thread or a permission packet; gates the difficulty and game rules
    /// buttons on an external server.
    gamemaster: bool,
    /// The cheats setting at open: the game rules button follows the
    /// commands the player has (in single player, the world's Allow
    /// Commands), not a switch — 26.1 has no cheats row on this screen.
    commands_at_open: bool,
    /// The world's difficulty, 0..=3.
    difficulty: u8,
    hardcore: bool,
    /// The level's own lock (LevelData.isDifficultyLocked).
    difficulty_locked: bool,
    view: View,
    /// InWorldGameRulesScreen, while it is open.
    rules: Option<RulesScreen>,
    focused_button: Option<WoButton>,
    popup_focus: Option<usize>,
}
/// The screen on show: the options, the lock's question over them, or the
/// RestrictionsScreen or InWorldGameRulesScreen they opened (all return to
/// the same options).
#[derive(Clone, Copy, Debug, PartialEq)]
enum View {
    Options,
    LockQuestion,
    Restrictions,
    GameRules,
}
/// The world fields a click applies: the before/after pair for
/// `apply_world_options`.
#[derive(Clone, Copy, PartialEq)]
struct WorldSnapshot {
    difficulty: u8,
    locked: bool,
}

/// The state of the Open to LAN screen.
pub(super) struct ShareToLan {
    /// The game mode for other players (`selectWorld.gameMode`), from
    /// the server's default game type at open.
    game_mode: i32,
    /// Whether other players may use commands
    /// (`selectWorld.allowCommands`), from the world's Allow Commands
    /// at open.
    commands: bool,
    /// The port edit box; empty means the hinted port.
    port: EditBox,
    /// The free port suggested while the box is empty.
    hint: u16,
    hardcore: bool,
    focused_button: Option<LanButton>,
}

/// The game mode cycle order (`GameType` values): survival, spectator,
/// creative, adventure — vanilla's `createGameModeButton` order, not the
/// numeric order.
const MODE_CYCLE: [i32; 4] = [0, 3, 1, 2];

/// The mode `step` places along the cycle (-1 back, +1 forward).
fn cycle_mode(mode: i32, step: i32) -> i32 {
    let at = MODE_CYCLE.iter().position(|m| *m == mode).unwrap_or(0);
    MODE_CYCLE[((at as i32 + step).rem_euclid(4)) as usize]
}

fn next_mode(mode: i32) -> i32 {
    cycle_mode(mode, 1)
}

/// `GameType.getName`, by id (survival, creative, adventure, spectator).
const MODES: [&str; 4] = ["survival", "creative", "adventure", "spectator"];
const DIFFICULTIES: [&str; 4] = ["peaceful", "easy", "normal", "hard"];

/// HeaderAndFooterLayout's header and footer heights.
const HEADER: f32 = 33.0;
const FOOTER: f32 = 33.0;
/// The grid's columns are 150 wide, 8 apart, rows 4 apart.
const COL_W: f32 = 150.0;
const COL_GAP: f32 = 8.0;
const ROW_STEP: f32 = 24.0;
/// ShareToLan's rows (`LinearLayout`, 8 between children, 4 inside the
/// port pair): the game mode and commands row, the other-players header,
/// the port label and box.
const MO_ROW_Y: f32 = HEADER + 28.0;
const MO_HEADER_Y: f32 = HEADER + 56.0;
const MO_PORT_LABEL_Y: f32 = HEADER + 69.0;
const MO_PORT_Y: f32 = HEADER + 82.0;
/// PopupScreen: 250 wide less its 18 pixel border each side, two buttons
/// `min((214 - 6) / 2, 150)` wide and 6 apart, rows 12 apart.
const POPUP_CONTENT: f32 = 214.0;
const POPUP_BUTTON: f32 = 104.0;
/// RestrictionsScreen's text widgets: 250 wide, padded by 4.
const RESTRICTION_WIDTH: f32 = 250.0;

#[derive(Clone, Copy, Debug, PartialEq)]
enum WoButton {
    Difficulty,
    /// DifficultyButtons.lockButton (LockIconButton).
    Lock,
    GameRules,
    Restrictions,
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum LanButton {
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

impl WorldOptions {
    /// No button keeps the focus a click gave it.
    pub(super) fn clear_click_focus(&mut self) {
        self.focused_button = None;
    }
    /// The snapshot of the world fields, for `apply_world_options`.
    fn snapshot(&self) -> WorldSnapshot {
        WorldSnapshot {
            difficulty: self.difficulty,
            locked: self.difficulty_locked,
        }
    }
    /// The grid's left edge: two 150-wide columns with an 8-pixel gap,
    /// centred.
    fn left(w: f32) -> f32 {
        (w / 2.0).floor() - 154.0
    }
    /// The grid, in vanilla order: game rules and difficulty on the first
    /// row, restrictions on the second (26.1 has no game mode or cheats
    /// row; the remote grid is the same two rows).
    fn visible_buttons(&self, w: f32) -> Vec<(WoButton, f32, f32, f32)> {
        let x = Self::left(w);
        let right = x + COL_W + COL_GAP;
        let mut buttons = vec![
            (WoButton::GameRules, x, HEADER, COL_W),
            (WoButton::Difficulty, right, HEADER, COL_W - 20.0),
            (WoButton::Lock, right + COL_W - 20.0, HEADER, 20.0),
        ];
        buttons.push((WoButton::Restrictions, x, HEADER + ROW_STEP, COL_W));
        buttons
    }
    /// The Done button: 200 wide, centred in the footer.
    fn footer(w: f32, h: f32) -> (f32, f32) {
        (((w - 200.0) / 2.0).trunc(), h - FOOTER + 6.0)
    }
    fn server_difficulty(&mut self, difficulty: u8, locked: bool) {
        // A server update is the new baseline, not a local edit. In particular,
        // the reply to a difficulty click may arrive after the screen has
        // already reopened.
        self.difficulty = difficulty;
        self.difficulty_locked = locked;
    }
    fn hit(&self, (x, y): (f32, f32), (w, h): (f32, f32)) -> Option<WoButton> {
        self.visible_buttons(w)
            .into_iter()
            .chain(std::iter::once({
                let (fx, fy) = Self::footer(w, h);
                (WoButton::Done, fx, fy, 200.0)
            }))
            .find(|(b, bx, by, bw)| self.active(*b) && ui::hovered((x, y), *bx, *by, *bw, 20.0))
            .map(|b| b.0)
    }
    fn label(&self, b: WoButton) -> Text {
        match b {
            WoButton::Lock => Text::translate("narrator.button.difficulty_lock", &[]),
            WoButton::GameRules => Text::translate("editGamerule.inGame.button", &[]),
            WoButton::Restrictions => Text::translate("restrictions_screen.button", &[]),
            WoButton::Difficulty => labelled(
                "options.difficulty",
                Text::translate(
                    &format!(
                        "options.difficulty.{}",
                        DIFFICULTIES[usize::from(self.difficulty) & 3]
                    ),
                    &[],
                ),
            ),
            WoButton::Done => Text::translate("gui.done", &[]),
        }
    }
    /// What button `b` narrates: the switches are `CycleButton`s ("<name>:
    /// <value> button" and the value a click moves to), the lock a
    /// `LockIconButton` (its message, then whether it is locked).
    fn narration(&self, b: WoButton, rect: [f32; 4], tr: super::screen_info::Tr<'_>) -> Narratable {
        let active = self.active(b);
        let labelled =
            |key: &str, value: String| tr("options.generic_value", &[&tr(key, &[]), &value]);
        let difficulty = |d: u8| {
            let name = DIFFICULTIES[usize::from(d) & 3];
            labelled(
                "options.difficulty",
                tr(&format!("options.difficulty.{name}"), &[]),
            )
        };
        let cycle = |message: String, next: String| Narratable::cycle(message, next, rect, active);
        match b {
            WoButton::Difficulty => cycle(
                difficulty(self.difficulty),
                difficulty(self.difficulty.wrapping_add(1)),
            ),
            WoButton::Lock => {
                let message = tr(
                    "gui.narrate.button",
                    &[&tr("narrator.button.difficulty_lock", &[])],
                );
                let state = if self.difficulty_locked || self.hardcore {
                    "narrator.button.difficulty_lock.locked"
                } else {
                    "narrator.button.difficulty_lock.unlocked"
                };
                Narratable::named(
                    join_for_narration(&[&message, &tr(state, &[])]),
                    rect,
                    active,
                )
            }
            _ => Narratable::button(self.label(b).plain(), rect, active),
        }
    }
    /// WorldOptionsScreen.updateButton and the update*Button methods.
    fn active(&self, b: WoButton) -> bool {
        if self.remote {
            return match b {
                WoButton::Difficulty | WoButton::Lock => {
                    self.gamemaster && !self.hardcore && !self.difficulty_locked
                }
                WoButton::GameRules => self.gamemaster,
                WoButton::Restrictions | WoButton::Done => true,
            };
        }
        match b {
            // DifficultyButtons.updateDifficultyButtonsState: the level's
            // lock or hardcore, not the wanted lock.
            WoButton::Difficulty | WoButton::Lock => !self.hardcore && !self.difficulty_locked,
            // updateButton from the player's permission: in single player
            // the world's Allow Commands (there is no cheats row here).
            WoButton::GameRules => !self.hardcore && self.commands_at_open,
            WoButton::Restrictions | WoButton::Done => true,
        }
    }
}

impl ShareToLan {
    /// The port to publish on: the box, or the hint while it is empty.
    /// Vanilla's range is 1024..65535 (`tryParsePort`); a `u16` parse
    /// caps the top, the filter the bottom. (The availability check is
    /// the bind's: a taken port fails the publish with
    /// `commands.publish.failed`.)
    fn parsed_port(&self) -> Option<u16> {
        if self.port.value.is_empty() {
            return Some(self.hint);
        }
        self.port.value.parse::<u16>().ok().filter(|p| *p >= 1024)
    }
    /// The game mode and commands row: two 150-wide columns with an
    /// 8-pixel gap.
    fn buttons(&self, w: f32) -> Vec<(LanButton, f32, f32, f32)> {
        let x = WorldOptions::left(w);
        vec![
            (LanButton::GameMode, x, MO_ROW_Y, COL_W),
            (
                LanButton::Commands,
                x + COL_W + COL_GAP,
                MO_ROW_Y,
                COL_W,
            ),
        ]
    }
    /// The footer's two buttons, centred in its 33 pixels.
    fn footer(w: f32, h: f32) -> [(LanButton, f32, f32, f32); 2] {
        let x = WorldOptions::left(w);
        [
            (LanButton::Start, x, h - FOOTER + 6.0, COL_W),
            (
                LanButton::Cancel,
                x + COL_W + COL_GAP,
                h - FOOTER + 6.0,
                COL_W,
            ),
        ]
    }
    fn hit(&self, (x, y): (f32, f32), (w, h): (f32, f32)) -> Option<LanButton> {
        self.buttons(w)
            .into_iter()
            .chain(Self::footer(w, h))
            .find(|(b, bx, by, bw)| self.active(*b) && ui::hovered((x, y), *bx, *by, *bw, 20.0))
            .map(|b| b.0)
    }
    fn label(&self, b: LanButton) -> Text {
        let mode = |m: i32| {
            Text::translate(
                &format!("selectWorld.gameMode.{}", MODES[m as usize & 3]),
                &[],
            )
        };
        match b {
            LanButton::GameMode => labelled("selectWorld.gameMode", mode(self.game_mode)),
            LanButton::Commands => {
                labelled("selectWorld.allowCommands", on_off(self.commands))
            }
            LanButton::Start => Text::translate("lanServer.start", &[]),
            LanButton::Cancel => Text::translate("gui.cancel", &[]),
        }
    }
    /// What button `b` narrates: the switches are `CycleButton`s ("<name>:
    /// <value> button" and the value a click moves to).
    fn narration(&self, b: LanButton, rect: [f32; 4], tr: super::screen_info::Tr<'_>) -> Narratable {
        let active = self.active(b);
        let mode = |m: i32| {
            tr(
                &format!("selectWorld.gameMode.{}", MODES[m as usize & 3]),
                &[],
            )
        };
        let on_off = |on: bool| tr(if on { "options.on" } else { "options.off" }, &[]);
        let labelled =
            |key: &str, value: String| tr("options.generic_value", &[&tr(key, &[]), &value]);
        let cycle = |message: String, next: String| Narratable::cycle(message, next, rect, active);
        match b {
            LanButton::GameMode => {
                let key = "selectWorld.gameMode";
                cycle(
                    labelled(key, mode(self.game_mode)),
                    labelled(key, mode(next_mode(self.game_mode))),
                )
            }
            LanButton::Commands => {
                let key = "selectWorld.allowCommands";
                cycle(
                    labelled(key, on_off(self.commands)),
                    labelled(key, on_off(!self.commands)),
                )
            }
            _ => Narratable::button(self.label(b).plain(), rect, active),
        }
    }
    /// ShareToLanScreen: hardcore locks the game mode and commands row,
    /// Start follows the port.
    fn active(&self, b: LanButton) -> bool {
        match b {
            LanButton::GameMode | LanButton::Commands => !self.hardcore,
            LanButton::Start => self.parsed_port().is_some(),
            LanButton::Cancel => true,
        }
    }
}

/// MultiLineLabel: the lines of `t` wrapped to `width`.
fn wrap(font: &Font, t: &Text, width: f32) -> Vec<Text> {
    font.split_lines(t, width)
        .into_iter()
        .map(|(line, _)| line)
        .collect()
}
fn widest(font: &Font, lines: &[Text]) -> f32 {
    lines.iter().map(|l| font.width_text(l)).fold(0.0, f32::max)
}
/// Centred MultiLineTextWidget lines: each centred on `mid` from `top`, 9 apart.
fn centred_lines(font: &Font, lines: Vec<Text>, mid: f32, top: f32) -> Vec<(Text, f32, f32)> {
    lines
        .into_iter()
        .enumerate()
        .map(|(i, line)| {
            let x = mid - (font.width_text(&line) / 2.0).trunc();
            (line, x, top + 9.0 * i as f32)
        })
        .collect()
}

/// PopupScreen centred on the screen: its title (bold) and message lines,
/// the background sprite's rectangle and its two buttons.
struct Popup {
    lines: Vec<(Text, f32, f32)>,
    frame: (f32, f32, f32, f32),
    buttons: [(f32, f32); 2],
}
impl Popup {
    fn new(font: &Font, title: &Text, message: &Text, (w, h): (f32, f32)) -> Self {
        let blocks = [
            wrap(font, title, POPUP_CONTENT),
            wrap(font, message, POPUP_CONTENT),
        ];
        let row = 2.0 * POPUP_BUTTON + 6.0;
        let lw = blocks.iter().map(|b| widest(font, b)).fold(row, f32::max);
        let lh = blocks
            .iter()
            .map(|b| 9.0 * b.len() as f32 + 12.0)
            .sum::<f32>()
            + 20.0;
        let (x, y) = (((w - lw) / 2.0).trunc(), ((h - lh) / 2.0).trunc());
        let mut lines = Vec::new();
        let mut top = y;
        for block in blocks {
            let bw = widest(font, &block);
            let mid = x + ((lw - bw) / 2.0).trunc() + (bw / 2.0).trunc();
            let n = block.len() as f32;
            lines.extend(centred_lines(font, block, mid, top));
            top += 9.0 * n + 12.0;
        }
        let bx = x + ((lw - row) / 2.0).trunc();
        Self {
            lines,
            frame: (x - 18.0, y - 18.0, lw + 36.0, lh + 36.0),
            buttons: [(bx, top), (bx + POPUP_BUTTON + 6.0, top)],
        }
    }
    /// The lock's question: WorldOptionsScreen's Apply when the lock changed.
    fn lock_question(font: &Font, difficulty: u8, size: (f32, f32)) -> Self {
        let name = Text::translate(
            &format!(
                "options.difficulty.{}",
                DIFFICULTIES[usize::from(difficulty) & 3]
            ),
            &[],
        );
        Self::new(
            font,
            &Text::translate("difficulty.lock.title", &[]).bold(),
            &Text::translate("difficulty.lock.question", &[name]),
            size,
        )
    }
}

/// RestrictionsScreen for the player of a single-player world, whose chat
/// abilities come from the Chat option alone (Minecraft.computeChatAbilities).
struct Restrictions {
    lines: Vec<(Text, f32, f32)>,
    /// The restriction's action, 200 wide: Chat Settings.
    action: Option<(f32, f32)>,
    /// The ScrollableLayout's top and bottom.
    area: (f32, f32),
}
impl Restrictions {
    fn new(font: &Font, chat_visibility: u8, (w, h): (f32, f32)) -> Self {
        let red = named_color("red").unwrap_or(0xFF5555);
        let green = named_color("green").unwrap_or(0x55FF55);
        let inner = RESTRICTION_WIDTH - 8.0;
        // ChatVisiblity HIDDEN takes chat and commands, SYSTEM the chat.
        let restriction = match chat_visibility {
            2 => Some("chat_restriction.chat_and_commands_disabled_by_options"),
            1 => Some("chat_restriction.chat_disabled_by_options"),
            _ => None,
        };
        let abilities = horizon_client::chat::ChatAbilities::new(
            horizon_client::chat::ChatVisibility::from_option(chat_visibility),
        );
        let (chat, commands) = (abilities.send_messages, abilities.send_commands);
        // The body's widgets: text blocks (lines, top) and the action's top.
        let mut blocks = Vec::new();
        let mut action = None;
        let mut y = 0.0;
        if let Some(key) = restriction {
            let lines = wrap(
                font,
                &Text::translate(key, &[]).bold().with_color(red),
                inner,
            );
            let n = lines.len() as f32;
            blocks.push((lines, y));
            y += 9.0 * n + 8.0;
            action = Some(y);
            // The button, then a 10 pixel SpacerElement.
            y += 20.0 + 10.0;
        }
        let permissions = [
            ("send_messages", chat),
            ("send_commands", commands),
            ("receive_system_messages", commands),
            ("receive_player_messages", chat),
        ];
        let lines: Vec<Text> = permissions
            .iter()
            .flat_map(|&(id, allowed)| {
                let state = if allowed { "allowed" } else { "denied" };
                let t =
                    Text::translate(&format!("restrictions_screen.permission.{id}.{state}"), &[])
                        .with_color(if allowed { green } else { red });
                wrap(font, &t, inner)
            })
            .collect();
        let n = lines.len() as f32;
        blocks.push((lines, y));
        y += 9.0 * n + 8.0;
        // HeaderAndFooterLayout: 30 under the header if it fits; the
        // ScrollableLayout keeps 10 pixels each side for its scrollbar.
        let height = y.min(h - HEADER - FOOTER);
        let top = (HEADER + 30.0).min(h - FOOTER - height);
        let x = ((w - (RESTRICTION_WIDTH + 20.0)) / 2.0).trunc() + 10.0;
        let mid = x + RESTRICTION_WIDTH / 2.0;
        Self {
            lines: blocks
                .into_iter()
                .flat_map(|(lines, by)| centred_lines(font, lines, mid, top + by + 4.0))
                .collect(),
            action: action.map(|ay| (x + 25.0, top + ay)),
            area: (top, top + height),
        }
    }
    /// The footer's Done, 200 wide.
    fn done(w: f32, h: f32) -> (f32, f32) {
        (((w - 200.0) / 2.0).trunc(), h - FOOTER + 6.0)
    }
}

impl Game<'_> {
    pub(super) fn remote_world_options_permission(&mut self, permission: bool) {
        if let Some(o) = self.world_options.as_mut() {
            let lost = o.gamemaster && !permission;
            o.gamemaster = permission;
            if lost && o.remote {
                self.close_world_options();
            }
        }
    }
    pub(super) fn remote_world_options_difficulty(&mut self, difficulty: u8, locked: bool) {
        if let Some(o) = self.world_options.as_mut() {
            o.server_difficulty(difficulty, locked);
        }
    }
    pub(super) fn remote_world_options_rules(&mut self, entries: Vec<play::GameRuleEntry>) {
        let Some(o) = self
            .world_options
            .as_mut()
            .filter(|o| o.view == View::GameRules)
        else {
            return;
        };
        // The response itself lists the server's available rules; the client
        // may not own this world's level.dat (including on the server thread).
        let features = if entries.iter().any(|e| {
            horizon_client::gamerules::rule(&e.name)
                .is_some_and(|r| r.name == horizon_client::gamerules::MAX_MINECART_SPEED.name)
        }) {
            vec![horizon_client::gamerules::MINECART_IMPROVEMENTS.to_owned()]
        } else {
            Vec::new()
        };
        let mut rules = horizon_client::gamerules::GameRules::with_features(&features);
        for entry in entries {
            rules.set_string(&entry.name, &entry.value);
        }
        o.rules = Some(RulesScreen::new(&rules));
    }
    pub(super) fn open_world_options(&mut self) {
        // The world is the server thread's when there is one.
        let state = match &self.integrated {
            Some(server) => match server.call(|g| g.server_options()) {
                Some(state) => state,
                None => return,
            },
            None => self.local_server_options(),
        };
        self.world_options = Some(WorldOptions {
            remote: self.external_server(),
            gamemaster: self.ui.registries.permission >= 2,
            commands_at_open: state.cheats,
            difficulty: state.difficulty,
            hardcore: state.hardcore,
            difficulty_locked: state.difficulty_locked,
            view: View::Options,
            rules: None,
            focused_button: None,
            popup_focus: None,
        });
        self.screen = Screen::WorldOptions;
    }

    /// World Options opens from the Options screen, and returns to it.
    pub(super) fn close_world_options(&mut self) {
        self.world_options = None;
        self.screen = Screen::Options;
    }

    pub(super) fn open_share_to_lan(&mut self) {
        let hint = horizon_client::lan::available_port();
        let mut port = EditBox::new(0.0, 0.0, 150.0, 20.0);
        port.message = super::screen_info::tr("lanServer.port", &[]);
        port.max_length = 5;
        // The world is the server thread's when there is one.
        let state = match &self.integrated {
            Some(server) => match server.call(|g| g.server_options()) {
                Some(state) => state,
                None => return,
            },
            None => self.local_server_options(),
        };
        // ShareToLanScreen.init: the game mode is the server's default
        // game type, commands the world's Allow Commands; the port box
        // starts empty (a blank port publishes on an available port).
        self.share_to_lan = Some(ShareToLan {
            game_mode: state.default_mode,
            commands: state.cheats,
            port,
            hint,
            hardcore: state.hardcore,
            focused_button: None,
        });
        self.screen = Screen::ShareToLan;
    }

    /// Open to LAN opens from the pause menu, and returns to it.
    pub(super) fn close_share_to_lan(&mut self) {
        self.share_to_lan = None;
        self.screen = Screen::Paused;
    }

    /// Escape: the question and RestrictionsScreen return to the options
    /// (their onClose), which close; the game rules ask first.
    pub(super) fn world_options_escape(&mut self) {
        match self.world_options.as_mut() {
            Some(o) if o.view == View::GameRules => {
                if o.rules.is_none() {
                    // A server may be slow to answer REQUEST_GAMERULE_VALUES.
                    o.view = View::Options;
                } else if let Some(exit) = o.rules.as_mut().and_then(RulesScreen::escape) {
                    self.close_game_rules(exit);
                }
            }
            Some(o) if o.view != View::Options => o.view = View::Options,
            _ => self.close_world_options(),
        }
    }

    /// Escape on Open to LAN closes it, discarding the changes.
    pub(super) fn share_to_lan_escape(&mut self) {
        self.close_share_to_lan();
    }

    /// InWorldGameRulesScreen's exit, back to the options. Done sends the
    /// changed rules (ServerboundSetGameRulePacket), which the server's
    /// handleSetGameRule sets for a gamemaster and tells its operators.
    fn close_game_rules(&mut self, exit: Exit) {
        if let Some(o) = self.world_options.as_mut() {
            o.view = View::Options;
            o.rules = None;
        }
        let Exit::Apply(changes) = exit else {
            return;
        };
        if self.remote.is_some() {
            if self.ui.registries.permission >= 2 && !changes.is_empty() {
                self.send(S::SetGameRule(play::SetGameRule {
                    entries: changes
                        .into_iter()
                        .map(|(name, value)| play::GameRuleEntry {
                            name: format!("minecraft:{name}"),
                            value: value.to_string(),
                        })
                        .collect(),
                }));
            }
            return;
        }
        if !self.allow_commands() || changes.is_empty() {
            return;
        }
        for (name, value) in changes {
            self.ui.rules.set(name, value);
            self.system_message(Text::tr("commands.gamerule.set", &[&name, &value]));
        }
        self.sync_rules();
    }

    pub(super) fn draw_world_options(&mut self, batch: &mut HudBatch) {
        let Some(mut o) = self.world_options.take() else {
            return;
        };
        match o.view {
            View::Options => self.draw_options_view(&mut o, batch, self.cursor),
            View::LockQuestion => {
                // PopupScreen: the screen behind without the pointer, then the popup.
                self.draw_options_view(&mut o, batch, (-1.0, -1.0));
                self.draw_lock_question(&o, batch);
            }
            View::Restrictions => self.draw_restrictions(batch),
            View::GameRules => {
                if let Some(rules) = o.rules.as_mut() {
                    rules.draw(batch, &self.gui.atlas, self.cursor);
                } else {
                    ui::menu_background(
                        batch,
                        &self.gui.atlas,
                        0.0,
                        0.0,
                        batch.width,
                        batch.height,
                        true,
                    );
                }
            }
        }
        self.world_options = Some(o);
    }

    pub(super) fn draw_share_to_lan(&mut self, batch: &mut HudBatch) {
        let Some(mut m) = self.share_to_lan.take() else {
            return;
        };
        self.draw_multiplayer_view(&mut m, batch, self.cursor);
        self.share_to_lan = Some(m);
    }

    fn draw_lock_question(&self, o: &WorldOptions, batch: &mut HudBatch) {
        // PopupScreen over WorldOptionsScreen: its title and message.
        horizon_client::widgets::begin_screen(|| {
            let name = Text::translate(
                &format!(
                    "options.difficulty.{}",
                    DIFFICULTIES[usize::from(o.difficulty) & 3]
                ),
                &[],
            );
            super::screen_info::popup(
                "world_options/lock",
                &Text::translate("difficulty.lock.title", &[]).plain(),
                &[&Text::translate("difficulty.lock.question", &[name]).plain()],
            )
        });
        let atlas = &self.gui.atlas;
        let (w, h) = (batch.width, batch.height);
        // Screen.extractTransparentBackground.
        batch.fill_gradient(
            atlas,
            (0.0, 0.0, w, h),
            rgba(0x10, 0x10, 0x10, 0xC0),
            rgba(0x10, 0x10, 0x10, 0xD0),
        );
        let p = Popup::lock_question(&atlas.font, o.difficulty, (w, h));
        let (fx, fy, fw, fh) = p.frame;
        ui::nine_slice(
            batch,
            atlas,
            "gui/sprites/popup/background",
            fx,
            fy,
            fw,
            fh,
            u32::MAX,
        );
        for (line, x, y) in &p.lines {
            batch.text_styled(atlas, line, *x, *y, u32::MAX, true);
        }
        for (i, ((bx, by), key)) in p.buttons.into_iter().zip(["gui.yes", "gui.no"]).enumerate() {
            let is_focused = o.popup_focus == Some(i);
            let hover = ui::hovered(self.cursor, bx, by, POPUP_BUTTON, 20.0) || is_focused;
            let label = Text::translate(key, &[]);
            ui::button(
                batch,
                atlas,
                bx,
                by,
                POPUP_BUTTON,
                20.0,
                &label,
                true,
                hover,
            );
        }
    }

    fn draw_restrictions(&self, batch: &mut HudBatch) {
        horizon_client::widgets::begin_screen(|| {
            horizon_client::widgets::ScreenInfo::new(
                "restrictions",
                super::screen_info::tr("restrictions_screen.title", &[]),
            )
        });
        let atlas = &self.gui.atlas;
        let (w, h) = (batch.width, batch.height);
        ui::menu_background(batch, atlas, 0.0, 0.0, w, h, true);
        let title = Text::translate("restrictions_screen.title", &[]);
        let tw = atlas.font.width_text(&title);
        batch.text_styled(
            atlas,
            &title,
            ((w - tw) / 2.0).floor(),
            12.0,
            u32::MAX,
            true,
        );
        let r = Restrictions::new(&atlas.font, self.settings.chat_visibility, (w, h));
        let first = batch.vertices.len();
        for (line, x, y) in &r.lines {
            batch.text_styled(atlas, line, *x, *y, u32::MAX, true);
        }
        if let Some((ax, ay)) = r.action {
            let is_focused = self.world_options.as_ref().and_then(|o| o.popup_focus) == Some(0);
            let hover = ui::hovered(self.cursor, ax, ay, 200.0, 20.0) || is_focused;
            let label = Text::translate("chat_restriction.chat_disabled_by_options.action", &[]);
            ui::button(batch, atlas, ax, ay, 200.0, 20.0, &label, true, hover);
        }
        batch.clip_since(first, 0.0, r.area.0, w, r.area.1);
        let (dx, dy) = Restrictions::done(w, h);
        let is_focused = self.world_options.as_ref().is_some_and(|o| {
            o.popup_focus == Some(1) || (o.popup_focus == Some(0) && r.action.is_none())
        });
        let hover = ui::hovered(self.cursor, dx, dy, 200.0, 20.0) || is_focused;
        let label = Text::translate("gui.done", &[]);
        ui::action_button(
            batch,
            atlas,
            horizon_client::widgets::Action::Done,
            dx,
            dy,
            200.0,
            20.0,
            &label,
            true,
            hover,
        );
    }

    fn draw_options_view(&self, o: &mut WorldOptions, batch: &mut HudBatch, cursor: (f32, f32)) {
        horizon_client::widgets::begin_screen(|| {
            horizon_client::widgets::ScreenInfo::new(
                "world_options",
                super::screen_info::tr("options.worldOptions.title", &[]),
            )
        });
        let atlas = &self.gui.atlas;
        let (w, h) = (batch.width, batch.height);
        ui::menu_background(batch, atlas, 0.0, 0.0, w, h, true);
        let title = Text::translate("options.worldOptions.title", &[]);
        let tw = atlas.font.width_text(&title);
        batch.text_styled(
            atlas,
            &title,
            ((w - tw) / 2.0).floor(),
            12.0,
            u32::MAX,
            true,
        );
        let inside = cursor.1 >= HEADER && cursor.1 < h - FOOTER;
        let tr = &super::screen_info::tr;
        for (b, bx, by, bw) in o.visible_buttons(w) {
            let active = o.active(b);
            let is_focused = o.focused_button == Some(b);
            let hover = active && ((inside && ui::hovered(cursor, bx, by, bw, 20.0)) || is_focused);
            if b == WoButton::Lock {
                widgets::narratable(|| o.narration(b, [bx, by, 20.0, 20.0], tr).hovered(hover));
                // An icon, but a button: the D-pad and Tab stop on it as on the others.
                if active {
                    widgets::target([bx, by, 20.0, 20.0]);
                }
                let lock = if o.difficulty_locked || o.hardcore {
                    "locked"
                } else {
                    "unlocked"
                };
                let state = match (active, hover) {
                    (false, _) => "_disabled",
                    (true, true) => "_highlighted",
                    (true, false) => "",
                };
                let sprite = format!("gui/sprites/widget/{lock}_button{state}");
                batch.sprite_scaled(atlas, &sprite, bx, by, 20.0, 20.0, u32::MAX);
                continue;
            }
            ui::button_as(
                batch,
                atlas,
                bx,
                by,
                bw,
                20.0,
                &o.label(b),
                active,
                hover,
                |rect| o.narration(b, rect, tr),
            );
        }
        ui::separators(batch, atlas, HEADER, h - FOOTER, true);
        let (fx, fy) = WorldOptions::footer(w, h);
        let is_focused = o.focused_button == Some(WoButton::Done);
        let hover = ui::hovered(cursor, fx, fy, 200.0, 20.0) || is_focused;
        ui::action_button(
            batch,
            atlas,
            horizon_client::widgets::Action::Done,
            fx,
            fy,
            200.0,
            20.0,
            &o.label(WoButton::Done),
            true,
            hover,
        );
    }

    fn draw_multiplayer_view(
        &self,
        m: &mut ShareToLan,
        batch: &mut HudBatch,
        cursor: (f32, f32),
    ) {
        horizon_client::widgets::begin_screen(|| {
            horizon_client::widgets::ScreenInfo::new(
                "share_to_lan",
                super::screen_info::tr("lanServer.title", &[]),
            )
        });
        let atlas = &self.gui.atlas;
        let (w, h) = (batch.width, batch.height);
        ui::menu_background(batch, atlas, 0.0, 0.0, w, h, true);
        let title = Text::translate("lanServer.title", &[]);
        let tw = atlas.font.width_text(&title);
        batch.text_styled(
            atlas,
            &title,
            ((w - tw) / 2.0).floor(),
            12.0,
            u32::MAX,
            true,
        );
        let inside = cursor.1 >= HEADER && cursor.1 < h - FOOTER;
        let tr = &super::screen_info::tr;
        let x = WorldOptions::left(w);
        let centre = x + 79.0;
        for (b, bx, by, bw) in m.buttons(w) {
            let active = m.active(b);
            let is_focused = m.focused_button == Some(b);
            let hover = active && ((inside && ui::hovered(cursor, bx, by, bw, 20.0)) || is_focused);
            ui::button_as(
                batch,
                atlas,
                bx,
                by,
                bw,
                20.0,
                &m.label(b),
                active,
                hover,
                |rect| m.narration(b, rect, tr),
            );
            if b == LanButton::GameMode {
                // The other-players header, plain text over the port row.
                let header = Text::translate("lanServer.otherPlayers", &[]);
                let hw = atlas.font.width_text(&header);
                let hx = ((w - hw) / 2.0).floor();
                batch.text_styled(atlas, &header, hx, MO_HEADER_Y, u32::MAX, true);
                widgets::narratable(|| {
                    Narratable::text(header.plain(), [hx, MO_HEADER_Y, hw, 9.0])
                });
                // The port label and box, under the header.
                let label = Text::translate("lanServer.port", &[]);
                batch.text_styled(atlas, &label, centre, MO_PORT_LABEL_Y, u32::MAX, true);
                m.port.x = centre;
                m.port.y = MO_PORT_Y;
                m.port.text_color = if m.parsed_port().is_some() {
                    rgba(0xE0, 0xE0, 0xE0, 255)
                } else {
                    rgba(0xDF, 0x50, 0x50, 255)
                };
                m.port.draw(batch, atlas, None);
                if m.port.value.is_empty() && !m.port.focused {
                    let hint = Text::literal(m.hint.to_string());
                    // EditBox.DEFAULT_HINT_STYLE: dark gray.
                    batch.text_styled(
                        atlas,
                        &hint,
                        m.port.text_x(),
                        m.port.text_y(),
                        rgba(0x55, 0x55, 0x55, 255),
                        true,
                    );
                }
            }
        }
        ui::separators(batch, atlas, HEADER, h - FOOTER, true);
        for (b, bx, by, bw) in ShareToLan::footer(w, h) {
            let active = m.active(b);
            let is_focused = m.focused_button == Some(b);
            let hover = active && (ui::hovered(cursor, bx, by, bw, 20.0) || is_focused);
            let action = match b {
                LanButton::Start => horizon_client::widgets::Action::Apply,
                _ => horizon_client::widgets::Action::Cancel,
            };
            ui::action_button(
                batch,
                atlas,
                action,
                bx,
                by,
                bw,
                20.0,
                &m.label(b),
                active,
                hover,
            );
        }
    }

    /// The mouse wheel over the screen: a hovered switch cycles, and the
    /// cycle applies at once like a click.
    pub(super) fn world_options_scroll(&mut self, delta: f32) {
        let (w, h) = self.gui_size();
        let stick_scroll = self.pad_stick_scroll();
        let Some(mut o) = self.world_options.take() else {
            return;
        };
        if o.view == View::Options {
            let hit_btn = o.hit(self.cursor, (w, h)).or(o.focused_button);
            if let Some(btn) = hit_btn.filter(|_| !stick_scroll) {
                let step = if delta > 0.0 { -1 } else { 1 };
                let cycled = match btn {
                    WoButton::Difficulty => {
                        let before = o.snapshot();
                        o.difficulty = ((o.difficulty as i32 + step).rem_euclid(4)) as u8;
                        let wanted = o.snapshot();
                        self.apply_world_options(&before, &wanted);
                        true
                    }
                    _ => false,
                };
                if cycled {
                    self.audio.play_button_click();
                }
            }
        } else if let Some(rules) = o.rules.as_mut() {
            rules.wheel(delta, h);
        }
        self.world_options = Some(o);
    }

    /// The mouse wheel over Open to LAN: a hovered switch cycles (Start
    /// LAN World publishes it).
    pub(super) fn share_to_lan_scroll(&mut self, delta: f32) {
        let (w, h) = self.gui_size();
        let stick_scroll = self.pad_stick_scroll();
        let Some(mut m) = self.share_to_lan.take() else {
            return;
        };
        let hit_btn = m.hit(self.cursor, (w, h)).or(m.focused_button);
        if let Some(btn) = hit_btn.filter(|_| !stick_scroll) {
            let step = if delta > 0.0 { -1 } else { 1 };
            let cycled = match btn {
                LanButton::GameMode => {
                    m.game_mode = cycle_mode(m.game_mode, step);
                    true
                }
                LanButton::Commands => {
                    m.commands = !m.commands;
                    true
                }
                _ => false,
            };
            if cycled {
                self.audio.play_button_click();
            }
        }
        self.share_to_lan = Some(m);
    }

    /// The pointer moved while a list's scrollbar is held: the game rules'
    /// list follows it (AbstractSelectionList.mouseDragged).
    pub(super) fn world_options_drag(&mut self) {
        let h = self.gui_size().1;
        if let Some(o) = self.world_options.as_mut()
            && o.view == View::GameRules
            && let Some(rules) = o.rules.as_mut()
        {
            rules.drag(self.cursor.1, h);
        }
    }

    /// The button came up: a held scrollbar lets go (onRelease).
    pub(super) fn world_options_release(&mut self) {
        if let Some(o) = self.world_options.as_mut()
            && let Some(rules) = o.rules.as_mut()
        {
            rules.release();
        }
    }

    /// A button press (click or keyboard): the world fields apply at once,
    /// the rest opens its screen. Returns true when the press closes the
    /// screen (Done).
    fn press_world_options_button(&mut self, o: &mut WorldOptions, b: WoButton) -> bool {
        match b {
            WoButton::Difficulty => {
                let before = o.snapshot();
                o.difficulty = (o.difficulty + 1) % 4;
                let wanted = o.snapshot();
                self.apply_world_options(&before, &wanted);
            }
            WoButton::Lock => o.view = View::LockQuestion,
            WoButton::Restrictions => o.view = View::Restrictions,
            WoButton::GameRules => {
                if self.remote.is_some() {
                    o.rules = None;
                    self.send(S::ClientCommand(play::ClientCommand { action: 2 }));
                } else {
                    o.rules = Some(RulesScreen::new(&self.ui.rules));
                }
                o.view = View::GameRules;
            }
            WoButton::Done => return true,
        }
        false
    }

    /// The lock question's Yes: lock the difficulty now, back to the options.
    fn lock_world_options_difficulty(&mut self, o: &mut WorldOptions) {
        let before = o.snapshot();
        o.difficulty_locked = true;
        let wanted = o.snapshot();
        self.apply_world_options(&before, &wanted);
        o.view = View::Options;
    }

    pub(super) fn world_options_click(&mut self) {
        let Some(mut o) = self.world_options.take() else {
            return;
        };
        let (w, h) = self.gui_size();
        match o.view {
            View::Options => {}
            View::GameRules => {
                let atlas = &self.gui.atlas;
                let exit = o.rules.as_mut().and_then(|rules| {
                    if rules.clicks_button(atlas, self.cursor, (w, h)) {
                        self.audio.play_button_click();
                    }
                    rules.click(atlas, self.cursor, (w, h))
                });
                self.world_options = Some(o);
                if let Some(exit) = exit {
                    self.close_game_rules(exit);
                }
                return;
            }
            View::LockQuestion => {
                let p = Popup::lock_question(&self.gui.atlas.font, o.difficulty, (w, h));
                let hit = p
                    .buttons
                    .iter()
                    .position(|&(bx, by)| ui::hovered(self.cursor, bx, by, POPUP_BUTTON, 20.0));
                if hit.is_some() {
                    self.audio.play_button_click();
                }
                match hit {
                    // Yes: lock it now, then back to the options; No: back to the options.
                    Some(0) => self.lock_world_options_difficulty(&mut o),
                    Some(_) => o.view = View::Options,
                    None => {}
                }
                self.world_options = Some(o);
                return;
            }
            View::Restrictions => {
                let r =
                    Restrictions::new(&self.gui.atlas.font, self.settings.chat_visibility, (w, h));
                let in_body = self.cursor.1 >= r.area.0 && self.cursor.1 < r.area.1;
                let action = r.action.is_some_and(|(ax, ay)| {
                    in_body && ui::hovered(self.cursor, ax, ay, 200.0, 20.0)
                });
                let (dx, dy) = Restrictions::done(w, h);
                let done = ui::hovered(self.cursor, dx, dy, 200.0, 20.0);
                if action || done {
                    self.audio.play_button_click();
                    o.view = View::Options;
                }
                self.world_options = Some(o);
                if action {
                    // ChatRestriction.Action.GO_TO_CHAT_SETTINGS: Chat Settings,
                    // which returns to the screen behind RestrictionsScreen.
                    self.open_options_at(false, horizon_client::options_ui::Page::Chat);
                    self.options_to_world_options = true;
                }
                return;
            }
        }
        let hit = o.hit(self.cursor, (w, h));
        if let Some(b) = hit {
            o.focused_button = Some(b);
            self.audio.play_button_click();
        }
        let close = hit.map(|b| self.press_world_options_button(&mut o, b));
        self.world_options = Some(o);
        if close == Some(true) {
            self.close_world_options();
        }
    }

    pub(super) fn share_to_lan_click(&mut self) {
        let Some(mut m) = self.share_to_lan.take() else {
            return;
        };
        let (w, h) = self.gui_size();
        // The port box takes focus on a click over it.
        m.port.focused = ui::hovered(
            self.cursor,
            WorldOptions::left(w) + 79.0,
            MO_PORT_Y,
            150.0,
            20.0,
        );
        let hit = m.hit(self.cursor, (w, h));
        if let Some(b) = hit {
            m.focused_button = Some(b);
            self.audio.play_button_click();
        }
        match hit {
            Some(LanButton::GameMode) => m.game_mode = next_mode(m.game_mode),
            Some(LanButton::Commands) => m.commands = !m.commands,
            Some(LanButton::Start) => return self.start_and_close_share_to_lan(m),
            Some(LanButton::Cancel) => {
                self.share_to_lan = Some(m);
                self.close_share_to_lan();
                return;
            }
            None => {}
        }
        self.share_to_lan = Some(m);
    }

    /// Start LAN World: close the screen, then publish (ShareToLanScreen's
    /// start button sets the screen to null before it publishes).
    fn start_and_close_share_to_lan(&mut self, m: ShareToLan) {
        let Some(port) = m.parsed_port() else {
            self.share_to_lan = Some(m);
            return;
        };
        let (game_mode, commands) = (m.game_mode, m.commands);
        self.close_share_to_lan();
        self.publish_to_lan(game_mode, commands, port);
    }

    /// `ShareToLanScreen.sendPublishMessage`: a client system message the
    /// narrator says.
    fn send_publish_message(&mut self, message: Text) {
        self.say_system_queued(&message);
        self.system_message(message);
    }

    pub(super) fn world_options_key(&mut self, event: &KeyEvent) {
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
        let chat_vis = self.settings.chat_visibility;
        let Some(o) = self.world_options.as_mut() else {
            return;
        };
        if let Some(rules) = o.rules.as_mut().filter(|r| r.typing()) {
            match shortcut {
                Some(shortcut) => rules.edit(|b| {
                    b.shortcut(shortcut, &mut self.clipboard);
                }),
                None => {
                    let mut handled = false;
                    rules.edit(|b| handled = b.key(key, ctrl, false));
                    // Only a key with text types: a `None` text is a
                    // backspace to `world_options_text`, which is the
                    // control socket's `\b`, not a key's.
                    if !handled
                        && types_text
                        && let Some(text) = event.text.as_deref()
                    {
                        self.world_options_text(Some(text));
                    }
                }
            }
            return;
        }
        if o.view == View::LockQuestion {
            match key {
                winit::keyboard::KeyCode::Tab => {
                    let reverse = shift;
                    o.popup_focus = match o.popup_focus {
                        Some(0) => Some(1),
                        Some(1) => Some(0),
                        _ => {
                            if reverse {
                                Some(1)
                            } else {
                                Some(0)
                            }
                        }
                    };
                }
                winit::keyboard::KeyCode::ArrowLeft | winit::keyboard::KeyCode::ArrowUp => {
                    o.popup_focus = Some(0)
                }
                winit::keyboard::KeyCode::ArrowRight | winit::keyboard::KeyCode::ArrowDown => {
                    o.popup_focus = Some(1)
                }
                winit::keyboard::KeyCode::Enter
                | winit::keyboard::KeyCode::NumpadEnter
                | winit::keyboard::KeyCode::Space => {
                    self.audio.play_button_click();
                    let yes = o.popup_focus == Some(0);
                    let mut o = self.world_options.take().unwrap();
                    if yes {
                        self.lock_world_options_difficulty(&mut o);
                    } else {
                        o.view = View::Options;
                    }
                    self.world_options = Some(o);
                }
                _ => {}
            }
            return;
        }
        if o.view == View::Restrictions {
            let r = Restrictions::new(&self.gui.atlas.font, chat_vis, (w, h));
            match key {
                winit::keyboard::KeyCode::Tab => {
                    let reverse = shift;
                    if r.action.is_some() {
                        o.popup_focus = match o.popup_focus {
                            Some(0) => Some(1),
                            Some(1) => Some(0),
                            _ => {
                                if reverse {
                                    Some(1)
                                } else {
                                    Some(0)
                                }
                            }
                        };
                    } else {
                        o.popup_focus = Some(1);
                    }
                }
                winit::keyboard::KeyCode::ArrowUp => {
                    if r.action.is_some() {
                        o.popup_focus = Some(0);
                    } else {
                        o.popup_focus = Some(1);
                    }
                }
                winit::keyboard::KeyCode::ArrowDown => o.popup_focus = Some(1),
                winit::keyboard::KeyCode::Enter
                | winit::keyboard::KeyCode::NumpadEnter
                | winit::keyboard::KeyCode::Space => {
                    self.audio.play_button_click();
                    let action = r.action.is_some() && o.popup_focus == Some(0);
                    o.view = View::Options;
                    if action {
                        self.open_options_at(false, horizon_client::options_ui::Page::Chat);
                        self.options_to_world_options = true;
                    }
                }
                _ => {}
            }
            return;
        }
        if o.view != View::Options {
            return;
        }
        let visible = o.visible_buttons(w);
        let (fx, fy) = WorldOptions::footer(w, h);
        let active_buttons: Vec<WoButton> = visible
            .iter()
            .filter(|b| o.active(b.0))
            .map(|b| b.0)
            .chain(std::iter::once(WoButton::Done))
            .collect();
        if active_buttons.is_empty() {
            return;
        }
        match key {
            winit::keyboard::KeyCode::Tab => {
                let reverse = shift;
                let next = match o.focused_button {
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
                o.focused_button = next;
            }
            winit::keyboard::KeyCode::ArrowUp
            | winit::keyboard::KeyCode::ArrowDown
            | winit::keyboard::KeyCode::ArrowLeft
            | winit::keyboard::KeyCode::ArrowRight => {
                let rects: Vec<(WoButton, f32, f32, f32)> = visible
                    .into_iter()
                    .filter(|b| o.active(b.0))
                    .chain(std::iter::once((WoButton::Done, fx, fy, 200.0)))
                    .collect();
                let cur_b = o.focused_button.unwrap_or(active_buttons[0]);
                if let Some(cur) = rects.iter().find(|r| r.0 == cur_b) {
                    let (cx, cy) = (cur.1 + cur.3 / 2.0, cur.2 + 10.0);
                    let mut best: Option<(WoButton, f32)> = None;
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
                        o.focused_button = Some(next_b);
                    }
                } else {
                    o.focused_button = Some(active_buttons[0]);
                }
            }
            winit::keyboard::KeyCode::Enter
            | winit::keyboard::KeyCode::NumpadEnter
            | winit::keyboard::KeyCode::Space => {
                if let Some(btn) = o.focused_button {
                    self.audio.play_button_click();
                    let mut o = self.world_options.take().unwrap();
                    let close = self.press_world_options_button(&mut o, btn);
                    self.world_options = Some(o);
                    if close {
                        self.close_world_options();
                    }
                }
            }
            _ => {}
        }
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
        let Some(m) = self.share_to_lan.as_mut() else {
            return;
        };
        if m.port.focused {
            if key == winit::keyboard::KeyCode::Tab {
                m.port.focused = false;
                if shift {
                    m.focused_button = Some(LanButton::Commands);
                } else {
                    m.focused_button = Some(LanButton::Start);
                }
                return;
            }
            if let Some(shortcut) = shortcut {
                m.port.shortcut(shortcut, &mut self.clipboard);
                return;
            }
            // Only a key with text types (a `None` is a backspace to
            // `share_to_lan_text`, the control socket's `\b`).
            if !m.port.key(key, ctrl, false)
                && types_text
                && let Some(text) = event.text.as_deref()
            {
                self.share_to_lan_text(Some(text));
            }
            return;
        }
        let visible = m.buttons(w);
        let footer = ShareToLan::footer(w, h);
        let active_buttons: Vec<LanButton> = visible
            .iter()
            .filter(|b| m.active(b.0))
            .map(|b| b.0)
            .chain(footer.iter().filter(|b| m.active(b.0)).map(|b| b.0))
            .collect();
        if active_buttons.is_empty() {
            return;
        }
        match key {
            winit::keyboard::KeyCode::Tab => {
                let reverse = shift;
                let next = match m.focused_button {
                    Some(cur) => {
                        // Tab order: the mode and commands row, the port
                        // box, the footer.
                        if !reverse && cur == LanButton::Commands {
                            m.port.focused = true;
                            m.focused_button = None;
                            return;
                        }
                        if reverse && cur == LanButton::Start {
                            m.port.focused = true;
                            m.focused_button = None;
                            return;
                        }
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
                m.focused_button = next;
            }
            winit::keyboard::KeyCode::ArrowUp
            | winit::keyboard::KeyCode::ArrowDown
            | winit::keyboard::KeyCode::ArrowLeft
            | winit::keyboard::KeyCode::ArrowRight => {
                let rects: Vec<(LanButton, f32, f32, f32)> = visible
                    .into_iter()
                    .filter(|b| m.active(b.0))
                    .chain(footer.into_iter().filter(|b| m.active(b.0)))
                    .collect();
                let cur_b = m.focused_button.unwrap_or(active_buttons[0]);
                if let Some(cur) = rects.iter().find(|r| r.0 == cur_b) {
                    let (cx, cy) = (cur.1 + cur.3 / 2.0, cur.2 + 10.0);
                    let mut best: Option<(LanButton, f32)> = None;
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
                        m.focused_button = Some(next_b);
                    }
                } else {
                    m.focused_button = Some(active_buttons[0]);
                }
            }
            winit::keyboard::KeyCode::Enter
            | winit::keyboard::KeyCode::NumpadEnter
            | winit::keyboard::KeyCode::Space => {
                if let Some(btn) = m.focused_button {
                    self.audio.play_button_click();
                    let mut m = self.share_to_lan.take().unwrap();
                    match btn {
                        LanButton::GameMode => m.game_mode = next_mode(m.game_mode),
                        LanButton::Commands => m.commands = !m.commands,
                        LanButton::Start => return self.start_and_close_share_to_lan(m),
                        LanButton::Cancel => {
                            self.share_to_lan = Some(m);
                            self.close_share_to_lan();
                            return;
                        }
                    }
                    self.share_to_lan = Some(m);
                }
            }
            _ => {}
        }
    }
    /// Whether a World Options text box has focus (a game rule box).
    pub(super) fn world_options_typing(&self) -> bool {
        self.world_options
            .as_ref()
            .is_some_and(|o| o.rules.as_ref().is_some_and(|r| r.typing()))
    }
    /// Text typed into a game rule box (the keyboard, or a controller's
    /// on-screen keyboard), and its backspace.
    pub(super) fn world_options_text(&mut self, text: Option<&str>) {
        let Some(rules) = self
            .world_options
            .as_mut()
            .and_then(|o| o.rules.as_mut())
            .filter(|r| r.typing())
        else {
            return;
        };
        match text {
            Some(text) => {
                let typed: String = text.chars().filter(|c| !c.is_control()).collect();
                if !typed.is_empty() {
                    rules.edit(|b| b.insert_text(&typed));
                }
            }
            None => rules.edit(|b| {
                b.key(winit::keyboard::KeyCode::Backspace, false, false);
            }),
        }
    }
    /// Whether the Open to LAN port box has focus.
    pub(super) fn share_to_lan_typing(&self) -> bool {
        self.share_to_lan
            .as_ref()
            .is_some_and(|m| m.port.focused)
    }
    /// Digits typed into the port field (the keyboard, or a controller's
    /// on-screen keyboard), and its backspace.
    pub(super) fn share_to_lan_text(&mut self, text: Option<&str>) {
        let Some(m) = self.share_to_lan.as_mut().filter(|m| m.port.focused) else {
            return;
        };
        match text {
            Some(text) => {
                let digits: String = text.chars().filter(char::is_ascii_digit).collect();
                if !digits.is_empty() {
                    m.port.insert_text(&digits);
                }
            }
            None => {
                m.port
                    .key(winit::keyboard::KeyCode::Backspace, false, false);
            }
        }
    }

    /// The world options of a world run in the frame.
    fn local_server_options(&self) -> ServerOptions {
        let (guest_commands, force) = self.lan.as_ref().map_or((false, true), |h| {
            (h.settings.guest_commands, h.settings.force_game_mode)
        });
        ServerOptions {
            default_mode: self.level.mode(),
            personal_mode: self.mode_id(),
            cheats: self.allow_commands(),
            difficulty: self.ui.difficulty,
            hardcore: self.level.hardcore,
            difficulty_locked: self.level.difficulty_locked,
            port: self.lan_port(),
            guest_commands,
            force,
        }
    }

    fn apply_world_options(&mut self, before: &WorldSnapshot, wanted: &WorldSnapshot) {
        if self.external_server() {
            if self.ui.registries.permission >= 2 {
                if wanted.difficulty != before.difficulty {
                    self.send(S::ChangeDifficulty(play::ChangeDifficultyRequest {
                        difficulty: i32::from(wanted.difficulty),
                    }));
                }
                if wanted.locked && !before.locked {
                    self.send(S::LockDifficulty(play::LockDifficulty { locked: true }));
                }
            }
            return;
        }
        if let Some(server) = &self.integrated {
            // The server applies them.
            let (b, w) = (*before, *wanted);
            server.call(move |g| g.apply_world_options(&b, &w));
            return;
        }
        if wanted.difficulty != before.difficulty {
            let name = DIFFICULTIES[usize::from(wanted.difficulty) & 3];
            self.as_console(|g| g.run_command(&format!("difficulty {name}")));
        }
        // ServerboundLockDifficultyPacket(true): MinecraftServer.setDifficultyLocked.
        if wanted.locked && !before.locked {
            self.level.difficulty_locked = true;
        }
    }

    /// `IntegratedServer.publishServer(gameType, allowCommands, port)`:
    /// the game type for joining players, commands for all players, then
    /// the listener. 26.1 has no force toggle; the setting is kept as it
    /// is. The started message comes from `lan_open`; a failed bind says
    /// `commands.publish.failed`.
    fn publish_to_lan(&mut self, game_mode: i32, commands: bool, port: u16) {
        let force = self
            .lan
            .as_ref()
            .map_or(true, |h| h.settings.force_game_mode);
        self.set_published_game_type(game_mode);
        self.lan_apply_settings(commands, force);
        let opened = self.lan_open(LanSettings {
            port,
            guest_commands: commands,
            force_game_mode: force,
        });
        if opened.is_err() {
            self.send_publish_message(Text::translate("commands.publish.failed", &[]));
        }
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

    #[test]
    fn the_switches_narrate_as_cycle_buttons_and_the_lock_as_a_lock() {
        use horizon_client::widgets::What;
        let o = options(true, false);
        let what = |b| o.narration(b, [0.0; 4], &keys).what;
        let generic = |name: &str, value: &str| format!("options.generic_value({name}, {value})");
        assert_eq!(
            what(WoButton::Difficulty),
            What::Cycle {
                message: generic("options.difficulty", "options.difficulty.normal"),
                next: generic("options.difficulty", "options.difficulty.hard"),
                active: true,
            }
        );
        assert_eq!(
            what(WoButton::Lock),
            What::Named {
                title: "gui.narrate.button(narrator.button.difficulty_lock). narrator.button.difficulty_lock.unlocked"
                    .into(),
                active: true,
            }
        );
        let mut locked = options(true, false);
        locked.difficulty_locked = true;
        assert_eq!(
            locked.narration(WoButton::Lock, [0.0; 4], &keys).what,
            What::Named {
                title: "gui.narrate.button(narrator.button.difficulty_lock). narrator.button.difficulty_lock.locked"
                    .into(),
                active: false,
            }
        );
        assert!(matches!(what(WoButton::Done), What::Button { .. }));
    }

    #[test]
    fn the_game_mode_cycle_follows_vanillas_order() {
        // createGameModeButton: survival, spectator, creative, adventure.
        assert_eq!(next_mode(0), 3);
        assert_eq!(next_mode(3), 1);
        assert_eq!(next_mode(1), 2);
        assert_eq!(next_mode(2), 0);
        assert_eq!(cycle_mode(0, -1), 2);
        assert_eq!(cycle_mode(3, -1), 0);
    }

    fn options(cheats: bool, hardcore: bool) -> WorldOptions {
        WorldOptions {
            remote: false,
            gamemaster: cheats,
            commands_at_open: cheats,
            difficulty: 2,
            hardcore,
            difficulty_locked: false,
            view: View::Options,
            rules: None,
            focused_button: None,
            popup_focus: None,
        }
    }

    fn share_to_lan(hardcore: bool) -> ShareToLan {
        ShareToLan {
            game_mode: 0,
            commands: false,
            port: EditBox::new(0.0, 0.0, 150.0, 20.0),
            hint: 25565,
            hardcore,
            focused_button: None,
        }
    }

    #[test]
    fn layout_follows_the_header_footer_and_grid() {
        let at = |b: WoButton| {
            let o = options(true, false);
            let (fx, fy) = WorldOptions::footer(427.0, 240.0);
            o.visible_buttons(427.0)
                .into_iter()
                .chain(std::iter::once((WoButton::Done, fx, fy, 200.0)))
                .find(|e| e.0 == b)
                .map(|(_, x, y, w)| (x, y, w))
                .unwrap()
        };
        // Header 33, two 150-wide columns 8 apart, rows 4 apart.
        assert_eq!(at(WoButton::GameRules), (59.0, 33.0, 150.0));
        assert_eq!(at(WoButton::Difficulty), (217.0, 33.0, 130.0));
        assert_eq!(at(WoButton::Lock), (347.0, 33.0, 20.0));
        // 26.1 has no game mode or cheats row; restrictions is the second row.
        assert_eq!(at(WoButton::Restrictions), (59.0, 57.0, 150.0));
        // Done: 200 wide, centred in the footer.
        assert_eq!(at(WoButton::Done), (113.0, 213.0, 200.0));
    }

    #[test]
    fn open_to_lan_layout_follows_vanilla() {
        let m = share_to_lan(false);
        let at = |b: LanButton| {
            m.buttons(427.0)
                .into_iter()
                .chain(ShareToLan::footer(427.0, 240.0))
                .find(|e| e.0 == b)
                .map(|(_, x, y, w)| (x, y, w))
                .unwrap()
        };
        // The mode and commands row, two 150-wide columns 8 apart.
        assert_eq!(at(LanButton::GameMode), (59.0, 61.0, 150.0));
        assert_eq!(at(LanButton::Commands), (217.0, 61.0, 150.0));
        assert_eq!(at(LanButton::Start), (59.0, 213.0, 150.0));
        assert_eq!(at(LanButton::Cancel), (217.0, 213.0, 150.0));
        // LinearLayout, 8 between children, 4 inside the port pair.
        assert_eq!(MO_ROW_Y, 61.0);
        assert_eq!(MO_HEADER_Y, 89.0);
        assert_eq!(MO_PORT_LABEL_Y, 102.0);
        assert_eq!(MO_PORT_Y, 115.0);
    }

    #[test]
    fn buttons_follow_commands_and_hardcore() {
        let o = options(true, false);
        assert!(o.active(WoButton::Difficulty));
        assert!(o.active(WoButton::GameRules));
        let o = options(false, false);
        // Game Rules follows the commands the player has at open.
        assert!(!o.active(WoButton::GameRules));
        let o = options(true, true);
        assert!(!o.active(WoButton::GameRules));
        let o = options(false, true);
        assert!(!o.active(WoButton::Difficulty));
        // Above the grid, nothing takes a click; the restrictions row does.
        let o = options(true, false);
        assert_eq!(o.hit((100.0, 30.0), (427.0, 240.0)), None);
        assert_eq!(
            o.hit((100.0, 70.0), (427.0, 240.0)),
            Some(WoButton::Restrictions)
        );
    }

    #[test]
    fn the_lock_follows_the_level() {
        let o = options(true, false);
        assert!(o.active(WoButton::Lock) && o.active(WoButton::Restrictions));
        // A locked level: neither the difficulty nor the lock.
        let mut o = options(true, false);
        o.difficulty_locked = true;
        assert!(!o.active(WoButton::Lock) && !o.active(WoButton::Difficulty));
        // Hardcore shows the lock closed, and inactive.
        let o = options(true, true);
        assert!(!o.active(WoButton::Lock));
    }

    #[test]
    fn remote_options_follow_vanilla_guest_layout_and_permissions() {
        let mut o = options(false, false);
        o.remote = true;
        // Vanilla omits host-only rows instead of disabling the whole screen.
        let rows = o.visible_buttons(427.0);
        assert_eq!(
            rows.iter().map(|b| b.0).collect::<Vec<_>>(),
            vec![
                WoButton::GameRules,
                WoButton::Difficulty,
                WoButton::Lock,
                WoButton::Restrictions,
            ]
        );
        assert_eq!(rows[0], (WoButton::GameRules, 59.0, 33.0, 150.0));
        assert_eq!(rows[1], (WoButton::Difficulty, 217.0, 33.0, 130.0));
        assert_eq!(rows[2], (WoButton::Lock, 347.0, 33.0, 20.0));
        assert_eq!(rows[3], (WoButton::Restrictions, 59.0, 57.0, 150.0));
        // A non-operator cannot activate the difficulty or rule controls.
        assert_eq!(o.hit((100.0, 40.0), (427.0, 240.0)), None);
        assert_eq!(o.hit((240.0, 40.0), (427.0, 240.0)), None);
        assert_eq!(
            o.hit((100.0, 70.0), (427.0, 240.0)),
            Some(WoButton::Restrictions)
        );
        o.gamemaster = true;
        assert_eq!(
            o.hit((100.0, 40.0), (427.0, 240.0)),
            Some(WoButton::GameRules)
        );
        assert_eq!(
            o.hit((240.0, 40.0), (427.0, 240.0)),
            Some(WoButton::Difficulty)
        );
        o.difficulty_locked = true;
        assert_eq!(o.hit((240.0, 40.0), (427.0, 240.0)), None);
        // Remote hardcore does not hide an operator's Game Rules screen.
        o.hardcore = true;
        assert!(o.active(WoButton::GameRules));
        assert!(!o.active(WoButton::Lock));
    }

    #[test]
    fn server_difficulty_updates_the_level_state() {
        let mut o = options(false, false);
        o.remote = true;
        o.gamemaster = true;
        // Reopened while the previous click's reply was still in flight.
        o.server_difficulty(3, false);
        assert_eq!(o.difficulty, 3);
        assert!(o.active(WoButton::Difficulty));
        o.server_difficulty(3, true);
        assert!(o.difficulty_locked);
        assert!(!o.active(WoButton::Difficulty));
        assert!(!o.active(WoButton::Lock));
    }

    #[test]
    fn the_port_must_be_in_vanillas_range() {
        let mut m = share_to_lan(false);
        // Empty means the hint.
        assert_eq!(m.parsed_port(), Some(25565));
        m.port.set_value("1023");
        assert_eq!(m.parsed_port(), None);
        m.port.set_value("1024");
        assert_eq!(m.parsed_port(), Some(1024));
        m.port.set_value("65535");
        assert_eq!(m.parsed_port(), Some(65535));
        m.port.set_value("65536");
        assert_eq!(m.parsed_port(), None);
        m.port.set_value("abc");
        assert_eq!(m.parsed_port(), None);
    }

    #[test]
    fn start_follows_the_port() {
        let mut m = share_to_lan(false);
        // An empty box means the hint, so Start is live at open.
        assert!(m.active(LanButton::Start) && m.active(LanButton::Cancel));
        // An invalid port disables Start until it parses again.
        m.port.set_value("80");
        assert!(!m.active(LanButton::Start));
        m.port.set_value("25566");
        assert!(m.active(LanButton::Start));
        // Hardcore locks the game mode and commands row, not Start.
        let m = share_to_lan(true);
        assert!(!m.active(LanButton::GameMode) && !m.active(LanButton::Commands));
        assert!(m.active(LanButton::Start));
    }

    #[test]
    fn the_share_to_lan_switches_narrate_as_cycle_buttons() {
        use horizon_client::widgets::What;
        let m = share_to_lan(false);
        let what = |b| m.narration(b, [0.0; 4], &keys).what;
        let generic = |name: &str, value: &str| format!("options.generic_value({name}, {value})");
        assert_eq!(
            what(LanButton::GameMode),
            What::Cycle {
                message: generic("selectWorld.gameMode", "selectWorld.gameMode.survival"),
                next: generic("selectWorld.gameMode", "selectWorld.gameMode.spectator"),
                active: true,
            }
        );
        assert_eq!(
            what(LanButton::Commands),
            What::Cycle {
                message: generic("selectWorld.allowCommands", "options.off"),
                next: generic("selectWorld.allowCommands", "options.on"),
                active: true,
            }
        );
        assert!(matches!(what(LanButton::Start), What::Button { .. }));
    }
}
