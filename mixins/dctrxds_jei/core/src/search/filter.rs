// The evaluation follows Just Enough Items (https://github.com/mezz/JustEnoughItems),
// Gui/src/main/java/mezz/jei/gui/ingredients/IngredientFilter.java.
// Copyright (c) 2014-2015 mezz. MIT licence: LICENSES/JustEnoughItems-MIT.txt.

//! Answers a search text with the set of ingredients it finds.

use super::index::{Bits, TextIndex, TextIndexBuilder};
use super::token::{split_by_operators, tokenize};
use super::{Field, SearchMode, SearchModes};

/// The texts one ingredient is found by, already in lower case.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchTexts {
    pub names: Vec<String>,
    pub mods: Vec<String>,
    pub tags: Vec<String>,
    pub tooltips: Vec<String>,
    pub creative_tabs: Vec<String>,
    pub colors: Vec<String>,
    pub identifiers: Vec<String>,
}

impl SearchTexts {
    fn of(&self, field: Field) -> &[String] {
        match field {
            Field::Names => &self.names,
            Field::Mods => &self.mods,
            Field::Tags => &self.tags,
            Field::Tooltips => &self.tooltips,
            Field::CreativeTabs => &self.creative_tabs,
            Field::Colors => &self.colors,
            Field::Identifiers => &self.identifiers,
        }
    }
}

/// The search over a fixed list of ingredients.
#[derive(Debug)]
pub struct Filter {
    len: usize,
    modes: SearchModes,
    indexes: [TextIndex; Field::COUNT],
}

impl Filter {
    /// Indexes `texts`, one per ingredient. A disabled field isn't indexed.
    pub fn new(texts: &[SearchTexts], modes: SearchModes) -> Self {
        let indexes = Field::ALL.map(|field| {
            let mut builder = TextIndexBuilder::default();
            if modes.of(field) != SearchMode::Disabled {
                for (n, ingredient) in texts.iter().enumerate() {
                    for text in ingredient.of(field) {
                        builder.put(text, n as u32);
                    }
                }
            }
            builder.build()
        });
        Self {
            len: texts.len(),
            modes,
            indexes,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn results(&self, word: &str, field: Option<Field>) -> Bits {
        let mut out = Bits::empty(self.len);
        match field {
            Some(field) => self.indexes[field.index()].search_into(word, &mut out),
            None => {
                for field in Field::ALL {
                    if self.modes.of(field) == SearchMode::Enabled {
                        self.indexes[field.index()].search_into(word, &mut out);
                    }
                }
            }
        }
        out
    }

    /// Every ingredient `text` finds. Whitespace is AND, `|` is OR and `-`
    /// excludes. No search words at all finds everything.
    pub fn search(&self, text: &str) -> Bits {
        let lower = text.to_lowercase();
        let tokens = tokenize(&lower);
        let mut any_group = false;
        let mut union = Bits::empty(self.len);
        for group in split_by_operators(&tokens) {
            let mut to_search = Vec::new();
            let mut to_remove = Vec::new();
            for token in &group {
                if let Some(parsed) = self.modes.parse_token(&token.text) {
                    if token.exclusion {
                        to_remove.push(parsed);
                    } else {
                        to_search.push(parsed);
                    }
                }
            }
            if to_search.is_empty() && to_remove.is_empty() {
                continue;
            }
            any_group = true;
            let mut results: Option<Bits> = None;
            for (word, field) in to_search {
                let found = self.results(word, field);
                match results.as_mut() {
                    Some(so_far) => so_far.intersect_with(&found),
                    None => results = Some(found),
                }
            }
            let mut results = results.unwrap_or_else(|| Bits::empty(self.len));
            // As JEI: no results and exclusions start from every ingredient,
            // whether words were given or not. `zzz -iron` lists all but iron.
            if results.is_empty() && !to_remove.is_empty() {
                results = Bits::full(self.len);
            }
            for (word, field) in to_remove {
                results.subtract(&self.results(word, field));
            }
            union.union_with(&results);
        }
        if any_group {
            union
        } else {
            Bits::full(self.len)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small table: the expected answers below are written by hand from
    /// JEI's rules, not computed by the code under test.
    fn table() -> Vec<SearchTexts> {
        let item = |name: &str, namespace: &str, tags: &[&str], id: &str| SearchTexts {
            names: vec![name.to_owned()],
            mods: vec![namespace.to_owned()],
            tags: tags.iter().map(|t| (*t).to_owned()).collect(),
            tooltips: vec![format!("tooltip of {name}")],
            creative_tabs: vec!["tab".to_owned()],
            colors: vec!["blue".to_owned()],
            identifiers: vec![id.to_owned()],
        };
        vec![
            item(
                "iron sword",
                "minecraft",
                &["swords"],
                "minecraft:iron_sword",
            ),
            item(
                "diamond sword",
                "minecraft",
                &["swords"],
                "minecraft:diamond_sword",
            ),
            item("oak log", "minecraft", &["logs"], "minecraft:oak_log"),
            item(
                "oak planks",
                "minecraft",
                &["planks"],
                "minecraft:oak_planks",
            ),
            item("iron gear", "create", &[], "create:iron_gear"),
            item("the \"best\" sword", "other", &[], "other:best"),
        ]
    }

    fn found(text: &str) -> Vec<usize> {
        let filter = Filter::new(&table(), SearchModes::default());
        let bits = filter.search(text);
        (0..filter.len()).filter(|&n| bits.contains(n)).collect()
    }

    #[test]
    fn the_cases_in_the_roadmap_answer_as_jei_does() {
        assert_eq!(found("sword -iron"), [1, 5]);
        assert_eq!(found("@minecraft #logs|#planks"), [2, 3]);
    }

    #[test]
    fn no_words_finds_everything() {
        for text in ["", "   ", "@", "#", "|", " | ", "-", "\"\""] {
            assert_eq!(found(text).len(), 6, "{text:?}");
        }
    }

    #[test]
    fn whitespace_is_and_and_a_pipe_is_or() {
        assert_eq!(found("oak log"), [2]);
        assert_eq!(found("log|planks"), [2, 3]);
        assert_eq!(found("iron gear|diamond"), [1, 4]);
    }

    #[test]
    fn upper_case_in_the_search_is_ignored() {
        assert_eq!(found("OAK LOG"), [2]);
    }

    #[test]
    fn quotes_keep_a_phrase_together() {
        assert_eq!(found("\"oak log\""), [2]);
        assert_eq!(found("\"log oak\""), Vec::<usize>::new());
        assert_eq!(found("\"the \\\"best\\\"\""), [5]);
    }

    #[test]
    fn a_prefix_searches_only_its_field() {
        assert_eq!(found("@create"), [4]);
        assert_eq!(found("#swords"), [0, 1]);
        assert_eq!(found("$tooltip"), [0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn a_require_prefix_field_is_not_searched_by_plain_words() {
        assert_eq!(found("create"), Vec::<usize>::new());
        assert_eq!(found("logs"), Vec::<usize>::new());
    }

    #[test]
    fn plain_words_also_search_the_enabled_tooltip_field() {
        assert_eq!(found("tooltip of oak log"), [2]);
    }

    #[test]
    fn a_disabled_prefix_is_part_of_the_word_and_finds_nothing_here() {
        assert_eq!(found("%tab"), Vec::<usize>::new());
        assert_eq!(found("&minecraft"), Vec::<usize>::new());
    }

    #[test]
    fn a_field_turned_on_is_searched() {
        let modes = SearchModes {
            identifiers: SearchMode::RequirePrefix,
            creative_tabs: SearchMode::Enabled,
            ..SearchModes::default()
        };
        let filter = Filter::new(&table(), modes);
        assert_eq!(filter.search("&create:").count(), 1);
        assert_eq!(filter.search("tab").count(), 6);
    }

    #[test]
    fn exclusions_alone_start_from_everything() {
        assert_eq!(found("-sword"), [2, 3, 4]);
        assert_eq!(found("-sword -log"), [3, 4]);
    }

    // JEI's own behaviour in IngredientFilter.getSearchResults: words that
    // find nothing, with an exclusion, start over from every ingredient.
    #[test]
    fn words_finding_nothing_with_an_exclusion_start_over_as_in_jei() {
        assert_eq!(found("zzz -sword"), [2, 3, 4]);
        assert_eq!(found("zzz"), Vec::<usize>::new());
    }

    #[test]
    fn groups_are_united_without_repeats() {
        assert_eq!(found("sword|iron"), [0, 1, 4, 5]);
        assert_eq!(found("|oak"), [2, 3]);
        assert_eq!(found("oak|"), [2, 3]);
    }
}
