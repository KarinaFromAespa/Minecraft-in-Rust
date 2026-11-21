//! EditGameRulesScreen: the rules by category, a switch or a number box
//! each, over Create New World's copy of the rules. Done hands the edited
//! rules back whole (`exitCallback.accept(Optional.of(gameRules))`);
//! Cancel and Escape discard them (`Optional.empty()`), without asking.
//! 1.21.11 has no search box, no in-world screen and no game-rule packets —
//! the search, the InWorldGameRulesScreen split and the 36-pixel header are
//! all 26.1 additions.
use crate::{
    font::Font,
    gamerules::{self, GameRules, Rule, Value},
    hud::{HudAtlas, HudBatch},
    lang,
    text::{Text, named_color},
    ui::{self, EditBox},
};

/// HeaderAndFooterLayout's default heights (33 each): the title alone in
/// the header, centred.
const HEADER: f32 = 33.0;
const FOOTER: f32 = 33.0;
/// RuleList: 24 pixel entries in AbstractSelectionList's 220 pixel rows.
const ROW: f32 = 24.0;
const ROW_WIDTH: f32 = 220.0;
/// EditBox text colours (`hud::rgba` order, ABGR): IntegerRuleEntry's valid
/// (-2039584) and invalid (-65536).
const VALID: u32 = 0xFFE0_E0E0;
const INVALID: u32 = 0xFF00_00FF;

#[derive(Clone, Copy)]
enum Row {
    Category(&'static str),
    Rule(&'static Rule),
}

/// What a click or Escape asks of Create New World.
#[derive(Debug, PartialEq)]
pub enum Exit {
    /// Done: the edited rules, whole.
    Apply(GameRules),
    /// Cancel, or Escape: the changes are discarded.
    Discard,
}

pub struct RulesScreen {
    values: GameRules,
    /// IntegerRuleEntry's boxes, by rule.
    numbers: Vec<(&'static str, EditBox)>,
    /// RuleList's scrolling: the wheel and the held scrollbar.
    scroll: ui::Scroll,
}

fn name_key(r: &Rule) -> String {
    format!("gamerule.minecraft.{}", r.name)
}
fn category_key(category: &str) -> String {
    format!("gamerule.category.minecraft.{category}")
}
/// GameRule.deserialize of an integer rule: Brigadier's readInt (digits,
/// '.' and '-' only, nothing after), then the rule's range.
fn parse(r: &Rule, s: &str) -> Option<i32> {
    let number = !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_digit() || c == '-' || c == '.');
    number
        .then(|| s.parse::<i32>().ok())
        .flatten()
        .filter(|v| (r.min..=r.max).contains(v))
}
fn lines(font: &Font, t: &Text, width: f32) -> Vec<Text> {
    font.split_lines(t, width)
        .into_iter()
        .map(|(line, _)| line)
        .collect()
}
fn row_left(w: f32) -> f32 {
    (w / 2.0).trunc() - ROW_WIDTH / 2.0
}
/// A rule's switch or box: 44 by 20, 45 from the entry content's right.
fn control_x(w: f32) -> f32 {
    row_left(w) + 2.0 + (ROW_WIDTH - 4.0) - 45.0
}
/// The footer's Done and Cancel, 150 wide and 8 apart.
fn footer(w: f32, h: f32) -> [(f32, f32); 2] {
    let x = ((w - 308.0) / 2.0).trunc();
    [(x, h - FOOTER + 6.0), (x + 158.0, h - FOOTER + 6.0)]
}
fn yellow() -> u32 {
    named_color("yellow").unwrap_or(0xFFFF55)
}
/// RuleList.addEntry's `narration`: the rule's description, when it has
/// one, over its default value.
fn narration(r: &Rule) -> String {
    let default = Text::translate(
        "editGamerule.default",
        &[Text::literal(r.default.to_string())],
    )
    .plain();
    let description = format!("{}.description", name_key(r));
    if lang::has(&description) {
        format!("{}\n{default}", lang::get(&description))
    } else {
        default
    }
}

/// BooleanRuleEntry's CycleButton (`displayOnlyValue`, its own narration):
/// "<name>: <value> button", the rule's description and default, and the
/// value a click moves to.
fn bool_narration(r: &Rule, value: bool, rect: [f32; 4]) -> crate::widgets::Narratable {
    let tr = crate::screen_narrator::translate;
    let on_off = |on: bool| tr(if on { "options.on" } else { "options.off" }, &[]);
    let full = tr(
        "options.generic_value",
        &[&lang::get(&name_key(r)), &on_off(value)],
    );
    let title = format!("{}\n{}", tr("gui.narrate.button", &[&full]), narration(r));
    crate::widgets::Narratable::new(
        crate::widgets::What::NamedCycle {
            title,
            next: on_off(!value),
            active: true,
        },
        rect,
    )
    .without_tooltip()
}

impl RulesScreen {
    /// EditGameRulesScreen over the new world's rules.
    pub fn new(rules: &GameRules) -> Self {
        let numbers = rules
            .available_rules()
            .filter(|r| matches!(r.default, Value::Int(_)))
            .map(|r| {
                let mut b = EditBox::new(0.0, 0.0, 44.0, 20.0);
                b.focused = false;
                b.set_value(&rules.int(r.name).to_string());
                b.text_color = VALID;
                (r.name, b)
            })
            .collect();
        Self {
            values: rules.clone(),
            numbers,
            scroll: ui::Scroll::default(),
        }
    }

    /// RuleList: every rule, by category, each sorted by description id.
    fn rows(&self) -> Vec<Row> {
        let mut categories: Vec<(&'static str, Vec<&'static Rule>)> = Vec::new();
        for r in self.values.available_rules() {
            match categories.iter_mut().find(|(c, _)| *c == r.category) {
                Some((_, rules)) => rules.push(r),
                None => categories.push((r.category, vec![r])),
            }
        }
        // By the categories' ids, then the rules' description ids.
        categories.sort_by_key(|(c, _)| *c);
        categories
            .into_iter()
            .flat_map(|(c, mut rules)| {
                rules.sort_by_key(|r| name_key(r));
                std::iter::once(Row::Category(c)).chain(rules.into_iter().map(Row::Rule))
            })
            .collect()
    }
    /// Entry `i`'s top: 2 under the list's top, less the scroll.
    fn row_top(&self, i: usize) -> f32 {
        HEADER + 2.0 - self.scroll.amount.trunc() + ROW * i as f32
    }
    /// The list's scrollbar column (AbstractSelectionList: 6 pixels right of
    /// the entries) and the pixels it scrolls.
    fn bar(w: f32) -> f32 {
        row_left(w) + ROW_WIDTH + 6.0 + 2.0
    }
    fn content(&self) -> f32 {
        ROW * self.rows().len() as f32 + 4.0
    }
    /// getEntryAtPosition: the row under the pointer, inside the list.
    fn hovered(&self, rows: usize, (x, y): (f32, f32), (w, h): (f32, f32)) -> Option<usize> {
        let left = row_left(w);
        let inside = y >= HEADER && y < h - FOOTER && x >= left && x < left + ROW_WIDTH;
        let i = ((y - self.row_top(0)) / ROW).floor();
        (inside && i >= 0.0 && (i as usize) < rows).then_some(i as usize)
    }
    fn number(&mut self, name: &str) -> Option<&mut EditBox> {
        self.numbers
            .iter_mut()
            .find(|(n, _)| *n == name)
            .map(|(_, b)| b)
    }
    /// An invalid number disables Done (markInvalid).
    fn invalid(&self) -> bool {
        self.numbers
            .iter()
            .any(|(name, b)| gamerules::rule(name).is_none_or(|r| parse(r, &b.value).is_none()))
    }
    /// Escape: onClose discards the changes, without asking.
    pub fn escape(&mut self) -> Option<Exit> {
        Some(Exit::Discard)
    }
    /// The mouse wheel (AbstractSelectionList: half an entry a step).
    pub fn wheel(&mut self, delta: f32, h: f32) {
        let content = self.content();
        self.scroll
            .wheel(delta, ROW / 2.0, HEADER, h - FOOTER, content);
    }
    /// A press on the scrollbar holds it (AbstractSelectionList.updateScrolling).
    pub fn press_scrollbar(&mut self, cursor: (f32, f32), (w, h): (f32, f32)) -> bool {
        let content = self.content();
        self.scroll
            .press(cursor, Self::bar(w), HEADER, h - FOOTER, content)
    }
    /// The pointer moved while the scrollbar is held.
    pub fn drag(&mut self, y: f32, h: f32) {
        if self.scroll.held() {
            let content = self.content();
            self.scroll.drag(y, HEADER, h - FOOTER, content);
        }
    }
    /// The button came up: the scrollbar lets go.
    pub fn release(&mut self) {
        self.scroll.release();
    }
    fn focused(&mut self) -> Option<(&'static str, &mut EditBox)> {
        self.numbers
            .iter_mut()
            .find(|(_, b)| b.focused)
            .map(|(n, b)| (*n, b))
    }
    /// A number sets its rule when valid (IntegerRuleEntry's responder).
    fn edited(&mut self, rule: &'static str) {
        let Some(r) = gamerules::rule(rule) else {
            return;
        };
        let value = self.number(rule).map(|b| parse(r, &b.value));
        if let Some(Some(v)) = value {
            self.values.set(rule, Value::Int(v));
        }
        if let Some(b) = self.number(rule) {
            b.text_color = if matches!(value, Some(Some(_))) {
                VALID
            } else {
                INVALID
            };
        }
    }
    /// Run an edit on the focused box, then its responder.
    pub fn edit(&mut self, f: impl FnOnce(&mut EditBox)) {
        let Some((rule, b)) = self.focused() else {
            return;
        };
        let before = b.value.clone();
        f(b);
        if b.value != before {
            self.edited(rule);
        }
    }
    pub fn typing(&self) -> bool {
        self.numbers.iter().any(|(_, b)| b.focused)
    }

    /// A left click; Some when the screen closes.
    pub fn click(
        &mut self,
        atlas: &HudAtlas,
        cursor: (f32, f32),
        size: (f32, f32),
    ) -> Option<Exit> {
        let (w, h) = size;
        let on = |x: f32, y: f32, bw: f32, bh: f32| ui::hovered(cursor, x, y, bw, bh);
        let [(dx, dy), (cx, cy)] = footer(w, h);
        if on(dx, dy, 150.0, 20.0) && !self.invalid() {
            return Some(Exit::Apply(self.values.clone()));
        }
        if on(cx, cy, 150.0, 20.0) {
            return Some(Exit::Discard);
        }
        // A press on the scrollbar holds it and clicks nothing else.
        if self.press_scrollbar(cursor, size) {
            self.unfocus();
            return None;
        }
        let rows = self.rows();
        let i = self.hovered(rows.len(), cursor, size)?;
        let Row::Rule(r) = rows[i] else {
            return None;
        };
        if !on(control_x(w), self.row_top(i) + 2.0, 44.0, 20.0) {
            return None;
        }
        self.unfocus();
        match r.default {
            Value::Bool(_) => {
                let on = self.values.bool(r.name);
                self.values.set(r.name, Value::Bool(!on));
            }
            Value::Int(_) => {
                if let Some(b) = self.number(r.name) {
                    b.focused = true;
                    b.reset_blink();
                    b.click(atlas, cursor.0, false, false);
                }
            }
        }
        None
    }
    fn unfocus(&mut self) {
        for (_, b) in &mut self.numbers {
            b.focused = false;
        }
    }
    /// Whether a click lands on a button (for its sound).
    pub fn clicks_button(&self, atlas: &HudAtlas, cursor: (f32, f32), size: (f32, f32)) -> bool {
        let (w, h) = size;
        let on = |(x, y): (f32, f32)| ui::hovered(cursor, x, y, 150.0, 20.0);
        let [done, cancel] = footer(w, h);
        if (on(done) && !self.invalid()) || on(cancel) {
            return true;
        }
        let rows = self.rows();
        self.hovered(rows.len(), cursor, size).is_some_and(|i| {
            matches!(rows[i], Row::Rule(r) if matches!(r.default, Value::Bool(_)))
                && ui::hovered(cursor, control_x(w), self.row_top(i) + 2.0, 44.0, 20.0)
        })
    }

    pub fn draw(&mut self, batch: &mut HudBatch, atlas: &HudAtlas, cursor: (f32, f32)) {
        let (w, h) = (batch.width, batch.height);
        // The screen opens from Create New World only, so the title-screen
        // backgrounds throughout.
        ui::menu_background(batch, atlas, 0.0, 0.0, w, h, false);
        crate::widgets::begin_screen(|| {
            let tr = |key: &str| Text::translate(key, &[]).plain();
            crate::widgets::ScreenInfo::new("game_rules", tr("editGamerule.title"))
        });
        let rows = self.rows();
        let content = ROW * rows.len() as f32 + 4.0;
        self.scroll.clamp(HEADER, h - FOOTER, content);
        // RuleList: the list background, its entries clipped to it, then the
        // separators and scrollbar.
        ui::list_background(batch, atlas, HEADER, h - FOOTER, false);
        let first = batch.vertices.len();
        let hovered = self.hovered(rows.len(), cursor, (w, h));
        let (left, x) = (row_left(w), control_x(w));
        // RuleList, a ContainerObjectSelectionList: its rows' widgets are
        // narrated with their row's place.
        let list = crate::widgets::key_of(&"game_rules/list");
        crate::widgets::narratable(|| {
            crate::widgets::Narratable::container_list(
                rows.len(),
                hovered,
                [0.0, HEADER, w, h - FOOTER - HEADER],
            )
            .key(list)
        });
        for (i, row) in rows.iter().enumerate() {
            let top = self.row_top(i);
            if top + ROW < HEADER || top > h - FOOTER {
                continue;
            }
            let (cx, cy) = (left + 2.0, top + 2.0);
            match *row {
                Row::Category(c) => {
                    // A FocusableTextWidget padded by 4, centred in the entry.
                    let label = Text::translate(&category_key(c), &[])
                        .bold()
                        .underlined()
                        .with_color(yellow());
                    let lw = atlas.font.width_text(&label);
                    let mid = cx + (ROW_WIDTH - 4.0) / 2.0;
                    let lx = mid - (lw / 2.0).trunc();
                    batch.text_styled(atlas, &label, lx, cy + 10.0 - 8.0 + 4.0, u32::MAX, true);
                    // CategoryRuleEntry's FocusableTextWidget: its label.
                    crate::widgets::narratable(|| {
                        crate::widgets::Narratable::text(
                            label.plain(),
                            [lx - 4.0, cy, lw + 8.0, 17.0],
                        )
                        .key(crate::widgets::key_of(&("game_rules/category", c)))
                        .child_of(list, i)
                        .without_tooltip()
                    });
                }
                Row::Rule(r) => {
                    let name = lines(&atlas.font, &Text::translate(&name_key(r), &[]), 175.0);
                    match name.as_slice() {
                        [one] => {
                            batch.text_styled(atlas, one, cx, cy + 5.0, u32::MAX, true);
                        }
                        [one, two, ..] => {
                            batch.text_styled(atlas, one, cx, cy, u32::MAX, true);
                            batch.text_styled(atlas, two, cx, cy + 10.0, u32::MAX, true);
                        }
                        [] => {}
                    }
                    let hover = hovered == Some(i) && ui::hovered(cursor, x, cy, 44.0, 20.0);
                    match r.default {
                        Value::Bool(_) => {
                            let key = if self.values.bool(r.name) {
                                "options.on"
                            } else {
                                "options.off"
                            };
                            let label = Text::translate(key, &[]);
                            let value = self.values.bool(r.name);
                            ui::button_as(
                                batch,
                                atlas,
                                x,
                                cy,
                                44.0,
                                20.0,
                                &label,
                                true,
                                hover,
                                |rect| {
                                    bool_narration(r, value, rect)
                                        .key(crate::widgets::key_of(&("game_rules", r.name)))
                                        .child_of(list, i)
                                },
                            );
                        }
                        Value::Int(_) => {
                            if let Some(b) = self.number(r.name) {
                                b.x = x;
                                b.y = cy;
                                // IntegerRuleEntry's box: the rule's name, its
                                // description and default, a line each.
                                if crate::widgets::narrating() {
                                    b.message =
                                        format!("{}\n{}\n", lang::get(&name_key(r)), narration(r));
                                }
                                b.draw(batch, atlas, None);
                                crate::widgets::last_narratable(|n| {
                                    *n = n
                                        .clone()
                                        .key(crate::widgets::key_of(&("game_rules", r.name)))
                                        .child_of(list, i)
                                        .without_tooltip();
                                });
                            }
                        }
                    }
                }
            }
        }
        batch.clip_since(first, 0.0, HEADER, w, h - FOOTER);
        ui::separators(batch, atlas, HEADER, h - FOOTER, false);
        ui::scrollbar(
            batch,
            atlas,
            Self::bar(w),
            HEADER,
            h - FOOTER,
            content,
            self.scroll.amount.trunc(),
        );
        // The header: the title alone, centred in its 33 pixels.
        let title = Text::translate("editGamerule.title", &[]);
        let tw = atlas.font.width_text(&title);
        batch.text_styled(
            atlas,
            &title,
            ((w - tw) / 2.0).trunc(),
            (HEADER - 9.0) / 2.0,
            u32::MAX,
            true,
        );
        let invalid = self.invalid();
        for ((bx, by), (key, active)) in footer(w, h)
            .into_iter()
            .zip([("gui.done", !invalid), ("gui.cancel", true)])
        {
            let hover = active && ui::hovered(cursor, bx, by, 150.0, 20.0);
            let label = Text::translate(key, &[]);
            let action = if key == "gui.done" {
                crate::widgets::Action::Done
            } else {
                crate::widgets::Action::Cancel
            };
            ui::action_button(
                batch, atlas, action, bx, by, 150.0, 20.0, &label, active, hover,
            );
        }
        // The hovered rule's tooltip: its id, description and default.
        if let Some(Row::Rule(r)) = hovered.map(|i| rows[i]) {
            let mut tip = vec![Text::literal(r.name).with_color(yellow())];
            let description = format!("{}.description", name_key(r));
            if lang::has(&description) {
                tip.extend(lines(
                    &atlas.font,
                    &Text::translate(&description, &[]),
                    150.0,
                ));
            }
            let gray = named_color("gray").unwrap_or(0xAAAAAA);
            tip.push(
                Text::translate(
                    "editGamerule.default",
                    &[Text::literal(r.default.to_string())],
                )
                .with_color(gray),
            );
            ui::tooltip(batch, atlas, &tip, cursor);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gamerules::RULES;

    #[test]
    fn a_switch_narrates_its_name_value_description_and_next_value() {
        use crate::screen_narrator::translate as tr;
        use crate::widgets::What;
        let r = RULES
            .iter()
            .find(|r| matches!(r.default, Value::Bool(_)))
            .unwrap();
        let n = bool_narration(r, true, [0.0; 4]);
        let What::NamedCycle {
            title,
            next,
            active,
        } = &n.what
        else {
            panic!("a CycleButton: {:?}", n.what);
        };
        let full = tr(
            "options.generic_value",
            &[&lang::get(&name_key(r)), &tr("options.on", &[])],
        );
        assert_eq!(
            *title,
            format!("{}\n{}", tr("gui.narrate.button", &[&full]), narration(r))
        );
        assert_eq!(*next, tr("options.off", &[]));
        assert!(*active);
        // The list draws the rule's tooltip, not the button.
        assert_eq!(n.hint.as_deref(), Some(""));
    }

    #[test]
    fn box_colours_are_vanillas() {
        assert_eq!(VALID, crate::hud::rgba(224, 224, 224, 255));
        assert_eq!(INVALID, crate::hud::rgba(255, 0, 0, 255));
    }

    #[test]
    fn numbers_parse_like_brigadier_within_the_rule() {
        let r = gamerules::rule("max_snow_accumulation_height").unwrap();
        assert_eq!(parse(r, "8"), Some(8));
        assert_eq!(parse(r, "9"), None);
        assert_eq!(parse(r, "-1"), None);
        for s in ["", "+3", " 3", "3 ", "3.0", "x"] {
            assert_eq!(parse(r, s), None, "{s:?}");
        }
        let r = gamerules::rule("fire_spread_radius_around_player").unwrap();
        assert_eq!(parse(r, "-1"), Some(-1));
    }

    #[test]
    fn rows_group_categories_and_rules_by_description_id() {
        let s = RulesScreen::new(&GameRules::default());
        let rows = s.rows();
        assert_eq!(rows.len(), RULES.len() + 7);
        assert!(matches!(rows[0], Row::Category("chat")));
        let categories: Vec<_> = rows
            .iter()
            .filter_map(|r| match r {
                Row::Category(c) => Some(*c),
                Row::Rule(_) => None,
            })
            .collect();
        assert_eq!(
            categories,
            [
                "chat", "drops", "misc", "mobs", "player", "spawning", "updates"
            ]
        );
    }

    /// AbstractScrollArea: a press on the bar holds it, a drag moves the
    /// list (to the ends past them), a release lets go.
    #[test]
    fn the_scroll_bar_can_be_held_dragged_and_let_go() {
        let mut s = RulesScreen::new(&GameRules::default());
        let (w, h) = (427.0, 240.0);
        let (top, bottom, content) = (HEADER, h - FOOTER, s.content());
        assert!(
            content > bottom - top,
            "the rules overflow a 240-high screen"
        );
        let bar = RulesScreen::bar(w);
        assert!(!s.press_scrollbar((bar - 4.0, top + 5.0), (w, h)));
        assert!(s.press_scrollbar((bar + 3.0, top + 5.0), (w, h)));
        s.drag(top + 40.0, h);
        assert!(s.scroll.amount > 0.0);
        s.drag(bottom + 10.0, h);
        assert_eq!(s.scroll.amount, content - (bottom - top));
        s.release();
        s.drag(top, h);
        assert_eq!(s.scroll.amount, content - (bottom - top));
    }

    #[test]
    fn edits_change_rules_and_bad_numbers_hold_done() {
        let mut s = RulesScreen::new(&GameRules::default());
        s.unfocus();
        s.number("random_tick_speed").unwrap().focused = true;
        s.edit(|b| b.set_value("10"));
        assert_eq!(
            s.values.get("random_tick_speed"),
            Some(Value::Int(10))
        );
        s.edit(|b| b.set_value("ten"));
        assert!(s.invalid());
        // The last valid value stays, as the responder left it.
        assert_eq!(
            s.values.get("random_tick_speed"),
            Some(Value::Int(10))
        );
        // Leaving discards, without asking (onClose).
        assert_eq!(s.escape(), Some(Exit::Discard));
    }
}
