//! What the list shows: stacks, ingredients and how they are ordered.

use crate::search::filter::SearchTexts;
use std::cmp::Ordering;

/// An item, a count and its components beyond the item's own, in the game's
/// item syntax (`[minecraft:potion_contents={potion:"minecraft:swiftness"}]`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Stack {
    pub item: String,
    pub count: u32,
    pub components: String,
}

impl Stack {
    pub fn of(item: &str) -> Self {
        Self {
            item: item.to_owned(),
            count: 1,
            components: String::new(),
        }
    }

    pub fn with_components(item: &str, components: &str) -> Self {
        Self {
            components: components.to_owned(),
            ..Self::of(item)
        }
    }

    /// The same thing as `other`, whatever the counts.
    pub fn same_kind(&self, other: &Self) -> bool {
        self.item == other.item && self.components == other.components
    }

    /// Whether this stack answers a search for `focus`: the same item and, if
    /// the focus names components, the same components.
    pub fn matches_focus(&self, focus: &Self) -> bool {
        self.item == focus.item
            && (focus.components.is_empty() || self.components == focus.components)
    }

    pub fn namespace(&self) -> &str {
        self.item.split_once(':').map_or("minecraft", |(ns, _)| ns)
    }

    /// One stack of this kind: the identity of a list entry or a bookmark.
    pub fn one(&self) -> Self {
        Self {
            count: 1,
            ..self.clone()
        }
    }
}

/// An item of the game's table.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RawItem {
    pub id: String,
    pub name: String,
    pub tags: Vec<String>,
}

/// One entry of the list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ingredient {
    pub stack: Stack,
    pub name: String,
    pub tags: Vec<String>,
    /// Its place in the game's own order; JEI's creative menu position.
    pub created: u32,
}

impl Ingredient {
    pub fn search_texts(&self) -> SearchTexts {
        let lower = |t: &str| t.to_lowercase();
        let name = lower(&self.name);
        SearchTexts {
            names: vec![name.clone()],
            mods: vec![lower(self.stack.namespace())],
            // As JEI: the path of the tag, not its namespace.
            tags: self
                .tags
                .iter()
                .map(|t| lower(t.split_once(':').map_or(t.as_str(), |(_, path)| path)))
                .collect(),
            // The game offers no tooltip text for a list of items: the name is
            // the only line. See docs/PORTING_NOTES.md.
            tooltips: vec![name],
            creative_tabs: Vec::new(),
            colors: Vec::new(),
            identifiers: vec![lower(&self.stack.item)],
        }
    }
}

/// The stages of JEI's `IngredientSortStage` that have data here. The others
/// have nothing to compare, see docs/PORTING_NOTES.md.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortStage {
    ModName,
    Alphabetical,
    CreativeMenu,
    Tag,
}

impl SortStage {
    /// JEI's default order, without its ingredient type stage: there is one type.
    pub const DEFAULT: [Self; 2] = [Self::ModName, Self::CreativeMenu];

    pub fn name(self) -> &'static str {
        match self {
            Self::ModName => "mod_name",
            Self::Alphabetical => "alphabetical",
            Self::CreativeMenu => "creative_menu",
            Self::Tag => "tag",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "mod_name" => Some(Self::ModName),
            "alphabetical" => Some(Self::Alphabetical),
            "creative_menu" => Some(Self::CreativeMenu),
            "tag" => Some(Self::Tag),
            _ => None,
        }
    }

    fn compare(self, a: &Ingredient, b: &Ingredient) -> Ordering {
        match self {
            // As JEI's `ModNameSortingConfig`: Minecraft, then by name.
            Self::ModName => {
                let (x, y) = (a.stack.namespace(), b.stack.namespace());
                (x != "minecraft").cmp(&(y != "minecraft")).then(x.cmp(y))
            }
            Self::Alphabetical => a.name.cmp(&b.name),
            Self::CreativeMenu => a.created.cmp(&b.created),
            // Tagged ones first, then by tag. JEI picks the tag with the most
            // items; that count isn't known here, so it is the first by name.
            Self::Tag => a.tags.is_empty().cmp(&b.tags.is_empty()).then_with(|| {
                let tag = |i: &'_ Ingredient| i.tags.iter().min().cloned().unwrap_or_default();
                tag(a).cmp(&tag(b))
            }),
        }
    }
}

/// The ingredient numbers in display order. Ties keep the list's order.
pub fn sorted_order(ingredients: &[Ingredient], stages: &[SortStage]) -> Vec<u32> {
    let mut order: Vec<u32> = (0..ingredients.len() as u32).collect();
    order.sort_by(|&a, &b| {
        let (x, y) = (&ingredients[a as usize], &ingredients[b as usize]);
        stages
            .iter()
            .map(|stage| stage.compare(x, y))
            .find(|o| o.is_ne())
            .unwrap_or(Ordering::Equal)
    });
    order
}

/// A short readable label for a stack's components, for the variants the game
/// only tells apart by them: `potion:"minecraft:swiftness"` gives `swiftness`.
pub fn components_label(components: &str) -> Option<String> {
    let start = components.find(":\"")? + 2;
    let end = start + components[start..].find('"')?;
    let value = &components[start..end];
    let path = value.split_once(':').map_or(value, |(_, path)| path);
    (!path.is_empty()).then(|| path.replace('_', " "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ingredient(item: &str, name: &str, tags: &[&str], created: u32) -> Ingredient {
        Ingredient {
            stack: Stack::of(item),
            name: name.to_owned(),
            tags: tags.iter().map(|t| (*t).to_owned()).collect(),
            created,
        }
    }

    fn sample() -> Vec<Ingredient> {
        vec![
            ingredient("zeta:a", "Apple", &[], 3),
            ingredient("minecraft:b", "Zinc", &["minecraft:zeta"], 2),
            ingredient("alpha:c", "Mango", &["alpha:gems"], 1),
            ingredient("minecraft:d", "Bread", &[], 0),
        ]
    }

    #[test]
    fn mod_name_puts_minecraft_first_then_by_name() {
        assert_eq!(sorted_order(&sample(), &SortStage::DEFAULT), [3, 1, 2, 0]);
    }

    #[test]
    fn alphabetical_sorts_by_name() {
        assert_eq!(
            sorted_order(&sample(), &[SortStage::Alphabetical]),
            [0, 3, 2, 1]
        );
    }

    #[test]
    fn tag_puts_tagged_ones_first_and_ties_keep_their_place() {
        assert_eq!(sorted_order(&sample(), &[SortStage::Tag]), [2, 1, 0, 3]);
    }

    #[test]
    fn no_stages_keep_the_lists_order() {
        assert_eq!(sorted_order(&sample(), &[]), [0, 1, 2, 3]);
    }

    #[test]
    fn stages_round_trip_through_their_names() {
        for stage in [
            SortStage::ModName,
            SortStage::Alphabetical,
            SortStage::CreativeMenu,
            SortStage::Tag,
        ] {
            assert_eq!(SortStage::parse(stage.name()), Some(stage));
        }
        assert_eq!(SortStage::parse("armor"), None);
    }

    #[test]
    fn a_focus_without_components_matches_every_variant_of_the_item() {
        let potion = Stack::with_components("minecraft:potion", "[x]");
        assert!(potion.matches_focus(&Stack::of("minecraft:potion")));
        assert!(potion.matches_focus(&potion));
        assert!(!Stack::of("minecraft:potion").matches_focus(&potion));
        assert!(!potion.matches_focus(&Stack::of("minecraft:stick")));
    }

    #[test]
    fn search_texts_are_lower_case_and_use_the_tag_path() {
        let texts = ingredient("Mod:Thing", "Big Thing", &["c:Ingots", "bare"], 0).search_texts();
        assert_eq!(texts.names, ["big thing"]);
        assert_eq!(texts.mods, ["mod"]);
        assert_eq!(texts.tags, ["ingots", "bare"]);
        assert_eq!(texts.identifiers, ["mod:thing"]);
    }

    #[test]
    fn components_give_a_readable_label_when_they_name_something() {
        let swiftness = "[minecraft:potion_contents={potion:\"minecraft:long_swiftness\"}]";
        assert_eq!(
            components_label(swiftness).as_deref(),
            Some("long swiftness")
        );
        assert_eq!(components_label("[minecraft:damage=3]"), None);
        assert_eq!(components_label(""), None);
    }
}
