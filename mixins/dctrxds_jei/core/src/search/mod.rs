//! JEI's search: tokens, prefixes, a substring index and their evaluation.

pub mod filter;
pub mod index;
pub mod token;

/// How a kind of text can be searched.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchMode {
    /// Plain words search it too.
    Enabled,
    /// Only with its prefix.
    RequirePrefix,
    /// Never: its prefix character is just text.
    Disabled,
}

impl SearchMode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::RequirePrefix => "require_prefix",
            Self::Disabled => "disabled",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "enabled" => Some(Self::Enabled),
            "require_prefix" => Some(Self::RequirePrefix),
            "disabled" => Some(Self::Disabled),
            _ => None,
        }
    }
}

/// The kinds of text an ingredient is found by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Names,
    Mods,
    Tags,
    Tooltips,
    CreativeTabs,
    Colors,
    Identifiers,
}

impl Field {
    pub const COUNT: usize = 7;
    pub const ALL: [Self; Self::COUNT] = [
        Self::Names,
        Self::Mods,
        Self::Tags,
        Self::Tooltips,
        Self::CreativeTabs,
        Self::Colors,
        Self::Identifiers,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    fn of_prefix(c: char) -> Option<Self> {
        match c {
            '@' => Some(Self::Mods),
            '#' => Some(Self::Tags),
            '$' => Some(Self::Tooltips),
            '%' => Some(Self::CreativeTabs),
            '^' => Some(Self::Colors),
            '&' => Some(Self::Identifiers),
            _ => None,
        }
    }
}

/// The mode of each prefix. The defaults are JEI's `IngredientFilterConfig`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchModes {
    pub mods: SearchMode,
    pub tags: SearchMode,
    pub tooltips: SearchMode,
    pub creative_tabs: SearchMode,
    pub colors: SearchMode,
    pub identifiers: SearchMode,
}

impl Default for SearchModes {
    fn default() -> Self {
        Self {
            mods: SearchMode::RequirePrefix,
            tags: SearchMode::RequirePrefix,
            tooltips: SearchMode::Enabled,
            creative_tabs: SearchMode::Disabled,
            colors: SearchMode::Disabled,
            identifiers: SearchMode::Disabled,
        }
    }
}

impl SearchModes {
    pub fn of(&self, field: Field) -> SearchMode {
        match field {
            Field::Names => SearchMode::Enabled,
            Field::Mods => self.mods,
            Field::Tags => self.tags,
            Field::Tooltips => self.tooltips,
            Field::CreativeTabs => self.creative_tabs,
            Field::Colors => self.colors,
            Field::Identifiers => self.identifiers,
        }
    }

    /// What a token searches, following `ElementPrefixParser.parseToken`: the
    /// field `None` is "every enabled field". A lone prefix searches nothing.
    pub fn parse_token<'a>(&self, token: &'a str) -> Option<(&'a str, Option<Field>)> {
        let first = token.chars().next()?;
        let Some(field) = Field::of_prefix(first).filter(|&f| self.of(f) != SearchMode::Disabled)
        else {
            return Some((token, None));
        };
        let rest = &token[first.len_utf8()..];
        (!rest.is_empty()).then_some((rest, Some(field)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_prefix_in_use_selects_its_field_and_is_removed() {
        let modes = SearchModes::default();
        assert_eq!(
            modes.parse_token("@create"),
            Some(("create", Some(Field::Mods)))
        );
        assert_eq!(
            modes.parse_token("#logs"),
            Some(("logs", Some(Field::Tags)))
        );
    }

    #[test]
    fn a_disabled_prefix_is_an_ordinary_part_of_the_word() {
        let modes = SearchModes::default();
        assert_eq!(modes.parse_token("%tools"), Some(("%tools", None)));
        assert_eq!(modes.parse_token("&id"), Some(("&id", None)));
    }

    #[test]
    fn a_lone_prefix_searches_nothing_unless_disabled() {
        let modes = SearchModes::default();
        assert_eq!(modes.parse_token("@"), None);
        assert_eq!(modes.parse_token("%"), Some(("%", None)));
        assert_eq!(modes.parse_token(""), None);
    }

    #[test]
    fn plain_words_search_every_enabled_field() {
        assert_eq!(
            SearchModes::default().parse_token("iron"),
            Some(("iron", None))
        );
    }

    #[test]
    fn modes_round_trip_through_their_names() {
        for mode in [
            SearchMode::Enabled,
            SearchMode::RequirePrefix,
            SearchMode::Disabled,
        ] {
            assert_eq!(SearchMode::parse(mode.name()), Some(mode));
        }
        assert_eq!(SearchMode::parse("other"), None);
    }
}
