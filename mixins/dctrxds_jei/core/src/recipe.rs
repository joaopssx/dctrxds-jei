//! Categories and recipes, and finding them from an item. JEI's index
//! (`RecipeIngredientRoleMap`) is by role; this one is too: what a recipe makes,
//! what it uses, and the station that makes it.

use crate::ingredient::{RawItem, Stack};
use std::collections::{BTreeMap, HashMap};

/// How the game classifies a recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawKind {
    CraftingShaped,
    CraftingShapeless,
    Smelting,
    Blasting,
    Smoking,
    Campfire,
    Stonecutting,
    Smithing,
    Brewing,
}

/// A recipe as the game gives it.
#[derive(Clone, Debug, PartialEq)]
pub struct RawRecipe {
    pub id: String,
    pub kind: RawKind,
    pub width: u32,
    pub height: u32,
    /// Each slot's options; an empty list is an empty cell.
    pub slots: Vec<Vec<Stack>>,
    pub results: Vec<Stack>,
    pub cooking_time: u32,
    pub experience: f32,
    /// The recipe book's id, or -1.
    pub book_id: i32,
}

/// A category another mod declares ([`crate::categories`]), with its recipes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CustomCategory {
    pub id: String,
    pub title: String,
    pub icon: Stack,
    pub catalysts: Vec<Stack>,
    pub recipes: Vec<CustomRecipe>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CustomRecipe {
    pub id: String,
    pub inputs: Vec<Vec<Stack>>,
    pub outputs: Vec<Stack>,
}

/// Everything the game tells about its items and recipes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tables {
    pub items: Vec<RawItem>,
    pub recipes: Vec<RawRecipe>,
    pub custom: Vec<CustomCategory>,
}

/// How a recipe is laid out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    Grid {
        width: u32,
        height: u32,
        shapeless: bool,
    },
    /// One input, an arrow, the result, and time and experience.
    Cooking { ticks: u32, xp_hundredths: u32 },
    /// Inputs in a row, an arrow, the result.
    Row,
    /// Many members, wrapped: a tag's items, or a category of another mod.
    Wrapped,
}

/// A category's title: translated, or as another mod wrote it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Title {
    Crafting,
    Stonecutting,
    Smelting,
    Smoking,
    Blasting,
    Campfire,
    Smithing,
    Brewing,
    Tags,
    Literal(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Category {
    pub id: String,
    pub title: Title,
    pub icon: Stack,
    /// The stations that make its recipes, JEI's crafting stations.
    pub catalysts: Vec<Stack>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recipe {
    pub id: String,
    pub category: usize,
    pub layout: Layout,
    pub inputs: Vec<Vec<Stack>>,
    pub outputs: Vec<Stack>,
    pub book_id: i32,
}

/// Which way an item is looked up.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    /// What makes it: the R key.
    Output,
    /// What it is used in: the U key.
    Input,
}

/// One tab of a lookup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub category: usize,
    pub recipes: Vec<u32>,
}

/// All recipes, indexed.
#[derive(Debug, Default)]
pub struct RecipeBook {
    categories: Vec<Category>,
    recipes: Vec<Recipe>,
    by_input: HashMap<String, Vec<u32>>,
    by_output: HashMap<String, Vec<u32>>,
    by_id: HashMap<String, u32>,
}

fn vanilla_category(id: &str, title: Title, station: &str) -> Category {
    Category {
        id: id.to_owned(),
        title,
        icon: Stack::of(station),
        catalysts: vec![Stack::of(station)],
    }
}

// In the order of JEI's VanillaPlugin.registerCategories.
const CRAFTING: usize = 0;
const STONECUTTING: usize = 1;
const SMELTING: usize = 2;
const SMOKING: usize = 3;
const BLASTING: usize = 4;
const CAMPFIRE: usize = 5;
const SMITHING: usize = 6;
const BREWING: usize = 7;
const TAGS: usize = 8;

fn vanilla_categories() -> Vec<Category> {
    let mut furnace = vanilla_category("minecraft:smelting", Title::Smelting, "minecraft:furnace");
    furnace.catalysts = vec![Stack::of("minecraft:furnace")];
    let mut campfire = vanilla_category(
        "minecraft:campfire_cooking",
        Title::Campfire,
        "minecraft:campfire",
    );
    campfire
        .catalysts
        .push(Stack::of("minecraft:soul_campfire"));
    let mut crafting = vanilla_category(
        "minecraft:crafting",
        Title::Crafting,
        "minecraft:crafting_table",
    );
    crafting.catalysts.push(Stack::of("minecraft:crafter"));
    vec![
        crafting,
        vanilla_category(
            "minecraft:stonecutting",
            Title::Stonecutting,
            "minecraft:stonecutter",
        ),
        furnace,
        vanilla_category("minecraft:smoking", Title::Smoking, "minecraft:smoker"),
        vanilla_category(
            "minecraft:blasting",
            Title::Blasting,
            "minecraft:blast_furnace",
        ),
        campfire,
        vanilla_category(
            "minecraft:smithing",
            Title::Smithing,
            "minecraft:smithing_table",
        ),
        vanilla_category(
            "minecraft:brewing",
            Title::Brewing,
            "minecraft:brewing_stand",
        ),
        Category {
            id: "jei:tags".to_owned(),
            title: Title::Tags,
            icon: Stack::of("minecraft:name_tag"),
            catalysts: Vec::new(),
        },
    ]
}

/// A shapeless recipe's grid, as the game's recipe viewer lays one out.
fn shapeless_grid(slots: usize) -> (u32, u32) {
    let slots = slots as u32;
    let width = match slots {
        0 | 1 => 1,
        2..=4 => 2,
        _ => 3,
    };
    (width, slots.div_ceil(width).max(1))
}

fn convert(raw: &RawRecipe) -> Recipe {
    let (category, layout) = match raw.kind {
        RawKind::CraftingShaped => (
            CRAFTING,
            Layout::Grid {
                width: raw.width.clamp(1, 3),
                height: raw.height.clamp(1, 3),
                shapeless: false,
            },
        ),
        RawKind::CraftingShapeless => {
            let (width, height) = shapeless_grid(raw.slots.len());
            (
                CRAFTING,
                Layout::Grid {
                    width,
                    height,
                    shapeless: true,
                },
            )
        }
        RawKind::Smelting | RawKind::Blasting | RawKind::Smoking | RawKind::Campfire => {
            let category = match raw.kind {
                RawKind::Blasting => BLASTING,
                RawKind::Smoking => SMOKING,
                RawKind::Campfire => CAMPFIRE,
                _ => SMELTING,
            };
            (
                category,
                Layout::Cooking {
                    ticks: raw.cooking_time,
                    xp_hundredths: (raw.experience * 100.0).round() as u32,
                },
            )
        }
        RawKind::Stonecutting => (STONECUTTING, Layout::Row),
        RawKind::Smithing => (SMITHING, Layout::Row),
        RawKind::Brewing => (BREWING, Layout::Row),
    };
    Recipe {
        id: raw.id.clone(),
        category,
        layout,
        inputs: raw.slots.clone(),
        outputs: raw.results.clone(),
        book_id: raw.book_id,
    }
}

impl RecipeBook {
    /// Indexes the game's recipes, a tag category when `show_tags`, and the
    /// categories of other mods.
    pub fn build(tables: &Tables, show_tags: bool) -> Self {
        let mut book = Self {
            categories: vanilla_categories(),
            ..Self::default()
        };
        for raw in &tables.recipes {
            book.add(convert(raw));
        }
        if show_tags {
            let mut tags: BTreeMap<&str, Vec<Vec<Stack>>> = BTreeMap::new();
            for item in &tables.items {
                for tag in &item.tags {
                    tags.entry(tag).or_default().push(vec![Stack::of(&item.id)]);
                }
            }
            for (tag, members) in tags {
                book.add(Recipe {
                    id: format!("#{tag}"),
                    category: TAGS,
                    layout: Layout::Wrapped,
                    inputs: members,
                    outputs: Vec::new(),
                    book_id: -1,
                });
            }
        }
        for custom in &tables.custom {
            let category = book.categories.len();
            book.categories.push(Category {
                id: custom.id.clone(),
                title: Title::Literal(custom.title.clone()),
                icon: custom.icon.clone(),
                catalysts: custom.catalysts.clone(),
            });
            for recipe in &custom.recipes {
                book.add(Recipe {
                    id: recipe.id.clone(),
                    category,
                    layout: Layout::Row,
                    inputs: recipe.inputs.clone(),
                    outputs: recipe.outputs.clone(),
                    book_id: -1,
                });
            }
        }
        book
    }

    fn add(&mut self, recipe: Recipe) {
        let n = self.recipes.len() as u32;
        self.by_id.entry(recipe.id.clone()).or_insert(n);
        let add_to = |index: &mut HashMap<String, Vec<u32>>, stack: &Stack| {
            let list = index.entry(stack.item.clone()).or_default();
            if list.last() != Some(&n) {
                list.push(n);
            }
        };
        for stack in recipe.inputs.iter().flatten() {
            add_to(&mut self.by_input, stack);
        }
        for stack in &recipe.outputs {
            add_to(&mut self.by_output, stack);
        }
        self.recipes.push(recipe);
    }

    pub fn categories(&self) -> &[Category] {
        &self.categories
    }

    pub fn recipes(&self) -> &[Recipe] {
        &self.recipes
    }

    pub fn recipe(&self, n: u32) -> Option<&Recipe> {
        self.recipes.get(n as usize)
    }

    pub fn recipe_by_id(&self, id: &str) -> Option<u32> {
        self.by_id.get(id).copied()
    }

    /// The tabs of a lookup, in the categories' order; none without results.
    /// A station looked up for its uses gets its whole category, as in JEI.
    pub fn lookup(&self, focus: &Stack, role: Role) -> Vec<Found> {
        let index = match role {
            Role::Input => &self.by_input,
            Role::Output => &self.by_output,
        };
        let matching = index.get(&focus.item).map_or(&[][..], Vec::as_slice);
        let mut per_category: BTreeMap<usize, Vec<u32>> = BTreeMap::new();
        for &n in matching {
            let recipe = &self.recipes[n as usize];
            let found = match role {
                Role::Output => recipe.outputs.iter().any(|s| s.matches_focus(focus)),
                Role::Input => recipe
                    .inputs
                    .iter()
                    .flatten()
                    .any(|s| s.matches_focus(focus)),
            };
            if found {
                per_category.entry(recipe.category).or_default().push(n);
            }
        }
        if role == Role::Input {
            for (category, details) in self.categories.iter().enumerate() {
                if details.catalysts.iter().any(|s| s.matches_focus(focus)) {
                    let all = (0..self.recipes.len() as u32)
                        .filter(|&n| self.recipes[n as usize].category == category)
                        .collect();
                    per_category.insert(category, all);
                }
            }
        }
        per_category
            .into_iter()
            .filter(|(_, recipes)| !recipes.is_empty())
            .map(|(category, recipes)| Found { category, recipes })
            .collect()
    }

    /// The tab holding just one recipe, for a recipe bookmark.
    pub fn single(&self, n: u32) -> Option<Found> {
        Some(Found {
            category: self.recipe(n)?.category,
            recipes: vec![n],
        })
    }

    /// Every stack with components the recipes mention, once each, in their
    /// order: the variants (potions, tipped arrows) the item table can't list.
    pub fn variants(&self) -> Vec<Stack> {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for recipe in &self.recipes {
            for stack in recipe.inputs.iter().flatten().chain(&recipe.outputs) {
                if !stack.components.is_empty() && seen.insert(stack.one()) {
                    out.push(stack.one());
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(id: &str, kind: RawKind, slots: &[&[&str]], results: &[&str]) -> RawRecipe {
        RawRecipe {
            id: id.to_owned(),
            kind,
            width: 0,
            height: 0,
            slots: slots
                .iter()
                .map(|s| s.iter().map(|i| Stack::of(i)).collect())
                .collect(),
            results: results.iter().map(|i| Stack::of(i)).collect(),
            cooking_time: 0,
            experience: 0.0,
            book_id: -1,
        }
    }

    fn book() -> RecipeBook {
        let planks = raw(
            "t:planks",
            RawKind::CraftingShapeless,
            &[&["m:oak_log", "m:birch_log"]],
            &["m:planks"],
        );
        let sticks = RawRecipe {
            width: 1,
            height: 2,
            ..raw(
                "t:sticks",
                RawKind::CraftingShaped,
                &[&["m:planks"], &["m:planks"]],
                &["m:stick"],
            )
        };
        let charcoal = RawRecipe {
            cooking_time: 200,
            experience: 0.15,
            ..raw(
                "t:charcoal",
                RawKind::Smelting,
                &[&["m:oak_log"]],
                &["m:charcoal"],
            )
        };
        let items = vec![
            RawItem {
                id: "m:oak_log".into(),
                name: "Oak".into(),
                tags: vec!["m:logs".into()],
            },
            RawItem {
                id: "m:birch_log".into(),
                name: "Birch".into(),
                tags: vec!["m:logs".into()],
            },
        ];
        let tables = Tables {
            items,
            recipes: vec![planks, sticks, charcoal],
            custom: vec![CustomCategory {
                id: "x:crush".into(),
                title: "Crushing".into(),
                icon: Stack::of("x:crusher"),
                catalysts: vec![Stack::of("x:crusher")],
                recipes: vec![CustomRecipe {
                    id: "x:r".into(),
                    inputs: vec![vec![Stack::of("m:oak_log")]],
                    outputs: vec![Stack::of("x:dust")],
                }],
            }],
        };
        RecipeBook::build(&tables, true)
    }

    fn ids(book: &RecipeBook, found: &[Found]) -> Vec<(String, Vec<String>)> {
        found
            .iter()
            .map(|f| {
                (
                    book.categories()[f.category].id.clone(),
                    f.recipes
                        .iter()
                        .map(|&n| book.recipes()[n as usize].id.clone())
                        .collect(),
                )
            })
            .collect()
    }

    #[test]
    fn recipes_of_an_item_are_those_that_make_it() {
        let book = book();
        let found = book.lookup(&Stack::of("m:planks"), Role::Output);
        assert_eq!(
            ids(&book, &found),
            [("minecraft:crafting".to_owned(), vec!["t:planks".to_owned()])]
        );
    }

    #[test]
    fn uses_of_an_item_span_categories_in_their_order() {
        let book = book();
        let found = ids(&book, &book.lookup(&Stack::of("m:oak_log"), Role::Input));
        let categories: Vec<&str> = found.iter().map(|(c, _)| c.as_str()).collect();
        assert_eq!(
            categories,
            [
                "minecraft:crafting",
                "minecraft:smelting",
                "jei:tags",
                "x:crush"
            ]
        );
    }

    #[test]
    fn an_item_used_twice_in_a_recipe_finds_it_once() {
        let book = book();
        let found = book.lookup(&Stack::of("m:planks"), Role::Input);
        assert_eq!(found[0].recipes.len(), 1);
    }

    #[test]
    fn a_station_shows_its_whole_category_as_uses() {
        let book = book();
        let found = ids(
            &book,
            &book.lookup(&Stack::of("minecraft:furnace"), Role::Input),
        );
        assert_eq!(
            found,
            [(
                "minecraft:smelting".to_owned(),
                vec!["t:charcoal".to_owned()]
            )]
        );
        let custom = ids(&book, &book.lookup(&Stack::of("x:crusher"), Role::Input));
        assert_eq!(custom[0].0, "x:crush");
        assert!(
            book.lookup(&Stack::of("minecraft:furnace"), Role::Output)
                .is_empty()
        );
    }

    #[test]
    fn nothing_found_gives_no_tabs() {
        assert!(book().lookup(&Stack::of("m:none"), Role::Input).is_empty());
    }

    #[test]
    fn components_narrow_a_focus_but_not_the_other_way_round() {
        let swift = Stack::with_components("m:potion", "[swift]");
        let brew = RawRecipe {
            results: vec![swift.clone()],
            ..raw(
                "t:brew",
                RawKind::Brewing,
                &[&["m:water"], &["m:sugar"]],
                &[],
            )
        };
        let tables = Tables {
            recipes: vec![brew],
            ..Tables::default()
        };
        let book = RecipeBook::build(&tables, false);
        assert_eq!(book.lookup(&swift, Role::Output).len(), 1);
        assert_eq!(book.lookup(&Stack::of("m:potion"), Role::Output).len(), 1);
        let slow = Stack::with_components("m:potion", "[slow]");
        assert!(book.lookup(&slow, Role::Output).is_empty());
        assert_eq!(book.variants(), [swift]);
    }

    #[test]
    fn tag_recipes_can_be_turned_off() {
        let tables = Tables {
            items: vec![RawItem {
                id: "m:a".into(),
                name: "A".into(),
                tags: vec!["m:t".into()],
            }],
            ..Tables::default()
        };
        assert_eq!(RecipeBook::build(&tables, true).recipes().len(), 1);
        assert!(RecipeBook::build(&tables, false).recipes().is_empty());
    }

    #[test]
    fn layouts_follow_the_kind() {
        let book = book();
        let layout = |id: &str| book.recipes()[book.recipe_by_id(id).unwrap() as usize].layout;
        assert_eq!(
            layout("t:sticks"),
            Layout::Grid {
                width: 1,
                height: 2,
                shapeless: false
            }
        );
        assert_eq!(
            layout("t:planks"),
            Layout::Grid {
                width: 1,
                height: 1,
                shapeless: true
            }
        );
        assert_eq!(
            layout("t:charcoal"),
            Layout::Cooking {
                ticks: 200,
                xp_hundredths: 15
            }
        );
    }

    #[test]
    fn shapeless_grids_grow_as_the_game_lays_them_out() {
        assert_eq!(shapeless_grid(1), (1, 1));
        assert_eq!(shapeless_grid(3), (2, 2));
        assert_eq!(shapeless_grid(4), (2, 2));
        assert_eq!(shapeless_grid(5), (3, 2));
        assert_eq!(shapeless_grid(9), (3, 3));
    }

    #[test]
    fn a_single_recipe_tab_can_be_made_for_a_bookmark() {
        let book = book();
        let n = book.recipe_by_id("t:charcoal").unwrap();
        assert_eq!(book.single(n).unwrap().recipes, [n]);
        assert!(book.single(9999).is_none());
    }
}
