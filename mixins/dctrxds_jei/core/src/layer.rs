//! The recipes page: a layer the overlay draws above the open screen. The game
//! can't open a mod's own screen over an inventory, and the container has to
//! stay open anyway so a recipe can be placed in it. Geometry only: everything
//! here is worked out from the lookup and the window, for drawing and clicks.

use crate::grid::{SLOT, ScreenInfo, inside};
use crate::ingredient::Stack;
use crate::recipe::{Found, Layout, Recipe, RecipeBook, Role};

pub const BUTTON: f32 = 12.0;
const MAX_WIDTH: f32 = 220.0;
const MAX_HEIGHT: f32 = 236.0;
const TAB: f32 = 20.0;
const NOTE: f32 = 12.0;
const GAP: f32 = 8.0;
const ARROW: f32 = 16.0;
const WRAP: usize = 8;
const WRAP_ROWS: usize = 6;

/// What the page shows; a value, so the history can keep it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lookup {
    /// The item asked about; none for a recipe bookmark.
    pub focus: Option<Stack>,
    pub role: Role,
    pub found: Vec<Found>,
    pub tab: usize,
    pub page: usize,
}

impl Lookup {
    /// `None` when nothing is found.
    pub fn of(book: &RecipeBook, focus: &Stack, role: Role) -> Option<Self> {
        let found = book.lookup(focus, role);
        (!found.is_empty()).then(|| Self {
            focus: Some(focus.one()),
            role,
            found,
            tab: 0,
            page: 0,
        })
    }

    /// A page with one recipe.
    pub fn single(book: &RecipeBook, recipe: u32) -> Option<Self> {
        Some(Self {
            focus: None,
            role: Role::Output,
            found: vec![book.single(recipe)?],
            tab: 0,
            page: 0,
        })
    }

    pub fn current(&self) -> Option<&Found> {
        self.found.get(self.tab)
    }

    /// Moves to a tab, wrapping around.
    pub fn step_tab(&mut self, forward: bool) {
        let count = self.found.len();
        if count > 0 {
            self.tab = (self.tab + if forward { 1 } else { count - 1 }) % count;
            self.page = 0;
        }
    }

    pub fn select_tab(&mut self, tab: usize) {
        if tab < self.found.len() {
            self.tab = tab;
            self.page = 0;
        }
    }
}

/// A slot of a recipe on the page.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlotView {
    pub at: [f32; 2],
    pub source: SlotSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotSource {
    Input(usize),
    Output,
}

impl SlotSource {
    /// The stacks the slot cycles through.
    pub fn stacks(self, recipe: &Recipe) -> &[Stack] {
        match self {
            Self::Input(n) => recipe.inputs.get(n).map_or(&[], Vec::as_slice),
            Self::Output => &recipe.outputs,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecipeView {
    pub recipe: u32,
    pub slots: Vec<SlotView>,
    pub arrow: Option<[f32; 2]>,
    pub note_at: [f32; 2],
    /// The + button, for a recipe the recipe book shows.
    pub place: Option<[f32; 4]>,
    pub bookmark: [f32; 4],
}

/// Where everything is on the page this frame.
#[derive(Clone, Debug, PartialEq)]
pub struct LayerView {
    pub panel: [f32; 4],
    pub back: [f32; 4],
    pub forward: [f32; 4],
    pub close: [f32; 4],
    pub title_at: [f32; 2],
    pub tabs: Vec<(usize, [f32; 4])>,
    pub previous_page: [f32; 4],
    pub next_page: [f32; 4],
    pub page_at: [f32; 2],
    pub page_count: usize,
    pub page: usize,
    pub recipes: Vec<RecipeView>,
}

impl LayerView {
    /// The slot under `point`: its recipe and where it gets its stacks.
    pub fn slot_at(&self, point: [f32; 2]) -> Option<(u32, SlotSource)> {
        self.recipes.iter().find_map(|view| {
            view.slots.iter().find_map(|slot| {
                inside([slot.at[0], slot.at[1], SLOT, SLOT], point)
                    .then_some((view.recipe, slot.source))
            })
        })
    }
}

fn block_height(recipe: &Recipe) -> f32 {
    let rows = match recipe.layout {
        Layout::Grid { height, .. } => height as f32 * SLOT,
        Layout::Cooking { .. } | Layout::Row => SLOT,
        Layout::Wrapped => wrapped_rows(recipe.inputs.len()) as f32 * SLOT,
    };
    let note = match recipe.layout {
        Layout::Cooking { .. }
        | Layout::Grid {
            shapeless: true, ..
        }
        | Layout::Wrapped => NOTE,
        _ => 0.0,
    };
    rows + note + GAP
}

fn wrapped_rows(members: usize) -> usize {
    members.div_ceil(WRAP).clamp(1, WRAP_ROWS)
}

/// The first recipe of each page, for a height of `room`.
fn page_starts(book: &RecipeBook, recipes: &[u32], room: f32) -> Vec<usize> {
    let mut starts = vec![0];
    let mut used = 0.0;
    for (i, &n) in recipes.iter().enumerate() {
        let height = book.recipe(n).map_or(0.0, block_height);
        if used > 0.0 && used + height > room {
            starts.push(i);
            used = 0.0;
        }
        used += height;
    }
    starts
}

fn recipe_view(recipe: &Recipe, n: u32, x: f32, y: f32, right: f32) -> RecipeView {
    let mut slots = Vec::new();
    let height = block_height(recipe) - GAP;
    let (columns, rows) = match recipe.layout {
        Layout::Grid { width, height, .. } => (width as usize, height as usize),
        Layout::Cooking { .. } | Layout::Row => (recipe.inputs.len().max(1), 1),
        Layout::Wrapped => (WRAP, wrapped_rows(recipe.inputs.len())),
    };
    let shown = match recipe.layout {
        Layout::Wrapped => recipe.inputs.len().min(WRAP * WRAP_ROWS),
        _ => columns * rows,
    };
    for i in 0..shown {
        slots.push(SlotView {
            at: [
                x + (i % columns) as f32 * SLOT,
                y + (i / columns) as f32 * SLOT,
            ],
            source: SlotSource::Input(i),
        });
    }
    let grid_width = columns as f32 * SLOT;
    let grid_height = rows as f32 * SLOT;
    let middle = y + (grid_height - SLOT) / 2.0;
    let has_output = !recipe.outputs.is_empty();
    if has_output {
        slots.push(SlotView {
            at: [x + grid_width + ARROW + 2.0, middle],
            source: SlotSource::Output,
        });
    }
    let bookmark = [right - BUTTON, y, BUTTON, BUTTON];
    RecipeView {
        recipe: n,
        slots,
        arrow: has_output.then_some([x + grid_width + 4.0, middle + 5.0]),
        note_at: [x, y + height - NOTE + 2.0],
        place: (recipe.book_id >= 0).then_some([right - 2.0 * BUTTON - 2.0, y, BUTTON, BUTTON]),
        bookmark,
    }
}

/// Lays out the page, and keeps its tab and page in range.
pub fn layout(lookup: &mut Lookup, book: &RecipeBook, screen: &ScreenInfo) -> LayerView {
    let width = MAX_WIDTH.min(screen.window[0] - 8.0).max(0.0);
    let height = MAX_HEIGHT.min(screen.window[1] - 8.0).max(0.0);
    let x = ((screen.window[0] - width) / 2.0).floor();
    let y = ((screen.window[1] - height) / 2.0).floor();
    let button = |bx: f32, by: f32| [bx, by, BUTTON, BUTTON];

    lookup.tab = lookup.tab.min(lookup.found.len().saturating_sub(1));
    let fit = (((width - 8.0) / TAB).floor() as usize).max(1);
    let first_tab = (lookup.tab + 1).saturating_sub(fit);
    let tabs = lookup
        .found
        .iter()
        .enumerate()
        .skip(first_tab)
        .take(fit)
        .map(|(i, _)| {
            let column = (i - first_tab) as f32;
            (i, [x + 4.0 + column * TAB, y + 20.0, TAB - 2.0, TAB - 2.0])
        })
        .collect();

    let top = y + 42.0;
    let room = (height - 42.0 - 18.0).max(SLOT);
    let recipes: &[u32] = lookup.current().map_or(&[], |f| f.recipes.as_slice());
    let starts = page_starts(book, recipes, room);
    let page = lookup.page.min(starts.len() - 1);
    let end = starts.get(page + 1).copied().unwrap_or(recipes.len());
    let mut views = Vec::new();
    let mut at = top;
    for &n in &recipes[starts[page]..end] {
        if let Some(recipe) = book.recipe(n) {
            views.push(recipe_view(recipe, n, x + 8.0, at, x + width - 4.0));
            at += block_height(recipe);
        }
    }
    let page_count = starts.len();
    lookup.page = page;
    let footer = y + height - 15.0;
    let center = x + width / 2.0;
    LayerView {
        panel: [x, y, width, height],
        back: button(x + 4.0, y + 4.0),
        forward: button(x + 18.0, y + 4.0),
        close: button(x + width - 16.0, y + 4.0),
        title_at: [x + 34.0, y + 6.0],
        tabs,
        previous_page: button(center - 40.0, footer),
        next_page: button(center + 28.0, footer),
        page_at: [center - 12.0, footer + 2.0],
        page_count,
        page,
        recipes: views,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::ScreenKind;
    use crate::ingredient::RawItem;
    use crate::recipe::{RawKind, RawRecipe, Tables};

    fn screen() -> ScreenInfo {
        ScreenInfo {
            kind: ScreenKind::Inventory,
            rect: [152.0, 52.0, 176.0, 166.0],
            window: [480.0, 270.0],
            mouse: [0.0, 0.0],
        }
    }

    fn crafting(id: &str, book_id: i32) -> RawRecipe {
        RawRecipe {
            id: id.to_owned(),
            kind: RawKind::CraftingShaped,
            width: 3,
            height: 3,
            slots: vec![vec![Stack::of("m:a")]; 9],
            results: vec![Stack::of("m:out")],
            cooking_time: 0,
            experience: 0.0,
            book_id,
        }
    }

    fn book(count: usize) -> RecipeBook {
        let recipes = (0..count)
            .map(|n| crafting(&format!("r{n}"), n as i32))
            .collect();
        RecipeBook::build(
            &Tables {
                recipes,
                ..Tables::default()
            },
            false,
        )
    }

    fn lookup(book: &RecipeBook) -> Lookup {
        Lookup::of(book, &Stack::of("m:out"), Role::Output).expect("found")
    }

    #[test]
    fn it_stays_inside_the_window() {
        let book = book(1);
        let mut lookup = lookup(&book);
        let view = layout(&mut lookup, &book, &screen());
        let [x, y, w, h] = view.panel;
        assert!(x >= 0.0 && y >= 0.0 && x + w <= 480.0 && y + h <= 270.0);
    }

    #[test]
    fn recipes_that_do_not_fit_go_to_more_pages_and_none_is_lost() {
        let book = book(20);
        let mut lookup = lookup(&book);
        let view = layout(&mut lookup, &book, &screen());
        assert!(view.page_count > 1);
        let mut seen = Vec::new();
        for page in 0..view.page_count {
            lookup.page = page;
            let view = layout(&mut lookup, &book, &screen());
            seen.extend(view.recipes.iter().map(|r| r.recipe));
        }
        assert_eq!(seen, (0..20).collect::<Vec<u32>>());
    }

    #[test]
    fn a_page_past_the_end_comes_back_in_range() {
        let book = book(2);
        let mut lookup = lookup(&book);
        lookup.page = 50;
        let view = layout(&mut lookup, &book, &screen());
        assert_eq!(view.page, view.page_count - 1);
        assert_eq!(lookup.page, view.page);
    }

    #[test]
    fn a_grid_recipe_has_its_slots_and_the_result_after_an_arrow() {
        let book = book(1);
        let mut lookup = lookup(&book);
        let view = layout(&mut lookup, &book, &screen());
        let recipe = &view.recipes[0];
        assert_eq!(recipe.slots.len(), 10);
        assert_eq!(recipe.slots[9].source, SlotSource::Output);
        assert!(recipe.arrow.is_some() && recipe.place.is_some());
        let [x, y] = recipe.slots[4].at;
        assert_eq!(
            view.slot_at([x + 1.0, y + 1.0]),
            Some((0, SlotSource::Input(4)))
        );
        assert_eq!(view.slot_at([0.0, 0.0]), None);
    }

    #[test]
    fn only_a_recipe_of_the_book_can_be_placed() {
        let book = RecipeBook::build(
            &Tables {
                recipes: vec![crafting("r", -1)],
                ..Tables::default()
            },
            false,
        );
        let mut lookup = lookup(&book);
        let view = layout(&mut lookup, &book, &screen());
        assert_eq!(view.recipes[0].place, None);
    }

    #[test]
    fn a_wrapped_recipe_shows_its_members_with_no_arrow() {
        let items = (0..3)
            .map(|n| RawItem {
                id: format!("m:{n}"),
                name: "x".into(),
                tags: vec!["m:t".into()],
            })
            .collect();
        let book = RecipeBook::build(
            &Tables {
                items,
                ..Tables::default()
            },
            true,
        );
        let mut lookup = Lookup::of(&book, &Stack::of("m:1"), Role::Input).expect("found");
        let view = layout(&mut lookup, &book, &screen());
        assert_eq!(view.recipes[0].slots.len(), 3);
        assert_eq!(view.recipes[0].arrow, None);
    }

    #[test]
    fn tabs_wrap_around_and_reset_the_page() {
        let book = book(1);
        let mut lookup = Lookup {
            found: vec![
                Found {
                    category: 0,
                    recipes: vec![0],
                },
                Found {
                    category: 1,
                    recipes: vec![0],
                },
            ],
            page: 3,
            ..lookup(&book)
        };
        lookup.step_tab(false);
        assert_eq!((lookup.tab, lookup.page), (1, 0));
        lookup.step_tab(true);
        assert_eq!(lookup.tab, 0);
        lookup.select_tab(1);
        assert_eq!(lookup.tab, 1);
        lookup.select_tab(9);
        assert_eq!(lookup.tab, 1);
    }

    #[test]
    fn a_tiny_window_lays_out_without_panicking() {
        let book = book(3);
        let mut lookup = lookup(&book);
        let tiny = ScreenInfo {
            window: [10.0, 10.0],
            ..screen()
        };
        let view = layout(&mut lookup, &book, &tiny);
        assert!(view.page_count >= 1);
    }

    #[test]
    fn nothing_found_gives_no_lookup() {
        assert!(Lookup::of(&book(1), &Stack::of("m:none"), Role::Output).is_none());
        assert!(Lookup::single(&book(1), 99).is_none());
    }
}
