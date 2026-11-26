//! `ConfirmScreen`: a title, a message wrapped to `width - 50` and two
//! 150-wide buttons 10 apart — the row spans width/2 ± 155. The title
//! sits 20 above the message (`titleTop`, clamped to 10..80) and the
//! buttons 20 below it, not above `height / 6 + 96` and not below
//! `height - 24`; Escape answers no.
use crate::{
    font::Font,
    hud::{HudAtlas, HudBatch},
    text::Text,
    ui,
};

pub struct Confirm {
    /// Each line and where it is drawn.
    pub lines: Vec<(Text, f32, f32)>,
    /// The yes and no buttons' corners.
    pub buttons: [(f32, f32); 2],
    pub focus: Option<usize>,
    /// `declare_no` was called.
    declares_no: bool,
}
impl Confirm {
    pub fn new(font: &Font, size: (f32, f32), title: &Text, message: &Text) -> Self {
        let Choice { lines, buttons, .. } = Choice::new(font, size, title, message, 2, 150.0);
        Self {
            lines,
            buttons: [buttons[0], buttons[1]],
            focus: None,
            declares_no: false,
        }
    }
    /// Tab cycles focus between yes (0) and no (1).
    pub fn tab(&mut self, reverse: bool) {
        self.focus = match self.focus {
            Some(0) => Some(1),
            Some(1) => Some(0),
            None => Some(if reverse { 1 } else { 0 }),
            _ => Some(0),
        };
    }
    /// Left or right arrow key focuses yes or no.
    pub fn arrow(&mut self, left: bool) {
        self.focus = Some(if left { 0 } else { 1 });
    }
    /// The answer of the currently focused button.
    pub fn answer(&self) -> Option<bool> {
        match self.focus {
            Some(0) => Some(true),
            Some(1) => Some(false),
            _ => None,
        }
    }
    /// The answer a click at `cursor` gives: true for yes, false for no.
    pub fn answer_at(&self, cursor: (f32, f32)) -> Option<bool> {
        let [yes, no] = self.buttons;
        let on = |(x, y): (f32, f32)| ui::hovered(cursor, x, y, 150.0, 20.0);
        if on(yes) {
            Some(true)
        } else if on(no) {
            Some(false)
        } else {
            None
        }
    }
    /// The lines and the buttons (yes first), over whatever is behind.
    pub fn draw(
        &self,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        cursor: (f32, f32),
        labels: [&Text; 2],
    ) {
        self.draw_active(batch, atlas, cursor, labels, true);
    }
    /// The second button answers no, as Escape does (`widgets::Action`), once drawn.
    pub fn declare_no(&mut self) {
        self.declares_no = true;
    }
    /// `draw` with the buttons greyed out while `setDelay` runs (`active` false).
    pub fn draw_active(
        &self,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        cursor: (f32, f32),
        labels: [&Text; 2],
        active: bool,
    ) {
        for (line, x, y) in &self.lines {
            batch.text_styled(atlas, line, *x, *y, u32::MAX, true);
        }
        for (i, ((bx, by), label)) in self.buttons.into_iter().zip(labels).enumerate() {
            let is_focused = self.focus == Some(i);
            let hover = active && (ui::hovered(cursor, bx, by, 150.0, 20.0) || is_focused);
            if self.declares_no && i == 1 {
                let action = crate::widgets::Action::Cancel;
                ui::action_button(
                    batch, atlas, action, bx, by, 150.0, 20.0, label, active, hover,
                );
            } else {
                ui::button(batch, atlas, bx, by, 150.0, 20.0, label, active, hover);
            }
        }
    }
}

/// `ConfirmScreen` with a row of `count` buttons `width` wide: `ConfirmLinkScreen`'s
/// yes, copy and no are three of 100, `Confirm` is its two of 150. 1.21.5 lays the row out by
/// absolute bounds spanning width/2 ± 155, so the gap fills the 310 span (10 for two of 150, 5
/// for three of 100); 1.21.6's `LinearLayout` puts them 4 apart and centres the whole column.
pub struct Choice {
    /// Each line and where it is drawn.
    pub lines: Vec<(Text, f32, f32)>,
    /// The buttons' corners, left to right.
    pub buttons: Vec<(f32, f32)>,
    pub width: f32,
    pub focus: Option<usize>,
    /// The button `declare_no` named.
    no_button: Option<usize>,
}
impl Choice {
    pub fn new(
        font: &Font,
        (w, h): (f32, f32),
        title: &Text,
        message: &Text,
        count: usize,
        width: f32,
    ) -> Self {
        let message: Vec<Text> = font
            .split_lines(message, w - 50.0)
            .into_iter()
            .map(|(line, _)| line)
            .collect();
        let tw = font.width_text(title);
        let mw = message
            .iter()
            .map(|l| font.width_text(l))
            .fold(0.0, f32::max);
        let mh = 9.0 * message.len() as f32;
        let gap = if count > 1 {
            ((310.0 - width * count as f32) / (count - 1) as f32).max(0.0)
        } else {
            0.0
        };
        let row = width * count as f32 + gap * count.saturating_sub(1) as f32;
        let col = tw.max(mw).max(row);
        // `ConfirmScreen.titleTop` / `messageTop`: the title sits 20 above
        // the message, the message's middle on the screen's, the title
        // clamped to 10..80; the buttons sit 20 below the message, not
        // above `height / 6 + 96` and not below `height - 24`. (1.21.6's
        // LinearLayout centres the whole column instead, and caps the
        // message at 15 rows.)
        let title_top = (((h - mh) / 2.0 - 29.0).trunc()).clamp(10.0, 80.0);
        let message_top = title_top + 20.0;
        // `Mth.clamp` (not f32::clamp: on a screen shorter than 144 the
        // lower bound passes the upper one, and Java still returns the
        // lower bound for a value under it).
        let lo = (h / 6.0).trunc() + 96.0;
        let hi = h - 24.0;
        let mut by = message_top + mh + 20.0;
        if by < lo {
            by = lo;
        } else if by > hi {
            by = hi;
        }
        let lx = ((w - col) / 2.0).trunc();
        let cell = |cw: f32| lx + ((col - cw) / 2.0).trunc();
        let mid = cell(mw) + (mw / 2.0).trunc();
        let mut lines = vec![(title.clone(), cell(tw), title_top)];
        for (i, line) in message.into_iter().enumerate() {
            let x = mid - (font.width_text(&line) / 2.0).trunc();
            lines.push((line, x, message_top + 9.0 * i as f32));
        }
        let bx = cell(row);
        Self {
            lines,
            buttons: (0..count)
                .map(|i| (bx + (width + gap) * i as f32, by))
                .collect(),
            width,
            focus: None,
            no_button: None,
        }
    }
    /// Tab cycles focus across the buttons.
    pub fn tab(&mut self, reverse: bool) {
        let count = self.buttons.len();
        if count == 0 {
            return;
        }
        self.focus = match self.focus {
            Some(cur) => {
                if reverse {
                    Some(if cur == 0 { count - 1 } else { cur - 1 })
                } else {
                    Some((cur + 1) % count)
                }
            }
            None => Some(if reverse { count - 1 } else { 0 }),
        };
    }
    /// Left or right arrow key moves focus.
    pub fn arrow(&mut self, left: bool) {
        let count = self.buttons.len();
        if count == 0 {
            return;
        }
        self.focus = match self.focus {
            Some(cur) => {
                if left {
                    Some(cur.saturating_sub(1))
                } else {
                    Some((cur + 1).min(count - 1))
                }
            }
            None => Some(if left { 0 } else { count - 1 }),
        };
    }
    /// The index of the currently focused button.
    pub fn pressed(&self) -> Option<usize> {
        self.focus
    }
    /// The button a click at `cursor` hits, counted from the left.
    pub fn pressed_at(&self, cursor: (f32, f32)) -> Option<usize> {
        self.buttons
            .iter()
            .position(|&(x, y)| ui::hovered(cursor, x, y, self.width, 20.0))
    }
    /// Button `i` answers no, as Escape does (`widgets::Action`), once drawn.
    pub fn declare_no(&mut self, i: usize) {
        self.no_button = Some(i);
    }
    /// The lines and the buttons, over whatever is behind.
    pub fn draw(
        &self,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        cursor: (f32, f32),
        labels: &[Text],
    ) {
        for (line, x, y) in &self.lines {
            batch.text_styled(atlas, line, *x, *y, u32::MAX, true);
        }
        for (i, (&(bx, by), label)) in self.buttons.iter().zip(labels).enumerate() {
            let is_focused = self.focus == Some(i);
            let hover = ui::hovered(cursor, bx, by, self.width, 20.0) || is_focused;
            if self.no_button == Some(i) {
                let action = crate::widgets::Action::Cancel;
                ui::action_button(
                    batch, atlas, action, bx, by, self.width, 20.0, label, true, hover,
                );
            } else {
                ui::button(batch, atlas, bx, by, self.width, 20.0, label, true, hover);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hud::Rect;
    use std::{collections::HashMap, path::Path};

    fn font() -> Font {
        // The missing-glyph metrics let these tests run without game assets.
        let rect = Rect {
            u0: 0.0,
            v0: 0.0,
            u1: 1.0,
            v1: 1.0,
            w: 1,
            h: 1,
        };
        Font::load(
            Path::new("/nonexistent-horizon-confirm-test-resources"),
            &HashMap::new(),
            &[],
            (1, 1),
            (0, 0),
            rect,
        )
    }

    #[test]
    fn three_buttons_span_width_over_two_plus_minus_155() {
        let (title, message) = (Text::literal("Open?"), Text::literal("https://x"));
        let c = Choice::new(&font(), (480.0, 270.0), &title, &message, 3, 100.0);
        let xs: Vec<f32> = c.buttons.iter().map(|b| b.0).collect();
        // 3 x 100 + 2 x 5 = 310 wide, centred on 480.
        assert_eq!(xs, vec![85.0, 190.0, 295.0]);
        assert_eq!(c.pressed_at((191.0, c.buttons[1].1 + 1.0)), Some(1));
        assert_eq!(c.pressed_at((188.0, c.buttons[1].1 + 1.0)), None);
        // Confirm keeps its two 150-wide buttons where they were.
        let two = Confirm::new(&font(), (480.0, 270.0), &title, &message);
        assert_eq!(two.buttons[0].0, 85.0);
        assert_eq!(two.buttons[1].0, 245.0);
    }

    #[test]
    fn title_message_and_buttons_follow_the_1_21_5_formulas() {
        let title = Text::literal("Title");
        // One line at 480x270: titleTop clamps to 80, the message sits at
        // 100, the buttons at height / 6 + 96 = 141.
        let one = Choice::new(&font(), (480.0, 270.0), &title, &Text::literal("One"), 2, 150.0);
        assert_eq!(one.lines[0].2, 80.0);
        assert_eq!(one.lines[1].2, 100.0);
        assert_eq!(one.buttons[0].1, 141.0);
        // Twenty lines: the title drops to 16, the message to 36, the
        // buttons to 236 — and a longer message is NOT capped at 15 rows
        // (1.21.6's MultiLineTextWidget limit).
        let long = "l\n".repeat(20);
        let many = Choice::new(
            &font(),
            (480.0, 270.0),
            &title,
            &Text::literal(long.trim_end()),
            2,
            150.0,
        );
        assert_eq!(many.lines.len(), 21);
        assert_eq!(many.lines[0].2, 16.0);
        assert_eq!(many.lines[20].2, 36.0 + 9.0 * 19.0);
        assert_eq!(many.buttons[0].1, 236.0);
        // A short screen: the title's middle is 66, the buttons sit at
        // height / 6 + 96 = 129.
        let short = Choice::new(&font(), (480.0, 200.0), &title, &Text::literal("One"), 2, 150.0);
        assert_eq!(short.lines[0].2, 66.0);
        assert_eq!(short.buttons[0].1, 129.0);
    }

    #[test]
    fn only_the_button_declared_no_gets_its_job() {
        use crate::widgets::{self, Action, DeclaredAction};
        let atlas = HudAtlas::build(Path::new("/nonexistent/horizon-test-assets")).unwrap();
        let (title, message) = (Text::literal("Title"), Text::literal("Message"));
        let labels = [
            Text::literal("Yes"),
            Text::literal("Copy"),
            Text::literal("No"),
        ];
        let drawn = |frame: u64, draw: &dyn Fn(&mut HudBatch)| {
            widgets::set_action_screen(7);
            draw(&mut HudBatch::new([480, 270]));
            widgets::end_frame(frame);
            widgets::last_actions(7)
        };
        let no = |rect| {
            vec![DeclaredAction {
                action: Action::Cancel,
                rect,
                enabled: true,
            }]
        };

        let mut confirm = Confirm::new(&font(), (480.0, 270.0), &title, &message);
        let draw_confirm = |confirm: &Confirm, batch: &mut HudBatch| {
            confirm.draw(batch, &atlas, (0.0, 0.0), [&labels[0], &labels[2]]);
        };
        assert!(drawn(1, &|b| draw_confirm(&confirm, b)).is_empty());
        confirm.declare_no();
        let (x, y) = confirm.buttons[1];
        assert_eq!(
            drawn(2, &|b| draw_confirm(&confirm, b)),
            no([x, y, 150.0, 20.0])
        );

        let mut choice = Choice::new(&font(), (480.0, 270.0), &title, &message, 3, 100.0);
        let draw_choice = |choice: &Choice, batch: &mut HudBatch| {
            choice.draw(batch, &atlas, (0.0, 0.0), &labels);
        };
        assert!(drawn(3, &|b| draw_choice(&choice, b)).is_empty());
        choice.declare_no(2);
        let (x, y) = choice.buttons[2];
        assert_eq!(
            drawn(4, &|b| draw_choice(&choice, b)),
            no([x, y, 100.0, 20.0])
        );
    }

    #[test]
    fn confirm_and_choice_keyboard_navigation() {
        let (title, message) = (Text::literal("Title"), Text::literal("Message"));
        let mut confirm = Confirm::new(&font(), (480.0, 270.0), &title, &message);
        assert_eq!(confirm.focus, None);
        assert_eq!(confirm.answer(), None);

        confirm.tab(false);
        assert_eq!(confirm.focus, Some(0));
        assert_eq!(confirm.answer(), Some(true));

        confirm.tab(false);
        assert_eq!(confirm.focus, Some(1));
        assert_eq!(confirm.answer(), Some(false));

        confirm.tab(true);
        assert_eq!(confirm.focus, Some(0));

        confirm.arrow(false);
        assert_eq!(confirm.focus, Some(1));
        confirm.arrow(true);
        assert_eq!(confirm.focus, Some(0));

        let mut choice = Choice::new(&font(), (480.0, 270.0), &title, &message, 3, 100.0);
        assert_eq!(choice.focus, None);
        assert_eq!(choice.pressed(), None);

        choice.tab(false);
        assert_eq!(choice.focus, Some(0));
        assert_eq!(choice.pressed(), Some(0));

        choice.tab(false);
        assert_eq!(choice.focus, Some(1));

        choice.tab(false);
        assert_eq!(choice.focus, Some(2));

        choice.tab(false);
        assert_eq!(choice.focus, Some(0));

        choice.arrow(false);
        assert_eq!(choice.focus, Some(1));
        choice.arrow(true);
        assert_eq!(choice.focus, Some(0));
    }
}
