//! Input method (IME) text: committed composition goes to the focused text
//! field through the same sinks the keyboard and `horizon-ctl type` use.
//! The window only asks the OS for an input method while such a field has
//! focus, so movement keys keep arriving as plain key events. 1.21.10 has
//! no preedit overlay (`PreeditEvent` is 26.x): the OS shows the
//! composition window itself, and only committed text reaches the game.
use super::{Game, Screen, books::BookScreen};

impl Game<'_> {
    /// Whether a text box has focus right now (the fields `ime_commit` can fill).
    pub(super) fn text_field_focused(&mut self) -> bool {
        if self.dialog_open() {
            return self.dialog_has_text_field();
        }
        if self.title_input_active()
            && let Some(title) = self.title.as_mut()
        {
            // A caret field, an append-only one (`focused_field`), or a game
            // rule's box (`has_caret_field` asks the rules screen itself, and
            // `title_edit` types into it).
            return title.has_caret_field() || title.focused_field().is_some();
        }
        if self.sign_edit.is_some() || matches!(self.book, Some(BookScreen::Edit { .. })) {
            return true;
        }
        match self.screen {
            Screen::Chat | Screen::CommandBlock | Screen::Social => true,
            Screen::DebugOptions => self
                .debug_options
                .as_ref()
                .is_some_and(|s| s.search_focused()),
            // The language list's search, the settings search and the pack
            // screen's search (OptionsUi::typing).
            Screen::Options => self.ui.options.typing(),
            Screen::ShareToLan => self.share_to_lan_typing(),
            Screen::Creative => self.mod_browser_typing() || self.creative_screen.tab().search,
            Screen::BlockEditor => self.block_editor.as_ref().is_some_and(|s| s.text_focused()),
            Screen::Inventory => {
                self.mod_browser_typing() || self.recipe_search_focused() || self.station_typing()
            }
            _ => false,
        }
    }

    /// Allow the input method only while a text box has focus.
    pub(super) fn sync_ime(&mut self) {
        let want = self.text_field_focused();
        if want != self.ime_allowed
            && let Some(window) = &self.window
        {
            window.set_ime_allowed(want);
            self.ime_allowed = want;
        }
    }

    /// Text the input method committed (`WindowEvent::Ime(Commit)`).
    pub(super) fn ime_commit(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        // Dialogs have their own fields; the other topmost prompts take no text.
        if self.prompt_open() && !self.dialog_open() {
            return;
        }
        let text: String = text.chars().filter(|c| !c.is_control()).collect();
        if let Some(result) = self.type_text_field_input(&text) {
            if let Err(error) = result {
                log::warn!("Horizon: text input: {error}");
            }
            return;
        }
        // ScreenEvent.CharacterTyped.Pre: the input method's committed text
        // reaches a text box by characters, as a key's own text does, so the
        // open screen hears of each of them first and a cancelled one never
        // reaches the box.
        let Some(text) = self.screen_text_typed(&text) else {
            return;
        };
        if text.is_empty() || !self.text_field_focused() {
            return;
        }
        if self.dialog_type(&text) {
            return;
        }
        if self.title_input_active() {
            self.title_edit(Some(&text), false);
        } else if self.book.is_some() {
            self.book_input(None, Some(&text));
        } else if self.sign_edit.is_some() {
            self.sign_input(None, Some(&text));
        } else {
            match self.screen {
                Screen::Chat => self.chat_type(&text),
                Screen::CommandBlock => self.command_block_type(&text),
                Screen::Social => self.social_type(&text),
                Screen::DebugOptions => self.debug_options_type(&text),
                Screen::Options => self.ui.options.type_text(&self.gui.atlas, &text),
                Screen::ShareToLan => self.share_to_lan_text(Some(&text)),
                Screen::BlockEditor => self.block_editor_type(&text),
                Screen::Inventory | Screen::Creative if self.mod_browser_typing() => {
                    self.mod_browser_text(Some(&text), false)
                }
                Screen::Creative => self.creative_search_edit(Some(&text)),
                Screen::Inventory if self.recipe_search_focused() => {
                    self.recipe_text(Some(&text), false)
                }
                Screen::Inventory => self.station_text(Some(&text), false),
                _ => {}
            }
        }
    }
}
