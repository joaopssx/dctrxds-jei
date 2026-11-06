//! The state of the overlay and how it reacts: input goes in, drawing commands
//! and actions come out. The game adapter only carries them across.

use crate::config::{Config, LangChoice};
use crate::edit_box::{EditBox, EditKey};
use crate::grid::{Grid, SLOT, ScreenInfo, ScreenKind, inside};
use crate::ingredient::{Ingredient, Stack, components_label, sorted_order};
use crate::lang::{self, Lang, Msg, text};
use crate::layer::{self, LayerView, Lookup};
use crate::nav::NavigationHistory;
use crate::persist::{self, Bookmark};
use crate::recipe::{Layout, RecipeBook, Role, Tables};
use crate::scroll_math;
use crate::search::filter::Filter;
use crate::search::index::Bits;
use std::collections::{HashMap, HashSet};

/// The name the overlay shows.
pub const DISPLAY_NAME: &str = "DctrXD\u{b4}s JEI";
/// The most stacks the lookup history keeps, JEI's default.
pub const HISTORY_LIMIT: usize = 100;

const WHITE: u32 = 0xFF_FF_FF;
const GREY: u32 = 0xA0_A0_A0;
const YELLOW: u32 = 0xFF_FF_60;
const SLOT_BACKGROUND: u32 = 0x40_00_00_00;
const SLOT_HOVER: u32 = 0x80_FF_FF_FF;
const SLOT_HIDDEN: u32 = 0x80_FF_30_30;
const BUTTON_COLOR: u32 = 0xC0_30_30_30;
const BUTTON_OFF: u32 = 0x60_30_30_30;
const BUTTON_ON: u32 = 0xE0_A0_30_30;
const BOX_COLOR: u32 = 0xC0_00_00_00;
const BOX_FOCUSED: u32 = 0xE0_20_20_50;
const SELECTION: u32 = 0x80_40_60_FF;
const CURSOR: u32 = 0xFF_FF_FF_FF;
const DIM: u32 = 0x90_00_00_00;
const PANEL: u32 = 0xF0_20_20_20;
const TAB_ON: u32 = 0xFF_50_50_50;
const TAB_OFF: u32 = 0xFF_28_28_28;
const NAV_BUTTON: f32 = 14.0;
const CHEAT_BUTTON: f32 = 16.0;

/// GLFW's modifier bits, as the game's hooks report them.
pub const MODIFIER_SHIFT: u32 = 1;
pub const MODIFIER_CONTROL: u32 = 2;
/// GLFW's mouse buttons.
pub const BUTTON_LEFT: i32 = 0;
pub const BUTTON_RIGHT: i32 = 1;
pub const BUTTON_MIDDLE: i32 = 2;
/// A full stack, cut to the item's own limit by the game.
const FULL_STACK: u32 = 64;

/// The mod's keys: name, label, default key. JEI's defaults, where it has one.
pub const KEYS: [[&str; 3]; 7] = [
    ["toggle", "Toggle the list (with Ctrl)", "key.keyboard.o"],
    ["focus_search", "Focus search (with Ctrl)", "key.keyboard.f"],
    ["recipe", "Show recipes", "key.keyboard.r"],
    ["uses", "Show uses", "key.keyboard.u"],
    ["bookmark", "Bookmark", "key.keyboard.a"],
    ["cheat", "Toggle cheat mode (with Ctrl)", "key.keyboard.k"],
    ["edit", "Toggle edit mode (with Ctrl)", "key.keyboard.j"],
];

/// The keys in use, in the order of [`KEYS`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyBinds {
    pub keys: [String; 7],
}

impl Default for KeyBinds {
    fn default() -> Self {
        Self {
            keys: KEYS.map(|k| k[2].to_owned()),
        }
    }
}

impl KeyBinds {
    fn is(&self, bind: usize, key: &str) -> bool {
        self.keys[bind] == key
    }
}

const BIND_TOGGLE: usize = 0;
const BIND_SEARCH: usize = 1;
const BIND_RECIPE: usize = 2;
const BIND_USES: usize = 3;
const BIND_BOOKMARK: usize = 4;
const BIND_CHEAT: usize = 5;
const BIND_EDIT: usize = 6;

/// Something the player did while a screen is open.
#[derive(Clone, Copy, Debug)]
pub enum Input<'a> {
    MousePress {
        button: i32,
        modifiers: u32,
    },
    MouseRelease {
        button: i32,
    },
    Scroll {
        y: f64,
    },
    Key {
        /// `key.keyboard.e`.
        key: &'a str,
        modifiers: u32,
        binds: &'a KeyBinds,
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
        stack: Stack,
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
    Give { stack: Stack, count: u32 },
    PlaceRecipe { book_id: i32, all: bool },
    SaveBookmarks { contents: String },
    SaveHidden { contents: String },
    SaveHistory { contents: String },
    SaveConfig { contents: String },
}

/// Numbers from the adapter.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub frames_per_second: u32,
    /// How long the last overlay drawing took.
    pub draw_micros: u64,
    /// A clock in milliseconds, for slots that cycle.
    pub time_millis: u64,
}

/// What was kept from earlier sessions.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Saved {
    pub bookmarks: Vec<Bookmark>,
    pub hidden: Vec<Stack>,
    pub history: Vec<Stack>,
}

/// Builds the list of ingredients: every item, then the variants recipes name.
fn ingredients_of(tables: &Tables, book: &RecipeBook) -> Vec<Ingredient> {
    let mut places: HashMap<&str, usize> = HashMap::new();
    let mut list: Vec<Ingredient> = Vec::with_capacity(tables.items.len());
    for item in &tables.items {
        if item.id != "minecraft:air" {
            places.insert(&item.id, list.len());
            list.push(Ingredient {
                stack: Stack::of(&item.id),
                name: item.name.clone(),
                tags: item.tags.clone(),
                created: list.len() as u32 * 1024,
            });
        }
    }
    let mut extra: HashMap<usize, u32> = HashMap::new();
    for variant in book.variants() {
        let Some(&place) = places.get(variant.item.as_str()) else {
            continue;
        };
        let base = &list[place];
        let name = components_label(&variant.components).map_or_else(
            || base.name.clone(),
            |label| format!("{} ({label})", base.name),
        );
        let tags = base.tags.clone();
        let seq = extra.entry(place).or_insert(0);
        *seq += 1;
        // Beside its item, as the creative tabs list variants.
        let created = place as u32 * 1024 + (*seq).min(1023);
        list.push(Ingredient {
            stack: variant,
            name,
            tags,
            created,
        });
    }
    list
}

/// The overlay while a world is open.
#[derive(Debug)]
pub struct App {
    config: Config,
    lang: Lang,
    ingredients: Vec<Ingredient>,
    positions: HashMap<Stack, u32>,
    order: Vec<u32>,
    filter: Filter,
    book: RecipeBook,
    known: HashSet<i32>,
    hidden: HashSet<Stack>,
    bookmarks: Vec<Bookmark>,
    history: Vec<Stack>,
    search: EditBox,
    /// What the list shows now, in order.
    listing: Vec<u32>,
    listing_stale: bool,
    first: usize,
    bookmark_first: usize,
    lookup: Option<Lookup>,
    trail: NavigationHistory<Lookup>,
    actions: Vec<Action>,
    /// Mouse buttons whose press was ours, so their release is too.
    captured: Vec<i32>,
    /// A bookmark the mouse went down on, and with which button.
    pressed_bookmark: Option<(usize, i32)>,
    /// The stack the game's tooltip hook named since the last drawing.
    tooltip_stack: Option<Stack>,
    /// The one named in the last frame: the slot under the mouse.
    hovered_slot: Option<Stack>,
    /// The clock of the last drawing: which option a cycling slot shows.
    last_millis: u64,
}

fn stages_key(
    config: &Config,
) -> (
    &crate::search::SearchModes,
    &[crate::ingredient::SortStage],
    bool,
) {
    (&config.modes, &config.sort_stages, config.show_tag_recipes)
}

impl App {
    pub fn new(tables: &Tables, config: Config, game_lang: Lang, saved: Saved) -> Self {
        let book = RecipeBook::build(tables, config.show_tag_recipes);
        let ingredients = ingredients_of(tables, &book);
        let positions = ingredients
            .iter()
            .enumerate()
            .map(|(n, i)| (i.stack.clone(), n as u32))
            .collect();
        let texts: Vec<_> = ingredients.iter().map(Ingredient::search_texts).collect();
        let filter = Filter::new(&texts, config.modes);
        let order = sorted_order(&ingredients, &config.sort_stages);
        let lang = match config.language {
            LangChoice::Game => game_lang,
            LangChoice::Fixed(lang) => lang,
        };
        Self {
            config,
            lang,
            ingredients,
            positions,
            order,
            filter,
            book,
            known: HashSet::new(),
            hidden: saved.hidden.into_iter().collect(),
            bookmarks: saved.bookmarks,
            history: saved.history,
            search: EditBox::default(),
            listing: Vec::new(),
            listing_stale: true,
            first: 0,
            bookmark_first: 0,
            lookup: None,
            trail: NavigationHistory::default(),
            actions: Vec::new(),
            captured: Vec::new(),
            pressed_bookmark: None,
            tooltip_stack: None,
            hovered_slot: None,
            last_millis: 0,
        }
    }

    /// The game's tables changed (a server's recipes arrived, a reload): build
    /// everything again, keeping what the player has open where it still fits.
    pub fn set_tables(&mut self, tables: &Tables, game_lang: Lang) {
        let saved = Saved {
            bookmarks: std::mem::take(&mut self.bookmarks),
            hidden: self.hidden.drain().collect(),
            history: std::mem::take(&mut self.history),
        };
        let search = std::mem::take(&mut self.search);
        let known = std::mem::take(&mut self.known);
        let actions = std::mem::take(&mut self.actions);
        *self = Self::new(tables, self.config.clone(), game_lang, saved);
        self.search = search;
        self.known = known;
        self.actions = actions;
    }

    /// New settings. What they don't touch isn't rebuilt.
    pub fn set_config(&mut self, config: Config, tables: &Tables, game_lang: Lang) {
        let rebuild = stages_key(&config) != stages_key(&self.config);
        self.lang = match config.language {
            LangChoice::Game => game_lang,
            LangChoice::Fixed(lang) => lang,
        };
        self.config = config;
        self.listing_stale = true;
        if rebuild {
            self.set_tables(tables, game_lang);
        }
    }

    pub fn lang(&self) -> Lang {
        self.lang
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn ingredient_count(&self) -> usize {
        self.ingredients.len()
    }

    pub fn search_text(&self) -> &str {
        self.search.text()
    }

    pub fn is_recipes_open(&self) -> bool {
        self.lookup.is_some()
    }

    pub fn bookmarks(&self) -> &[Bookmark] {
        &self.bookmarks
    }

    /// The recipe book ids the player knows.
    pub fn set_known(&mut self, known: &[i32]) {
        self.known = known.iter().copied().collect();
    }

    /// The actions waiting to run, oldest first.
    pub fn take_actions(&mut self) -> Vec<Action> {
        std::mem::take(&mut self.actions)
    }

    /// The game named a stack in a tooltip it is drawing.
    pub fn note_tooltip_stack(&mut self, stack: Stack) {
        self.tooltip_stack = Some(stack);
    }

    /// The screen closed: the page and the search focus go with it.
    pub fn on_screen_closed(&mut self) {
        self.lookup = None;
        self.trail.clear();
        self.search.focused = false;
        self.pressed_bookmark = None;
        self.hovered_slot = None;
        self.tooltip_stack = None;
    }

    // ---------------------------------------------------------------- the list

    fn refresh_listing(&mut self) {
        if !self.listing_stale {
            return;
        }
        self.listing_stale = false;
        let mut found = self.filter.search(self.search.text());
        if !self.config.edit_mode {
            let mut hidden = Bits::empty(self.ingredients.len());
            for stack in &self.hidden {
                if let Some(&n) = self.positions.get(stack) {
                    hidden.set(n as usize);
                }
            }
            found.subtract(&hidden);
        }
        self.listing = self
            .order
            .iter()
            .copied()
            .filter(|&n| found.contains(n as usize))
            .collect();
    }

    /// The first index the list can start at, for this grid.
    fn last_first(&self, grid: &Grid) -> usize {
        let capacity = grid.capacity();
        let count = self.listing.len();
        if self.config.scrolling {
            scroll_math::max_first_item_index(
                count as i32,
                grid.columns as i32,
                grid.rows as i32,
                capacity as i32,
            ) as usize
        } else {
            count.saturating_sub(1) / capacity.max(1) * capacity
        }
    }

    fn move_list(&mut self, grid: &Grid, forward: bool) {
        let step = if self.config.scrolling {
            grid.columns
        } else {
            grid.capacity()
        };
        let last = self.last_first(grid);
        self.first = if forward {
            if self.first >= last {
                if self.config.scrolling { last } else { 0 }
            } else {
                (self.first + step).min(last)
            }
        } else if self.first == 0 {
            if self.config.scrolling { 0 } else { last }
        } else {
            self.first.saturating_sub(step)
        };
    }

    fn visible_bookmarks(&self) -> Vec<(usize, Stack)> {
        self.bookmarks
            .iter()
            .enumerate()
            .filter_map(|(n, bookmark)| {
                let stack = match bookmark {
                    Bookmark::Ingredient(stack) => stack.clone(),
                    Bookmark::Recipe { id } => {
                        let recipe = self.book.recipe(self.book.recipe_by_id(id)?)?;
                        recipe.outputs.first()?.one()
                    }
                };
                self.known_item(&stack).then_some((n, stack))
            })
            .collect()
    }

    /// Whether the game has this item now. A bookmark of a removed mod stays
    /// in the file and just isn't shown.
    fn known_item(&self, stack: &Stack) -> bool {
        self.positions.contains_key(&stack.one())
            || self.positions.contains_key(&Stack::of(&stack.item))
    }

    fn save_bookmarks(&mut self) {
        self.actions.push(Action::SaveBookmarks {
            contents: persist::encode_bookmarks(&self.bookmarks),
        });
    }

    fn save_config(&mut self) {
        self.actions.push(Action::SaveConfig {
            contents: self.config.to_toml(),
        });
    }

    fn toggle_bookmark(&mut self, stack: &Stack) {
        let bookmark = Bookmark::Ingredient(stack.one());
        match self.bookmarks.iter().position(|b| *b == bookmark) {
            Some(place) => {
                self.bookmarks.remove(place);
            }
            None => self.bookmarks.push(bookmark),
        }
        self.save_bookmarks();
    }

    fn toggle_recipe_bookmark(&mut self, id: String) {
        let bookmark = Bookmark::Recipe { id };
        match self.bookmarks.iter().position(|b| *b == bookmark) {
            Some(place) => {
                self.bookmarks.remove(place);
            }
            None => self.bookmarks.push(bookmark),
        }
        self.save_bookmarks();
    }

    fn toggle_hidden(&mut self, stack: &Stack) {
        let stack = stack.one();
        if !self.hidden.remove(&stack) {
            self.hidden.insert(stack);
        }
        self.listing_stale = true;
        let mut list: Vec<Stack> = self.hidden.iter().cloned().collect();
        list.sort();
        self.actions.push(Action::SaveHidden {
            contents: persist::encode_stacks(&list),
        });
    }

    fn note_history(&mut self, stack: &Stack) {
        if !self.config.history_enabled {
            return;
        }
        let stack = stack.one();
        self.history.retain(|s| *s != stack);
        self.history.insert(0, stack);
        self.history.truncate(HISTORY_LIMIT);
        self.actions.push(Action::SaveHistory {
            contents: persist::encode_stacks(&self.history),
        });
    }

    // --------------------------------------------------------- the recipes page

    /// Opens the recipes or uses of a stack; false when there are none.
    pub fn open(&mut self, stack: &Stack, role: Role) -> bool {
        let Some(new) = Lookup::of(&self.book, stack, role) else {
            return false;
        };
        self.note_history(stack);
        if let Some(old) = self.lookup.take() {
            self.trail.record(old);
        }
        self.lookup = Some(new);
        true
    }

    fn open_recipe(&mut self, id: &str) -> bool {
        let Some(new) = self
            .book
            .recipe_by_id(id)
            .and_then(|n| Lookup::single(&self.book, n))
        else {
            return false;
        };
        if let Some(old) = self.lookup.take() {
            self.trail.record(old);
        }
        self.lookup = Some(new);
        true
    }

    fn go_back(&mut self) {
        if let Some(current) = self.lookup.take() {
            match self.trail.go_back(current.clone()) {
                Some(previous) => self.lookup = Some(previous),
                // Nothing behind it: the page closes, as with JEI's Back.
                None => self.trail.clear(),
            }
        }
    }

    fn go_forward(&mut self) {
        if let Some(current) = self.lookup.take() {
            match self.trail.go_forward(current.clone()) {
                Some(next) => self.lookup = Some(next),
                None => self.lookup = Some(current),
            }
        }
    }

    fn layer_view(&mut self, screen: &ScreenInfo) -> Option<LayerView> {
        let mut lookup = self.lookup.take()?;
        let view = layer::layout(&mut lookup, &self.book, screen);
        self.lookup = Some(lookup);
        Some(view)
    }

    fn turn_page(&mut self, view: &LayerView, forward: bool) {
        if let Some(lookup) = self.lookup.as_mut() {
            let count = view.page_count.max(1);
            lookup.page = (view.page + if forward { 1 } else { count - 1 }) % count;
        }
    }

    fn cycle(stacks: &[Stack], millis: u64) -> Option<&Stack> {
        if stacks.is_empty() {
            return None;
        }
        stacks.get((millis / 1000) as usize % stacks.len())
    }

    /// The stack in a recipe's slot under the mouse, now.
    fn layer_slot_stack(&self, view: &LayerView, point: [f32; 2], millis: u64) -> Option<Stack> {
        let (recipe, source) = view.slot_at(point)?;
        let recipe = self.book.recipe(recipe)?;
        Self::cycle(source.stacks(recipe), millis).cloned()
    }

    // -------------------------------------------------------------------- input

    pub fn on_input(&mut self, screen: &ScreenInfo, input: Input<'_>) -> Handled {
        self.refresh_listing();
        match input {
            Input::MousePress { button, modifiers } => self.on_press(screen, button, modifiers),
            Input::MouseRelease { button } => self.on_release(screen, button),
            Input::Scroll { y } => self.on_scroll(screen, y),
            Input::Key {
                key,
                modifiers,
                binds,
            } => self.on_key(screen, key, modifiers, binds),
            Input::Char(c) => self.on_char(c),
        }
    }

    fn take_capture(&mut self, button: i32) -> Handled {
        let before = self.captured.len();
        self.captured.retain(|&b| b != button);
        if self.captured.len() < before {
            Handled::Consumed
        } else {
            Handled::Pass
        }
    }

    fn capture(&mut self, button: i32) -> Handled {
        self.captured.push(button);
        Handled::Consumed
    }

    /// What is at a press on the list: `true` when it took the press.
    fn on_press(&mut self, screen: &ScreenInfo, button: i32, modifiers: u32) -> Handled {
        self.captured.retain(|&b| b != button);
        if self.lookup.is_some() {
            self.press_layer(screen, button, modifiers);
            return self.capture(button);
        }
        if !self.config.overlay_visible {
            return Handled::Pass;
        }
        if let Some(grid) = Grid::list(screen)
            && grid.contains(screen.mouse)
        {
            self.press_list(screen, &grid, button, modifiers);
            return self.capture(button);
        }
        self.search.focused = false;
        if let Some(grid) = Grid::bookmarks(screen)
            && grid.contains(screen.mouse)
        {
            self.press_bookmarks(screen, &grid, button);
            return self.capture(button);
        }
        Handled::Pass
    }

    fn cheat_button(grid: &Grid) -> [f32; 4] {
        let [x, y, w, h] = grid.footer();
        [x + w - CHEAT_BUTTON, y, CHEAT_BUTTON, h]
    }

    fn search_box(grid: &Grid) -> [f32; 4] {
        let [x, y, w, h] = grid.footer();
        [x, y, w - CHEAT_BUTTON - 2.0, h]
    }

    fn nav_buttons(grid: &Grid) -> ([f32; 4], [f32; 4]) {
        let [x, y, w, _] = grid.nav();
        (
            [x, y, NAV_BUTTON, NAV_BUTTON],
            [x + w - NAV_BUTTON, y, NAV_BUTTON, NAV_BUTTON],
        )
    }

    fn press_list(&mut self, screen: &ScreenInfo, grid: &Grid, button: i32, modifiers: u32) {
        let mouse = screen.mouse;
        self.search.focused = false;
        if inside(Self::search_box(grid), mouse) {
            if button == BUTTON_RIGHT {
                self.search.clear();
                self.search_changed();
            }
            self.search.focused = true;
            return;
        }
        if inside(Self::cheat_button(grid), mouse) {
            self.config.cheat_mode = !self.config.cheat_mode;
            self.save_config();
            return;
        }
        let (previous, next) = Self::nav_buttons(grid);
        if inside(previous, mouse) {
            self.move_list(grid, false);
            return;
        }
        if inside(next, mouse) {
            self.move_list(grid, true);
            return;
        }
        let Some(stack) = self.list_stack_at(grid, mouse) else {
            return;
        };
        self.click_stack(&stack, button, modifiers);
    }

    /// A click on an item of the list or a bookmark.
    fn click_stack(&mut self, stack: &Stack, button: i32, modifiers: u32) {
        if self.config.edit_mode {
            self.toggle_hidden(stack);
            return;
        }
        if self.config.cheat_mode {
            let count = if button == BUTTON_MIDDLE || modifiers & MODIFIER_SHIFT != 0 {
                FULL_STACK
            } else {
                1
            };
            self.actions.push(Action::Give {
                stack: stack.one(),
                count,
            });
            return;
        }
        match button {
            BUTTON_LEFT => {
                self.open(stack, Role::Output);
            }
            BUTTON_RIGHT => {
                self.open(stack, Role::Input);
            }
            _ => {}
        }
    }

    fn list_stack_at(&self, grid: &Grid, point: [f32; 2]) -> Option<Stack> {
        let slot = grid.slot_at(point)?;
        let n = *self.listing.get(self.first + slot)?;
        Some(self.ingredients.get(n as usize)?.stack.clone())
    }

    fn history_rows(&self) -> usize {
        usize::from(self.config.history_enabled && !self.history.is_empty())
    }

    /// How many slots of the bookmark panel hold bookmarks.
    fn bookmark_capacity(&self, grid: &Grid) -> usize {
        grid.columns * grid.rows.saturating_sub(self.history_rows()).max(1)
    }

    fn press_bookmarks(&mut self, screen: &ScreenInfo, grid: &Grid, button: i32) {
        let mouse = screen.mouse;
        let visible = self.visible_bookmarks();
        let capacity = self.bookmark_capacity(grid);
        let (previous, next) = Self::nav_buttons(grid);
        if inside(previous, mouse) || inside(next, mouse) {
            let last = visible.len().saturating_sub(1) / capacity.max(1) * capacity;
            self.bookmark_first = if inside(next, mouse) {
                if self.bookmark_first >= last {
                    0
                } else {
                    self.bookmark_first + capacity
                }
            } else if self.bookmark_first == 0 {
                last
            } else {
                self.bookmark_first.saturating_sub(capacity)
            };
            return;
        }
        let Some(slot) = grid.slot_at(mouse) else {
            return;
        };
        if slot >= capacity {
            let row_start = capacity;
            if let Some(stack) = self.history.get(slot - row_start).cloned() {
                self.click_stack(&stack, button, 0);
            }
            return;
        }
        if let Some((place, _)) = visible.get(self.bookmark_first + slot) {
            self.pressed_bookmark = Some((*place, button));
        }
    }

    fn on_release(&mut self, screen: &ScreenInfo, button: i32) -> Handled {
        let handled = self.take_capture(button);
        let Some((from, pressed_with)) = self.pressed_bookmark else {
            return handled;
        };
        if pressed_with != button {
            return handled;
        }
        self.pressed_bookmark = None;
        let Some(grid) = Grid::bookmarks(screen) else {
            return handled;
        };
        let target = grid.slot_at(screen.mouse).and_then(|slot| {
            let visible = self.visible_bookmarks();
            visible
                .get(self.bookmark_first + slot)
                .map(|(place, _)| *place)
        });
        match target {
            // Let go on the same one: a click.
            Some(place) if place == from => match self.bookmarks.get(from).cloned() {
                Some(Bookmark::Ingredient(stack)) => self.click_stack(&stack, button, 0),
                Some(Bookmark::Recipe { id }) => {
                    self.open_recipe(&id);
                }
                None => {}
            },
            // Let go on another: moved there.
            Some(place) if from < self.bookmarks.len() => {
                let bookmark = self.bookmarks.remove(from);
                self.bookmarks
                    .insert(place.min(self.bookmarks.len()), bookmark);
                self.save_bookmarks();
            }
            _ => {}
        }
        handled
    }

    fn press_layer(&mut self, screen: &ScreenInfo, button: i32, modifiers: u32) {
        let Some(view) = self.layer_view(screen) else {
            return;
        };
        let mouse = screen.mouse;
        if inside(view.close, mouse) {
            self.close_layer();
        } else if inside(view.back, mouse) {
            self.go_back();
        } else if inside(view.forward, mouse) {
            self.go_forward();
        } else if inside(view.previous_page, mouse) {
            self.turn_page(&view, false);
        } else if inside(view.next_page, mouse) {
            self.turn_page(&view, true);
        } else if let Some(&(tab, _)) = view.tabs.iter().find(|(_, rect)| inside(*rect, mouse)) {
            if let Some(lookup) = self.lookup.as_mut() {
                lookup.select_tab(tab);
            }
        } else if let Some(recipe) = view.recipes.iter().find(|r| inside(r.bookmark, mouse)) {
            if let Some(id) = self.book.recipe(recipe.recipe).map(|r| r.id.clone()) {
                self.toggle_recipe_bookmark(id);
            }
        } else if let Some(recipe) = view
            .recipes
            .iter()
            .find(|r| r.place.is_some_and(|rect| inside(rect, mouse)))
        {
            let book_id = self.book.recipe(recipe.recipe).map_or(-1, |r| r.book_id);
            if self.known.contains(&book_id) {
                self.actions.push(Action::PlaceRecipe {
                    book_id,
                    all: modifiers & MODIFIER_SHIFT != 0,
                });
                self.close_layer();
            }
        } else if let Some((recipe, source)) = view.slot_at(mouse) {
            let millis = self.last_millis;
            let stack = self
                .book
                .recipe(recipe)
                .and_then(|r| Self::cycle(source.stacks(r), millis).cloned());
            if let Some(stack) = stack {
                let role = if button == BUTTON_RIGHT {
                    Role::Input
                } else {
                    Role::Output
                };
                self.open(&stack, role);
            }
        }
    }

    fn close_layer(&mut self) {
        self.lookup = None;
        self.trail.clear();
    }

    fn on_scroll(&mut self, screen: &ScreenInfo, y: f64) -> Handled {
        let forward = y < 0.0;
        if self.lookup.is_some() {
            if let Some(view) = self.layer_view(screen) {
                self.turn_page(&view, forward);
            }
            return Handled::Consumed;
        }
        if !self.config.overlay_visible {
            return Handled::Pass;
        }
        if let Some(grid) = Grid::list(screen)
            && grid.contains(screen.mouse)
        {
            self.move_list(&grid, forward);
            return Handled::Consumed;
        }
        Handled::Pass
    }

    fn search_changed(&mut self) {
        self.listing_stale = true;
        self.first = 0;
    }

    fn on_search_key(&mut self, name: &str, modifiers: u32) -> Handled {
        let extend = modifiers & MODIFIER_SHIFT != 0;
        let control = modifiers & MODIFIER_CONTROL != 0;
        let edit = match name {
            "backspace" => Some(EditKey::Backspace),
            "delete" => Some(EditKey::Delete),
            "left" => Some(EditKey::Left),
            "right" => Some(EditKey::Right),
            "home" => Some(EditKey::Home),
            "end" => Some(EditKey::End),
            "a" if control => Some(EditKey::SelectAll),
            _ => None,
        };
        if let Some(edit) = edit {
            if self.search.key(edit, extend) {
                self.search_changed();
            }
            return Handled::Consumed;
        }
        match name {
            "escape" | "enter" | "kp.enter" => self.search.focused = false,
            // The window's own keys stay the game's.
            name if is_function_key(name) => return Handled::Pass,
            _ => {}
        }
        Handled::Consumed
    }

    fn on_key(
        &mut self,
        screen: &ScreenInfo,
        key: &str,
        modifiers: u32,
        binds: &KeyBinds,
    ) -> Handled {
        let control = modifiers & MODIFIER_CONTROL != 0;
        if control && binds.is(BIND_TOGGLE, key) && self.lookup.is_none() {
            self.config.overlay_visible = !self.config.overlay_visible;
            self.search.focused = false;
            self.save_config();
            return Handled::Consumed;
        }
        let name = key.strip_prefix("key.keyboard.").unwrap_or(key);
        if self.lookup.is_some() {
            return self.on_layer_key(screen, name, key, modifiers, binds);
        }
        if !self.config.overlay_visible {
            return Handled::Pass;
        }
        if self.search.focused {
            return self.on_search_key(name, modifiers);
        }
        if control {
            if binds.is(BIND_SEARCH, key) {
                self.search.focused = true;
                return Handled::Consumed;
            }
            if binds.is(BIND_CHEAT, key) {
                self.config.cheat_mode = !self.config.cheat_mode;
                self.save_config();
                return Handled::Consumed;
            }
            if binds.is(BIND_EDIT, key) {
                self.config.edit_mode = !self.config.edit_mode;
                self.listing_stale = true;
                self.save_config();
                return Handled::Consumed;
            }
            return Handled::Pass;
        }
        let Some(stack) = self.hovered_stack(screen) else {
            return Handled::Pass;
        };
        self.on_stack_key(&stack, key, binds)
    }

    fn on_stack_key(&mut self, stack: &Stack, key: &str, binds: &KeyBinds) -> Handled {
        if binds.is(BIND_RECIPE, key) {
            self.open(stack, Role::Output);
        } else if binds.is(BIND_USES, key) {
            self.open(stack, Role::Input);
        } else if binds.is(BIND_BOOKMARK, key) {
            self.toggle_bookmark(stack);
        } else {
            return Handled::Pass;
        }
        Handled::Consumed
    }

    fn on_layer_key(
        &mut self,
        screen: &ScreenInfo,
        name: &str,
        key: &str,
        modifiers: u32,
        binds: &KeyBinds,
    ) -> Handled {
        let shift = modifiers & MODIFIER_SHIFT != 0;
        let view = self.layer_view(screen);
        match name {
            "escape" => self.close_layer(),
            "backspace" => self.go_back(),
            "page.up" | "page.down" => {
                let forward = name == "page.down";
                if shift {
                    if let Some(lookup) = self.lookup.as_mut() {
                        lookup.step_tab(forward);
                    }
                } else if let Some(view) = &view {
                    self.turn_page(view, forward);
                }
            }
            name if is_function_key(name) => return Handled::Pass,
            _ => {
                let stack = view
                    .as_ref()
                    .and_then(|view| self.layer_slot_stack(view, screen.mouse, self.last_millis));
                if let Some(stack) = stack {
                    self.on_stack_key(&stack, key, binds);
                }
            }
        }
        // The page is on top: nothing else takes the keyboard, not even E.
        Handled::Consumed
    }

    /// What the mouse is over: the panels', or a slot of the screen by way of
    /// the game's tooltip. Not on the creative screen: its search box can't be
    /// told focused or not from here, and R there must stay a letter.
    fn hovered_stack(&self, screen: &ScreenInfo) -> Option<Stack> {
        if let Some(grid) = Grid::list(screen)
            && grid.contains(screen.mouse)
        {
            return self.list_stack_at(&grid, screen.mouse);
        }
        if let Some(grid) = Grid::bookmarks(screen)
            && grid.contains(screen.mouse)
        {
            let slot = grid.slot_at(screen.mouse)?;
            return self
                .visible_bookmarks()
                .into_iter()
                .nth(self.bookmark_first + slot)
                .map(|(_, stack)| stack);
        }
        if screen.kind == ScreenKind::Creative {
            return None;
        }
        self.hovered_slot.clone()
    }

    fn on_char(&mut self, c: char) -> Handled {
        if self.lookup.is_some() {
            return Handled::Consumed;
        }
        if !self.search.focused {
            return Handled::Pass;
        }
        if self.search.insert(c) {
            self.search_changed();
        }
        Handled::Consumed
    }

    // ------------------------------------------------------------------ drawing

    /// What to draw this frame. `measure` gives a text's width.
    pub fn draw(
        &mut self,
        screen: &ScreenInfo,
        stats: Stats,
        measure: &dyn Fn(&str) -> f32,
    ) -> Vec<DrawCmd> {
        // Tooltips are drawn before this hook, so this is this frame's answer.
        self.hovered_slot = self.tooltip_stack.take();
        self.last_millis = stats.time_millis;
        self.refresh_listing();
        let mut out = Vec::new();
        let mut tooltip = None;
        if self.config.overlay_visible || self.lookup.is_some() {
            if self.config.overlay_visible {
                if let Some(grid) = Grid::list(screen) {
                    tooltip = self.draw_list(&grid, screen, stats, measure, &mut out);
                }
                if let Some(grid) = Grid::bookmarks(screen) {
                    tooltip = self
                        .draw_bookmarks(&grid, screen, measure, &mut out)
                        .or(tooltip);
                }
            }
            if self.lookup.is_some() {
                out.push(DrawCmd::Fill {
                    rect: [0.0, 0.0, screen.window[0], screen.window[1]],
                    argb: DIM,
                });
                tooltip = self.draw_layer(screen, stats, measure, &mut out);
            }
        }
        if let Some(lines) = tooltip {
            out.push(DrawCmd::Tooltip {
                lines,
                at: screen.mouse,
            });
        }
        out
    }

    fn stack_lines(&self, stack: &Stack) -> Vec<String> {
        let mut lines = vec![self.stack_name(stack)];
        lines.push(stack.namespace().to_owned());
        if self.config.edit_mode && self.hidden.contains(&stack.one()) {
            lines.push(text(self.lang, Msg::Hidden).to_owned());
        }
        lines
    }

    fn stack_name(&self, stack: &Stack) -> String {
        self.positions
            .get(&stack.one())
            .or_else(|| self.positions.get(&Stack::of(&stack.item)))
            .and_then(|&n| self.ingredients.get(n as usize))
            .map_or_else(|| stack.item.clone(), |i| i.name.clone())
    }

    fn draw_button(
        out: &mut Vec<DrawCmd>,
        rect: [f32; 4],
        label: &str,
        argb: u32,
        measure: &dyn Fn(&str) -> f32,
    ) {
        out.push(DrawCmd::Fill { rect, argb });
        let width = measure(label);
        out.push(DrawCmd::Text {
            text: label.to_owned(),
            at: [
                rect[0] + ((rect[2] - width) / 2.0).floor(),
                rect[1] + ((rect[3] - 8.0) / 2.0).floor(),
            ],
            rgb: WHITE,
        });
    }

    fn draw_centered(
        out: &mut Vec<DrawCmd>,
        rect: [f32; 4],
        label: String,
        measure: &dyn Fn(&str) -> f32,
    ) {
        let width = measure(&label);
        out.push(DrawCmd::Text {
            at: [rect[0] + ((rect[2] - width) / 2.0).floor(), rect[1] + 3.0],
            text: label,
            rgb: WHITE,
        });
    }

    fn draw_list(
        &mut self,
        grid: &Grid,
        screen: &ScreenInfo,
        stats: Stats,
        measure: &dyn Fn(&str) -> f32,
        out: &mut Vec<DrawCmd>,
    ) -> Option<Vec<String>> {
        self.first = self.first.min(self.last_first(grid));
        let capacity = grid.capacity();
        let (previous, next) = Self::nav_buttons(grid);
        Self::draw_button(out, previous, "<", BUTTON_COLOR, measure);
        Self::draw_button(out, next, ">", BUTTON_COLOR, measure);
        let page = self.first / capacity.max(1) + 1;
        let pages = self.listing.len().div_ceil(capacity.max(1)).max(1);
        Self::draw_centered(out, grid.nav(), format!("{page}/{pages}"), measure);
        let hovered = if self.lookup.is_some() {
            None
        } else {
            grid.slot_at(screen.mouse)
        };
        let mut tooltip = None;
        for slot in 0..capacity {
            let Some(&n) = self.listing.get(self.first + slot) else {
                break;
            };
            let Some(ingredient) = self.ingredients.get(n as usize) else {
                continue;
            };
            let [x, y] = grid.slot_position(slot);
            let argb = if hovered == Some(slot) {
                tooltip = Some(self.stack_lines(&ingredient.stack));
                SLOT_HOVER
            } else if self.hidden.contains(&ingredient.stack) {
                SLOT_HIDDEN
            } else {
                SLOT_BACKGROUND
            };
            out.push(DrawCmd::Fill {
                rect: [x, y, SLOT, SLOT],
                argb,
            });
            out.push(DrawCmd::Item {
                stack: ingredient.stack.clone(),
                at: [x + 1.0, y + 1.0],
            });
        }
        self.draw_search(grid, measure, out);
        let cheat = Self::cheat_button(grid);
        Self::draw_button(
            out,
            cheat,
            "C",
            if self.config.cheat_mode {
                BUTTON_ON
            } else {
                BUTTON_OFF
            },
            measure,
        );
        if inside(cheat, screen.mouse) && self.lookup.is_none() {
            tooltip = Some(vec![text(self.lang, Msg::CheatMode).to_owned()]);
        }
        let notice = if self.config.edit_mode {
            Some(Msg::EditOn)
        } else if self.config.cheat_mode {
            Some(Msg::CheatOn)
        } else {
            None
        };
        if let Some(notice) = notice {
            out.push(DrawCmd::Text {
                text: text(self.lang, notice).to_owned(),
                at: [screen.rect[0], (screen.rect[1] - 10.0).max(0.0)],
                rgb: YELLOW,
            });
        }
        if self.config.show_stats {
            out.push(DrawCmd::Text {
                text: format!(
                    "{} fps, draw {} us",
                    stats.frames_per_second, stats.draw_micros
                ),
                at: [screen.rect[0], screen.rect[1] + screen.rect[3] + 2.0],
                rgb: GREY,
            });
        }
        tooltip
    }

    fn draw_search(&self, grid: &Grid, measure: &dyn Fn(&str) -> f32, out: &mut Vec<DrawCmd>) {
        let rect = Self::search_box(grid);
        let [x, y, w, h] = rect;
        out.push(DrawCmd::Fill {
            rect,
            argb: if self.search.focused {
                BOX_FOCUSED
            } else {
                BOX_COLOR
            },
        });
        let text_x = x + 3.0;
        let text_y = y + ((h - 8.0) / 2.0).floor();
        let room = w - 6.0;
        if self.search.text().is_empty() && !self.search.focused {
            out.push(DrawCmd::Text {
                text: text(self.lang, Msg::SearchHint).to_owned(),
                at: [text_x, text_y],
                rgb: GREY,
            });
            return;
        }
        // A long text shows its end, where the typing is.
        let full = self.search.text();
        let mut start = 0;
        while measure(&full[start..]) > room {
            match full[start..].chars().next() {
                Some(c) => start += c.len_utf8(),
                None => break,
            }
        }
        let skipped = measure(&full[..start]);
        if let Some((from, to)) = self.search.selection() {
            let left = (measure(self.search.before(from)) - skipped).max(0.0);
            let right = (measure(self.search.before(to)) - skipped).min(room);
            out.push(DrawCmd::Fill {
                rect: [text_x + left, y + 1.0, (right - left).max(0.0), h - 2.0],
                argb: SELECTION,
            });
        }
        out.push(DrawCmd::Text {
            text: full[start..].to_owned(),
            at: [text_x, text_y],
            rgb: WHITE,
        });
        if self.search.focused {
            let at = (measure(self.search.before_cursor()) - skipped).clamp(0.0, room);
            out.push(DrawCmd::Fill {
                rect: [text_x + at, y + 2.0, 1.0, h - 4.0],
                argb: CURSOR,
            });
        }
    }

    fn draw_bookmarks(
        &mut self,
        grid: &Grid,
        screen: &ScreenInfo,
        measure: &dyn Fn(&str) -> f32,
        out: &mut Vec<DrawCmd>,
    ) -> Option<Vec<String>> {
        let visible = self.visible_bookmarks();
        let capacity = self.bookmark_capacity(grid);
        if visible.is_empty() && self.history.is_empty() {
            return None;
        }
        if self.bookmark_first >= visible.len() {
            self.bookmark_first = visible.len().saturating_sub(1) / capacity.max(1) * capacity;
        }
        let hovered = if self.lookup.is_some() {
            None
        } else {
            grid.slot_at(screen.mouse)
        };
        let (previous, next) = Self::nav_buttons(grid);
        Self::draw_button(out, previous, "<", BUTTON_COLOR, measure);
        Self::draw_button(out, next, ">", BUTTON_COLOR, measure);
        Self::draw_centered(
            out,
            grid.nav(),
            text(self.lang, Msg::Bookmarks).to_owned(),
            measure,
        );
        let mut tooltip = None;
        for (slot, (_, stack)) in visible
            .iter()
            .skip(self.bookmark_first)
            .take(capacity)
            .enumerate()
        {
            tooltip = self
                .draw_panel_slot(grid, slot, stack, hovered, out)
                .or(tooltip);
        }
        if self.history_rows() > 0 {
            for (i, stack) in self.history.iter().take(grid.columns).enumerate() {
                tooltip = self
                    .draw_panel_slot(grid, capacity + i, stack, hovered, out)
                    .or(tooltip);
            }
        }
        tooltip
    }

    fn draw_panel_slot(
        &self,
        grid: &Grid,
        slot: usize,
        stack: &Stack,
        hovered: Option<usize>,
        out: &mut Vec<DrawCmd>,
    ) -> Option<Vec<String>> {
        let [x, y] = grid.slot_position(slot);
        let is_hovered = hovered == Some(slot);
        out.push(DrawCmd::Fill {
            rect: [x, y, SLOT, SLOT],
            argb: if is_hovered {
                SLOT_HOVER
            } else {
                SLOT_BACKGROUND
            },
        });
        out.push(DrawCmd::Item {
            stack: stack.clone(),
            at: [x + 1.0, y + 1.0],
        });
        is_hovered.then(|| self.stack_lines(stack))
    }

    fn draw_layer(
        &mut self,
        screen: &ScreenInfo,
        stats: Stats,
        measure: &dyn Fn(&str) -> f32,
        out: &mut Vec<DrawCmd>,
    ) -> Option<Vec<String>> {
        let view = self.layer_view(screen)?;
        let lookup = self.lookup.as_ref()?;
        let lang = self.lang;
        let mouse = screen.mouse;
        out.push(DrawCmd::Fill {
            rect: view.panel,
            argb: PANEL,
        });
        Self::draw_button(out, view.back, "<", BUTTON_COLOR, measure);
        Self::draw_button(out, view.forward, ">", BUTTON_COLOR, measure);
        Self::draw_button(out, view.close, "x", BUTTON_COLOR, measure);
        let title = match &lookup.focus {
            Some(focus) => format!(
                "{}: {}",
                text(
                    lang,
                    if lookup.role == Role::Output {
                        Msg::Recipes
                    } else {
                        Msg::Uses
                    }
                ),
                self.stack_name(focus)
            ),
            None => text(lang, Msg::Recipes).to_owned(),
        };
        out.push(DrawCmd::Text {
            text: title,
            at: view.title_at,
            rgb: WHITE,
        });
        let mut tooltip = None;
        for &(tab, rect) in &view.tabs {
            let Some(category) = lookup
                .found
                .get(tab)
                .and_then(|f| self.book.categories().get(f.category))
            else {
                continue;
            };
            out.push(DrawCmd::Fill {
                rect,
                argb: if tab == lookup.tab { TAB_ON } else { TAB_OFF },
            });
            out.push(DrawCmd::Item {
                stack: category.icon.clone(),
                at: [rect[0] + 1.0, rect[1] + 1.0],
            });
            if inside(rect, mouse) {
                tooltip = Some(vec![lang::title(lang, &category.title).to_owned()]);
            }
        }
        Self::draw_button(out, view.previous_page, "<", BUTTON_COLOR, measure);
        Self::draw_button(out, view.next_page, ">", BUTTON_COLOR, measure);
        out.push(DrawCmd::Text {
            text: format!("{}/{}", view.page + 1, view.page_count),
            at: view.page_at,
            rgb: WHITE,
        });
        for recipe_view in &view.recipes {
            let Some(recipe) = self.book.recipe(recipe_view.recipe) else {
                continue;
            };
            for slot in &recipe_view.slots {
                let rect = [slot.at[0], slot.at[1], SLOT, SLOT];
                let hovered = inside(rect, mouse);
                out.push(DrawCmd::Fill {
                    rect,
                    argb: if hovered { SLOT_HOVER } else { SLOT_BACKGROUND },
                });
                if let Some(stack) = Self::cycle(slot.source.stacks(recipe), stats.time_millis) {
                    out.push(DrawCmd::Item {
                        stack: stack.clone(),
                        at: [slot.at[0] + 1.0, slot.at[1] + 1.0],
                    });
                    if stack.count > 1 {
                        let label = stack.count.to_string();
                        out.push(DrawCmd::Text {
                            at: [slot.at[0] + 17.0 - measure(&label), slot.at[1] + 10.0],
                            text: label,
                            rgb: WHITE,
                        });
                    }
                    if hovered {
                        tooltip = Some(self.stack_lines(stack));
                    }
                }
            }
            if let Some(at) = recipe_view.arrow {
                out.push(DrawCmd::Text {
                    text: "->".to_owned(),
                    at,
                    rgb: WHITE,
                });
            }
            let note = match recipe.layout {
                Layout::Grid {
                    shapeless: true, ..
                } => Some(text(lang, Msg::Shapeless).to_owned()),
                Layout::Cooking {
                    ticks,
                    xp_hundredths,
                } => Some(format!(
                    "{} {}, {} {}",
                    ticks as f32 / 20.0,
                    text(lang, Msg::Seconds),
                    xp_hundredths as f32 / 100.0,
                    text(lang, Msg::Experience),
                )),
                Layout::Wrapped => Some(recipe.id.clone()),
                _ => None,
            };
            if let Some(note) = note {
                out.push(DrawCmd::Text {
                    text: note,
                    at: recipe_view.note_at,
                    rgb: GREY,
                });
            }
            if let Some(rect) = recipe_view.place {
                let known = self.known.contains(&recipe.book_id);
                Self::draw_button(
                    out,
                    rect,
                    "+",
                    if known { BUTTON_COLOR } else { BUTTON_OFF },
                    measure,
                );
                if inside(rect, mouse) {
                    tooltip = Some(vec![
                        text(
                            lang,
                            if known {
                                Msg::PlaceRecipe
                            } else {
                                Msg::NotInBook
                            },
                        )
                        .to_owned(),
                    ]);
                }
            }
            let marked = self.bookmarks.contains(&Bookmark::Recipe {
                id: recipe.id.clone(),
            });
            Self::draw_button(
                out,
                recipe_view.bookmark,
                "*",
                if marked { BUTTON_ON } else { BUTTON_COLOR },
                measure,
            );
            if inside(recipe_view.bookmark, mouse) {
                tooltip = Some(vec![text(lang, Msg::AddBookmark).to_owned()]);
            }
        }
        tooltip
    }
}

fn is_function_key(name: &str) -> bool {
    name.strip_prefix('f')
        .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;
