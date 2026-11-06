//! The state of the overlay and how it reacts: input goes in, drawing commands
//! and actions come out. The game adapter only carries them across.

use crate::grid::{Grid, SLOT, ScreenInfo};
use crate::persist;

/// The name the overlay shows.
pub const DISPLAY_NAME: &str = "DctrXD\u{b4}s JEI";
/// The most characters of typed text kept.
pub const MAX_TYPED: usize = 32;

const ROW_HEIGHT: f32 = 10.0;
const WHITE: u32 = 0xFF_FF_FF;
const GREY: u32 = 0xA0_A0_A0;
const SLOT_BACKGROUND: u32 = 0x60_00_00_00;
const SLOT_HOVER: u32 = 0x80_FF_FF_FF;
/// GLFW's modifier bit for Control, as the game's hooks report it.
const MODIFIER_CONTROL: u32 = 2;
/// GLFW's left mouse button.
const BUTTON_LEFT: i32 = 0;

/// An item the game knows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub name: String,
}

/// Something the player did while a screen is open.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Input<'a> {
    MousePress {
        button: i32,
    },
    MouseRelease {
        button: i32,
    },
    Key {
        /// `key.keyboard.e`.
        key: &'a str,
        /// The key the player bound to toggling the overlay.
        toggle_key: &'a str,
        modifiers: u32,
    },
    Char(char),
}

/// Whether the event stops here or goes on to the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handled {
    Pass,
    Consumed,
}

/// One thing to draw, in GUI pixels.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawCmd {
    Fill {
        rect: [f32; 4],
        argb: u32,
    },
    Item {
        id: String,
        at: [f32; 2],
    },
    Text {
        text: String,
        at: [f32; 2],
        rgb: u32,
    },
    Tooltip {
        lines: Vec<String>,
        at: [f32; 2],
    },
}

/// What the game has to do for the overlay, later and outside any drawing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Give { item: String, count: u32 },
    SaveText { contents: String },
}

/// Numbers from the adapter, shown on screen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub frames_per_second: u32,
    /// How long the last overlay drawing took.
    pub draw_micros: u64,
}

/// The overlay while a world is open.
#[derive(Debug)]
pub struct App {
    items: Vec<Item>,
    typed: String,
    visible: bool,
    actions: Vec<Action>,
    /// Mouse buttons whose press was ours, so their release is too.
    captured: Vec<i32>,
    /// The stack the game's tooltip hook named since the last drawing.
    tooltip_stack: Option<String>,
    /// What the last drawing saw, so the row doesn't flicker between frames.
    last_tooltip_stack: Option<String>,
}

impl App {
    /// `items` in the game's order; `saved` is the text kept from last time.
    pub fn new(items: Vec<Item>, saved: String) -> Self {
        let mut typed = saved;
        truncate_chars(&mut typed, MAX_TYPED);
        Self {
            items: items
                .into_iter()
                .filter(|i| i.id != "minecraft:air")
                .collect(),
            typed,
            visible: true,
            actions: Vec::new(),
            captured: Vec::new(),
            tooltip_stack: None,
            last_tooltip_stack: None,
        }
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn typed(&self) -> &str {
        &self.typed
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    /// The actions waiting to run, oldest first.
    pub fn take_actions(&mut self) -> Vec<Action> {
        std::mem::take(&mut self.actions)
    }

    /// The game named a stack in a tooltip it is drawing: this tells whether
    /// that hook can stand in for "the slot under the mouse".
    pub fn note_tooltip_stack(&mut self, item: &str) {
        self.tooltip_stack = Some(item.to_owned());
    }

    pub fn on_input(&mut self, screen: &ScreenInfo, input: Input<'_>) -> Handled {
        match input {
            Input::Key {
                key,
                toggle_key,
                modifiers,
            } => self.on_key(screen, key, toggle_key, modifiers),
            Input::Char(c) => self.on_char(screen, c),
            Input::MousePress { button } => self.on_press(screen, button),
            Input::MouseRelease { button } => self.on_release(button),
        }
    }

    fn hovered_grid(&self, screen: &ScreenInfo) -> Option<Grid> {
        if !self.visible {
            return None;
        }
        Grid::compute(screen).filter(|grid| grid.contains(screen.mouse))
    }

    fn on_press(&mut self, screen: &ScreenInfo, button: i32) -> Handled {
        self.captured.retain(|&b| b != button);
        let Some(grid) = self.hovered_grid(screen) else {
            return Handled::Pass;
        };
        self.captured.push(button);
        if button == BUTTON_LEFT
            && let Some(item) = grid
                .slot_at(screen.mouse)
                .and_then(|slot| self.items.get(slot))
        {
            self.actions.push(Action::Give {
                item: item.id.clone(),
                count: 1,
            });
        }
        Handled::Consumed
    }

    /// The game asks a release to be cancelled only when its press was: a
    /// cancelled release of a press it saw would leave a drag held.
    fn on_release(&mut self, button: i32) -> Handled {
        let before = self.captured.len();
        self.captured.retain(|&b| b != button);
        if self.captured.len() < before {
            Handled::Consumed
        } else {
            Handled::Pass
        }
    }

    fn on_key(
        &mut self,
        screen: &ScreenInfo,
        key: &str,
        toggle_key: &str,
        modifiers: u32,
    ) -> Handled {
        let control = modifiers & MODIFIER_CONTROL != 0;
        if control && key == toggle_key {
            self.visible = !self.visible;
            return Handled::Consumed;
        }
        if control || self.hovered_grid(screen).is_none() {
            return Handled::Pass;
        }
        match key.strip_prefix("key.keyboard.") {
            Some("backspace") => {
                if self.typed.pop().is_some() {
                    self.queue_save();
                }
                Handled::Consumed
            }
            // A typing key: keep it from closing the screen (E) or the like.
            // Its character still arrives, as `Input::Char`.
            Some(name) if name == "space" || name.chars().count() == 1 => Handled::Consumed,
            _ => Handled::Pass,
        }
    }

    fn on_char(&mut self, screen: &ScreenInfo, c: char) -> Handled {
        if c.is_control() || self.hovered_grid(screen).is_none() {
            return Handled::Pass;
        }
        if self.typed.chars().count() < MAX_TYPED {
            self.typed.push(c);
            self.queue_save();
        }
        Handled::Consumed
    }

    fn queue_save(&mut self) {
        self.actions.push(Action::SaveText {
            contents: persist::encode(&self.typed),
        });
    }

    /// What to draw this frame.
    pub fn draw(&mut self, screen: &ScreenInfo, stats: Stats) -> Vec<DrawCmd> {
        // Tooltips are drawn before this hook, so this is this frame's answer.
        self.last_tooltip_stack = self.tooltip_stack.take();
        let Some(grid) = (self.visible).then(|| Grid::compute(screen)).flatten() else {
            return Vec::new();
        };
        let [x, y, _, _] = grid.area;
        let text = |row: f32, text: String, rgb| DrawCmd::Text {
            text,
            at: [x, y + row * ROW_HEIGHT],
            rgb,
        };
        let tooltip_row = self
            .last_tooltip_stack
            .as_deref()
            .map_or_else(|| "Slot: -".to_owned(), |item| format!("Slot: {item}"));
        let mut out = vec![
            text(0.0, DISPLAY_NAME.to_owned(), WHITE),
            text(1.0, format!("Typed: {}", self.typed), WHITE),
            text(2.0, tooltip_row, GREY),
            text(
                3.0,
                format!(
                    "{} fps, draw {} us",
                    stats.frames_per_second, stats.draw_micros
                ),
                GREY,
            ),
        ];
        let hovered = grid.slot_at(screen.mouse);
        for (slot, item) in self.items.iter().take(grid.capacity()).enumerate() {
            let [left, top] = grid.slot_position(slot);
            out.push(DrawCmd::Fill {
                rect: [left, top, SLOT, SLOT],
                argb: if hovered == Some(slot) {
                    SLOT_HOVER
                } else {
                    SLOT_BACKGROUND
                },
            });
            out.push(DrawCmd::Item {
                id: item.id.clone(),
                at: [left + 1.0, top + 1.0],
            });
        }
        if let Some(item) = hovered.and_then(|slot| self.items.get(slot)) {
            out.push(DrawCmd::Tooltip {
                lines: vec![item.name.clone(), item.id.clone()],
                at: screen.mouse,
            });
        }
        out
    }
}

fn truncate_chars(text: &mut String, limit: usize) {
    if let Some((end, _)) = text.char_indices().nth(limit) {
        text.truncate(end);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOGGLE: &str = "key.keyboard.o";

    fn items(n: usize) -> Vec<Item> {
        (0..n)
            .map(|i| Item {
                id: format!("test:item_{i}"),
                name: format!("Item {i}"),
            })
            .collect()
    }

    /// A screen whose grid starts at x 332, y 4, with its first slot at y 44.
    fn screen(mouse: [f32; 2]) -> ScreenInfo {
        ScreenInfo {
            rect: [152.0, 52.0, 176.0, 166.0],
            window: [480.0, 270.0],
            mouse,
        }
    }
    const ON_SLOT_0: [f32; 2] = [335.0, 47.0];
    const ON_SLOT_1: [f32; 2] = [353.0, 47.0];
    const ON_HEADER: [f32; 2] = [335.0, 10.0];
    const OUTSIDE: [f32; 2] = [10.0, 10.0];

    fn key<'a>(name: &'a str, modifiers: u32) -> Input<'a> {
        Input::Key {
            key: name,
            toggle_key: TOGGLE,
            modifiers,
        }
    }

    fn app() -> App {
        App::new(items(200), String::new())
    }

    #[test]
    fn air_is_left_out() {
        let mut list = items(2);
        list.insert(
            0,
            Item {
                id: "minecraft:air".into(),
                name: "Air".into(),
            },
        );
        assert_eq!(App::new(list, String::new()).item_count(), 2);
    }

    #[test]
    fn a_left_click_on_a_slot_is_consumed_and_gives_that_item() {
        let mut app = app();
        let handled = app.on_input(&screen(ON_SLOT_1), Input::MousePress { button: 0 });
        assert_eq!(handled, Handled::Consumed);
        assert_eq!(
            app.take_actions(),
            [Action::Give {
                item: "test:item_1".into(),
                count: 1
            }]
        );
        assert!(app.take_actions().is_empty());
    }

    #[test]
    fn a_right_click_or_a_click_on_the_header_is_consumed_and_gives_nothing() {
        let mut app = app();
        let right = app.on_input(&screen(ON_SLOT_0), Input::MousePress { button: 1 });
        let header = app.on_input(&screen(ON_HEADER), Input::MousePress { button: 0 });
        assert_eq!((right, header), (Handled::Consumed, Handled::Consumed));
        assert!(app.take_actions().is_empty());
    }

    #[test]
    fn a_click_on_an_empty_slot_gives_nothing() {
        let mut app = App::new(items(1), String::new());
        let handled = app.on_input(&screen(ON_SLOT_1), Input::MousePress { button: 0 });
        assert_eq!(handled, Handled::Consumed);
        assert!(app.take_actions().is_empty());
    }

    #[test]
    fn a_click_elsewhere_goes_on_to_the_game() {
        let mut app = app();
        let handled = app.on_input(&screen(OUTSIDE), Input::MousePress { button: 0 });
        assert_eq!(handled, Handled::Pass);
        assert!(app.take_actions().is_empty());
    }

    #[test]
    fn only_a_release_whose_press_was_ours_is_consumed() {
        let mut app = app();
        let s = screen(ON_SLOT_0);
        assert_eq!(
            app.on_input(&s, Input::MouseRelease { button: 0 }),
            Handled::Pass
        );
        app.on_input(&s, Input::MousePress { button: 0 });
        assert_eq!(
            app.on_input(&screen(OUTSIDE), Input::MouseRelease { button: 0 }),
            Handled::Consumed
        );
        assert_eq!(
            app.on_input(&s, Input::MouseRelease { button: 0 }),
            Handled::Pass
        );
    }

    #[test]
    fn a_press_that_went_to_the_game_clears_a_stale_capture() {
        let mut app = app();
        app.on_input(&screen(ON_SLOT_0), Input::MousePress { button: 0 });
        // The screen closed before the release arrived. The next press is
        // the game's, and so is its release.
        app.on_input(&screen(OUTSIDE), Input::MousePress { button: 0 });
        assert_eq!(
            app.on_input(&screen(OUTSIDE), Input::MouseRelease { button: 0 }),
            Handled::Pass
        );
    }

    #[test]
    fn typing_over_the_grid_is_kept_and_saved_but_not_elsewhere() {
        let mut app = app();
        // The E key would close the inventory: its press must stop here.
        assert_eq!(
            app.on_input(&screen(ON_HEADER), key("key.keyboard.e", 0)),
            Handled::Consumed
        );
        assert_eq!(
            app.on_input(&screen(ON_HEADER), Input::Char('e')),
            Handled::Consumed
        );
        assert_eq!(app.typed(), "e");
        assert_eq!(
            app.take_actions(),
            [Action::SaveText {
                contents: persist::encode("e")
            }]
        );
        assert_eq!(
            app.on_input(&screen(OUTSIDE), key("key.keyboard.e", 0)),
            Handled::Pass
        );
        assert_eq!(
            app.on_input(&screen(OUTSIDE), Input::Char('x')),
            Handled::Pass
        );
        assert_eq!(app.typed(), "e");
    }

    #[test]
    fn escape_function_keys_and_control_chords_are_not_taken() {
        let mut app = app();
        let s = screen(ON_HEADER);
        for input in [
            key("key.keyboard.escape", 0),
            key("key.keyboard.f3", 0),
            key("key.keyboard.c", MODIFIER_CONTROL),
        ] {
            assert_eq!(app.on_input(&s, input), Handled::Pass, "{input:?}");
        }
        assert_eq!(app.on_input(&s, Input::Char('\u{8}')), Handled::Pass);
    }

    #[test]
    fn backspace_deletes_a_whole_character_and_saves() {
        let mut app = App::new(items(1), "a\u{e7}".into());
        let handled = app.on_input(&screen(ON_HEADER), key("key.keyboard.backspace", 0));
        assert_eq!(handled, Handled::Consumed);
        assert_eq!(app.typed(), "a");
        assert_eq!(app.take_actions().len(), 1);
    }

    #[test]
    fn backspace_on_nothing_saves_nothing() {
        let mut app = app();
        app.on_input(&screen(ON_HEADER), key("key.keyboard.backspace", 0));
        assert!(app.take_actions().is_empty());
    }

    #[test]
    fn typed_text_stops_at_its_limit_and_stays_consumed() {
        let mut app = app();
        let s = screen(ON_HEADER);
        for _ in 0..MAX_TYPED {
            app.on_input(&s, Input::Char('\u{e7}'));
        }
        app.take_actions();
        assert_eq!(app.on_input(&s, Input::Char('z')), Handled::Consumed);
        assert_eq!(app.typed().chars().count(), MAX_TYPED);
        assert!(app.take_actions().is_empty());
    }

    #[test]
    fn saved_text_comes_back_cut_to_the_limit_on_a_character_boundary() {
        let long = "\u{e7}".repeat(MAX_TYPED + 5);
        let app = App::new(items(1), long);
        assert_eq!(app.typed().chars().count(), MAX_TYPED);
    }

    #[test]
    fn control_and_the_toggle_key_hide_the_overlay_and_free_its_area() {
        let mut app = app();
        let s = screen(ON_SLOT_0);
        // Plain O over the grid is typing, not the toggle.
        app.on_input(&s, key(TOGGLE, 0));
        assert!(app.visible());
        assert_eq!(
            app.on_input(&s, key(TOGGLE, MODIFIER_CONTROL)),
            Handled::Consumed
        );
        assert!(!app.visible());
        assert!(app.draw(&s, Stats::default()).is_empty());
        assert_eq!(
            app.on_input(&s, Input::MousePress { button: 0 }),
            Handled::Pass
        );
        app.on_input(&s, key(TOGGLE, MODIFIER_CONTROL));
        assert!(app.visible());
    }

    #[test]
    fn the_drawing_shows_name_text_numbers_and_only_the_items_that_fit() {
        let mut app = App::new(items(200), "abc".into());
        let stats = Stats {
            frames_per_second: 60,
            draw_micros: 12,
        };
        let cmds = app.draw(&screen(OUTSIDE), stats);
        let texts: Vec<&str> = cmds
            .iter()
            .filter_map(|c| match c {
                DrawCmd::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            texts,
            [
                "DctrXD\u{b4}s JEI",
                "Typed: abc",
                "Slot: -",
                "60 fps, draw 12 us"
            ]
        );
        let icons = cmds
            .iter()
            .filter(|c| matches!(c, DrawCmd::Item { .. }))
            .count();
        assert_eq!(icons, 96);
        assert!(!cmds.iter().any(|c| matches!(c, DrawCmd::Tooltip { .. })));
    }

    #[test]
    fn hovering_a_slot_highlights_it_and_shows_its_tooltip() {
        let mut app = app();
        let cmds = app.draw(&screen(ON_SLOT_1), Stats::default());
        assert!(cmds.contains(&DrawCmd::Tooltip {
            lines: vec!["Item 1".into(), "test:item_1".into()],
            at: ON_SLOT_1,
        }));
        let highlighted = cmds
            .iter()
            .filter(|c| matches!(c, DrawCmd::Fill { argb, .. } if *argb == SLOT_HOVER))
            .count();
        assert_eq!(highlighted, 1);
    }

    #[test]
    fn a_tooltip_stack_shows_for_the_frame_it_was_named_in() {
        let mut app = app();
        let s = screen(OUTSIDE);
        let row = |cmds: Vec<DrawCmd>| {
            cmds.into_iter().find_map(|c| match c {
                DrawCmd::Text { text, .. } if text.starts_with("Slot:") => Some(text),
                _ => None,
            })
        };
        app.note_tooltip_stack("minecraft:stone");
        assert_eq!(
            row(app.draw(&s, Stats::default())).as_deref(),
            Some("Slot: minecraft:stone")
        );
        assert_eq!(
            row(app.draw(&s, Stats::default())).as_deref(),
            Some("Slot: -")
        );
    }

    #[test]
    fn no_room_draws_nothing_and_takes_nothing() {
        let mut app = app();
        let tiny = ScreenInfo {
            rect: [0.0, 0.0, 176.0, 166.0],
            window: [176.0, 166.0],
            mouse: [170.0, 10.0],
        };
        assert!(app.draw(&tiny, Stats::default()).is_empty());
        assert_eq!(
            app.on_input(&tiny, Input::MousePress { button: 0 }),
            Handled::Pass
        );
    }
}
