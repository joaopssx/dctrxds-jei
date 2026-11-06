//! The search box's text: a cursor and a selection, counted in characters.

/// How the keyboard edits the text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKey {
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
    SelectAll,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditBox {
    text: String,
    cursor: usize,
    /// The other end of the selection, when there is one.
    anchor: Option<usize>,
    pub focused: bool,
}

impl EditBox {
    /// The longest text it keeps, as JEI's search box.
    pub const MAX_CHARS: usize = 128;

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// The selected characters, as `start..end`.
    pub fn selection(&self) -> Option<(usize, usize)> {
        let anchor = self.anchor?;
        (anchor != self.cursor).then(|| (anchor.min(self.cursor), anchor.max(self.cursor)))
    }

    fn len(&self) -> usize {
        self.text.chars().count()
    }

    fn byte(&self, chars: usize) -> usize {
        self.text
            .char_indices()
            .nth(chars)
            .map_or(self.text.len(), |(i, _)| i)
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.chars().take(Self::MAX_CHARS).collect();
        self.cursor = self.len();
        self.anchor = None;
    }

    pub fn clear(&mut self) {
        self.set_text("");
    }

    fn delete_selection(&mut self) -> bool {
        let Some((start, end)) = self.selection() else {
            self.anchor = None;
            return false;
        };
        let (from, to) = (self.byte(start), self.byte(end));
        self.text.replace_range(from..to, "");
        self.cursor = start;
        self.anchor = None;
        true
    }

    /// Types a character; true when the text changed.
    pub fn insert(&mut self, c: char) -> bool {
        if c.is_control() {
            return false;
        }
        let replaced = self.delete_selection();
        if self.len() >= Self::MAX_CHARS {
            return replaced;
        }
        let at = self.byte(self.cursor);
        self.text.insert(at, c);
        self.cursor += 1;
        true
    }

    /// Handles a key; true when the text changed. `extend` is Shift held.
    pub fn key(&mut self, key: EditKey, extend: bool) -> bool {
        match key {
            EditKey::Backspace | EditKey::Delete => {
                if self.delete_selection() {
                    return true;
                }
                let remove = match key {
                    EditKey::Backspace if self.cursor > 0 => self.cursor - 1,
                    EditKey::Delete if self.cursor < self.len() => self.cursor,
                    _ => return false,
                };
                let (from, to) = (self.byte(remove), self.byte(remove + 1));
                self.text.replace_range(from..to, "");
                self.cursor = remove;
                true
            }
            EditKey::SelectAll => {
                self.anchor = Some(0);
                self.cursor = self.len();
                false
            }
            EditKey::Left | EditKey::Right | EditKey::Home | EditKey::End => {
                let target = match key {
                    EditKey::Left => self.cursor.saturating_sub(1),
                    EditKey::Right => (self.cursor + 1).min(self.len()),
                    EditKey::Home => 0,
                    _ => self.len(),
                };
                if extend {
                    self.anchor.get_or_insert(self.cursor);
                } else {
                    self.anchor = None;
                }
                self.cursor = target;
                false
            }
        }
    }

    /// The text before the cursor; what the drawing measures to place it.
    pub fn before_cursor(&self) -> &str {
        &self.text[..self.byte(self.cursor)]
    }

    /// The text before character `chars`.
    pub fn before(&self, chars: usize) -> &str {
        &self.text[..self.byte(chars)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with(text: &str) -> EditBox {
        let mut edit = EditBox::default();
        edit.set_text(text);
        edit
    }

    #[test]
    fn typing_goes_at_the_cursor_including_in_the_middle() {
        let mut edit = with("ac");
        edit.key(EditKey::Left, false);
        assert!(edit.insert('b'));
        assert_eq!((edit.text(), edit.cursor()), ("abc", 2));
    }

    #[test]
    fn control_characters_are_not_text() {
        let mut edit = EditBox::default();
        assert!(!edit.insert('\u{8}'));
        assert_eq!(edit.text(), "");
    }

    #[test]
    fn backspace_and_delete_remove_whole_characters() {
        let mut edit = with("aç日");
        assert!(edit.key(EditKey::Backspace, false));
        assert_eq!(edit.text(), "aç");
        edit.key(EditKey::Home, false);
        assert!(edit.key(EditKey::Delete, false));
        assert_eq!(edit.text(), "ç");
        assert!(edit.key(EditKey::Delete, false));
        assert!(!edit.key(EditKey::Delete, false));
        assert!(!edit.key(EditKey::Backspace, false));
    }

    #[test]
    fn shift_selects_and_typing_replaces_the_selection() {
        let mut edit = with("hello");
        edit.key(EditKey::Left, true);
        edit.key(EditKey::Left, true);
        assert_eq!(edit.selection(), Some((3, 5)));
        assert!(edit.insert('p'));
        assert_eq!(edit.text(), "help");
        assert_eq!(edit.selection(), None);
    }

    #[test]
    fn select_all_then_backspace_empties_it() {
        let mut edit = with("hello");
        edit.key(EditKey::SelectAll, false);
        assert_eq!(edit.selection(), Some((0, 5)));
        assert!(edit.key(EditKey::Backspace, false));
        assert_eq!(edit.text(), "");
    }

    #[test]
    fn moving_without_shift_drops_the_selection() {
        let mut edit = with("hello");
        edit.key(EditKey::SelectAll, false);
        edit.key(EditKey::Home, false);
        assert_eq!(edit.selection(), None);
        assert_eq!(edit.cursor(), 0);
    }

    #[test]
    fn the_cursor_stays_inside_the_text() {
        let mut edit = with("ab");
        edit.key(EditKey::Right, false);
        assert_eq!(edit.cursor(), 2);
        edit.key(EditKey::Home, false);
        edit.key(EditKey::Left, false);
        assert_eq!(edit.cursor(), 0);
    }

    #[test]
    fn the_text_has_a_limit() {
        let mut edit = with(&"x".repeat(500));
        assert_eq!(edit.text().chars().count(), EditBox::MAX_CHARS);
        assert!(!edit.insert('y'));
    }

    #[test]
    fn it_tells_what_is_before_the_cursor() {
        let mut edit = with("açb");
        edit.key(EditKey::Left, false);
        assert_eq!(edit.before_cursor(), "aç");
        assert_eq!(edit.before(1), "a");
    }
}
