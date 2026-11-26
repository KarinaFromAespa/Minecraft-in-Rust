//! `AccessibilityOnboardingScreen`: the screen shown once before the title
//! on a first launch (`Gui.addInitialScreens` while `onboardAccessibility`
//! is set), with the Narrator toggle, the Sound, Accessibility and Language
//! pages and Continue.
use super::{Button, Mode, Title, translate};
use crate::{
    hud::{HudAtlas, HudBatch, rgba},
    lang,
};

/// `AccessibilityOnboardingScreen.TEXT_WIDGET_WIDTH`.
const TEXT_WIDTH: f32 = 374.0;
/// `HeaderAndFooterLayout(this, initTitleYPos(), 33)`.
const HEADER: f32 = 90.0;
const FOOTER: f32 = 33.0;
/// The ids of the screen's buttons.
pub(super) const NARRATOR: u8 = 80;
pub(super) const SOUNDS: u8 = 81;
pub(super) const ACCESSIBILITY: u8 = 82;
pub(super) const LANGUAGE: u8 = 83;
pub(super) const CONTINUE: u8 = 84;

/// The open screen. 1.21.5 has no fade: it appears at once and `onClose`
/// closes it at once (`close(true, onClose)`); the two-second fade in and
/// one-second fade out are 1.21.6-only.
#[derive(Clone, Debug, Default)]
pub struct Onboarding;

impl Onboarding {
    pub fn new() -> Self {
        Self
    }
}

/// Where the screen's parts go.
pub(super) struct Layout {
    /// The title box: x, y, width, height.
    pub text: [f32; 4],
    /// The four grid buttons, row by row.
    pub grid: [(f32, f32); 4],
    pub continue_at: (f32, f32),
}

/// `HeaderAndFooterLayout.arrangeElements` for the contents (the title in
/// an 8 px cell over the grid of four 150-wide buttons in 4 px cells, both
/// centred) with the title wrapped to `lines`.
pub(super) fn layout(w: f32, h: f32, lines: usize) -> Layout {
    let text_h = 9.0 * lines as f32 + 8.0;
    // The 8 px cell around the box, then the grid's 4 px cell around the
    // two rows of 4 px cells.
    let content_w = TEXT_WIDTH + 16.0;
    let content_h = text_h + 16.0 + 64.0;
    let y = (HEADER + 30.0).min(h - FOOTER - content_h);
    let x = ((w.max(content_w) - content_w) / 2.0).trunc();
    // The grid: 316 wide in the 382 left by the cell's padding.
    let grid_x = x + 4.0 + ((content_w - 8.0 - 316.0) / 2.0).trunc() + 4.0;
    let grid_y = y + text_h + 16.0 + 4.0 + 4.0;
    Layout {
        text: [x + 8.0, y + 8.0, TEXT_WIDTH, text_h],
        grid: [
            (grid_x, grid_y),
            (grid_x + 158.0, grid_y),
            (grid_x, grid_y + 28.0),
            (grid_x + 158.0, grid_y + 28.0),
        ],
        continue_at: (
            ((w - 150.0) / 2.0).trunc(),
            h - FOOTER + ((FOOTER - 20.0) / 2.0).trunc(),
        ),
    }
}

/// The wrapped lines of the title (`accessibility.onboarding.screen.title`).
fn title_lines(atlas: &HudAtlas) -> Vec<String> {
    let title = lang::get_or(
        "accessibility.onboarding.screen.title",
        "Welcome to Minecraft!\n\nWould you like to enable the Narrator or visit the Accessibility Settings?",
    );
    let mut lines = Vec::new();
    for paragraph in title.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
        } else {
            lines.extend(
                atlas
                    .font
                    .split(&crate::text::Text::literal(paragraph), TEXT_WIDTH - 8.0)
                    .iter()
                    .map(|line| line.plain()),
            );
        }
    }
    lines
}

impl Title {
    /// The first screen of a first launch (`addInitialScreens`).
    pub fn start_onboarding(&mut self) {
        self.mode = Mode::Onboarding(Onboarding::new());
    }

    /// `onClose`: Escape and Continue close at once, and the title follows
    /// with onboarding finished (`Options.onboardingAccessibilityFinished`).
    pub(super) fn onboarding_close(&mut self) {
        if matches!(self.mode, Mode::Onboarding(_)) {
            self.mode = Mode::Main;
            self.onboarding_finished = true;
        }
    }

    pub(super) fn onboarding_buttons(&self, w: f32, h: f32) -> Vec<Button> {
        let layout = layout(w, h, self.message_lines.get());
        let button =
            |(x, y): (f32, f32), label: String, enabled: bool, id: u8, icon: Option<_>| Button {
                x,
                y,
                w: 150.0,
                label,
                enabled,
                id,
                icon: None,
                trailing: icon,
                action: None,
            };
        // The Narrator button is inactive without a narrator, and shows
        // `options.narrator.notavailable`.
        let narrator = translate(
            "options.generic_value",
            "%s: %s",
            &[
                &lang::get_or("options.narrator", "Narrator"),
                &lang::get_or("options.narrator.notavailable", "Not Available"),
            ],
        );
        vec![
            button(layout.grid[0], narrator, false, NARRATOR, None),
            button(
                layout.grid[1],
                lang::get_or("options.sounds", "Music & Sounds..."),
                true,
                SOUNDS,
                Some(("icon/music_notes", 16.0)),
            ),
            button(
                layout.grid[2],
                lang::get_or(
                    "accessibility.onboarding.accessibility.button",
                    "Accessibility Settings...",
                ),
                true,
                ACCESSIBILITY,
                Some(("icon/accessibility", 15.0)),
            ),
            button(
                layout.grid[3],
                lang::get_or("options.language", "Language..."),
                true,
                LANGUAGE,
                Some(("icon/language", 15.0)),
            ),
            button(
                layout.continue_at,
                lang::get_or("gui.continue", "Continue"),
                true,
                CONTINUE,
                None,
            ),
        ]
    }

    /// The title box over the background.
    pub(super) fn draw_onboarding(&self, atlas: &HudAtlas, batch: &mut HudBatch, w: f32, h: f32) {
        let Mode::Onboarding(_) = &self.mode else {
            return;
        };
        crate::ui::menu_background(batch, atlas, 0.0, 0.0, w, h, false);
        let lines = title_lines(atlas);
        self.message_lines.set(lines.len());
        let [x, y, bw, bh] = layout(w, h, lines.len()).text;
        // FocusableTextWidget: a black fill from x + 1 and the grey outline.
        batch.fill(atlas, x + 1.0, y, bw - 1.0, bh, rgba(0, 0, 0, 255));
        let border = rgba(160, 160, 160, 255);
        batch.fill(atlas, x, y, bw, 1.0, border);
        batch.fill(atlas, x, y + bh - 1.0, bw, 1.0, border);
        batch.fill(atlas, x, y, 1.0, bh, border);
        batch.fill(atlas, x + bw - 1.0, y, 1.0, bh, border);
        // Centred on the widget, the text 4 px inside.
        let middle = x + (bw / 2.0).trunc();
        for (i, line) in lines.iter().enumerate() {
            let width = HudBatch::text_width(atlas, line);
            batch.text(
                atlas,
                line,
                middle - (width.ceil() / 2.0).floor(),
                y + 4.0 + 9.0 * i as f32,
                u32::MAX,
                true,
            );
        }
    }

    /// After the buttons: the logo, at full alpha (`LogoRenderer` draws it
    /// after the widgets).
    pub(super) fn draw_onboarding_logo(&self, atlas: &HudAtlas, batch: &mut HudBatch, w: f32) {
        let Mode::Onboarding(_) = &self.mode else {
            return;
        };
        super::draw_logo(atlas, batch, w);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_widgets_sit_where_the_layouts_put_them() {
        // 854 x 480 GUI pixels, the title wrapped to 4 lines: the contents
        // frame starts at min(90 + 30, 480 - 33 - 124).
        let l = layout(854.0, 480.0, 4);
        assert_eq!(l.text, [240.0, 128.0, 374.0, 44.0]);
        assert_eq!(l.grid[0], (273.0, 188.0));
        assert_eq!(l.grid[1], (431.0, 188.0));
        assert_eq!(l.grid[2], (273.0, 216.0));
        assert_eq!(l.continue_at, (352.0, 453.0));
        // A short window moves the contents up to clear the footer.
        let l = layout(854.0, 240.0, 4);
        assert_eq!(l.text[1], 240.0 - 33.0 - 124.0 + 8.0);
    }

    #[test]
    fn escape_closes_at_once_and_the_title_follows() {
        let mut title = Title::new(
            std::path::PathBuf::new(),
            std::path::Path::new("/nonexistent/horizon-test-resources"),
        );
        title.start_onboarding();
        title.escape();
        assert!(matches!(title.mode, Mode::Main));
        assert!(title.onboarding_finished);
    }
}
