//! The player's settings: a small TOML file, `config/dctrxds_jei-client.toml`.

use crate::ingredient::SortStage;
use crate::lang::{Lang, Msg, text};
use crate::search::{SearchMode, SearchModes};

/// A language: the game's, or a fixed one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LangChoice {
    Game,
    Fixed(Lang),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub overlay_visible: bool,
    /// JEI's cheat mode: off by default.
    pub cheat_mode: bool,
    /// Whether cheat mode may send `/give` for the player when a plain gift
    /// isn't possible. Off until the player turns it on.
    pub allow_give_command: bool,
    pub edit_mode: bool,
    pub history_enabled: bool,
    pub show_tag_recipes: bool,
    pub scrolling: bool,
    pub show_stats: bool,
    pub language: LangChoice,
    pub modes: SearchModes,
    pub sort_stages: Vec<SortStage>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            overlay_visible: true,
            cheat_mode: false,
            allow_give_command: false,
            edit_mode: false,
            history_enabled: false,
            show_tag_recipes: true,
            scrolling: false,
            show_stats: false,
            language: LangChoice::Game,
            modes: SearchModes::default(),
            sort_stages: SortStage::DEFAULT.to_vec(),
        }
    }
}

/// The settings the mod's page in Options > Mods switches, in its order.
type Flag = fn(&mut Config) -> &mut bool;

const SWITCHES: [(Msg, Flag); 7] = [
    (Msg::OverlayShown, |c| &mut c.overlay_visible),
    (Msg::CheatMode, |c| &mut c.cheat_mode),
    (Msg::GiveCommand, |c| &mut c.allow_give_command),
    (Msg::EditMode, |c| &mut c.edit_mode),
    (Msg::History, |c| &mut c.history_enabled),
    (Msg::TagRecipes, |c| &mut c.show_tag_recipes),
    (Msg::Scrolling, |c| &mut c.scrolling),
];

fn mode_from(
    table: &toml::Table,
    key: &str,
    default: SearchMode,
    warnings: &mut Vec<String>,
) -> SearchMode {
    match table.get(key) {
        None => default,
        Some(value) => match value.as_str().and_then(SearchMode::parse) {
            Some(mode) => mode,
            None => {
                warnings.push(format!("{key}: use enabled, require_prefix or disabled"));
                default
            }
        },
    }
}

impl Config {
    /// Reads a file's text. What can't be used keeps its default and gets a
    /// warning; a typo must not lose the rest of the file.
    pub fn parse(contents: &str) -> (Self, Vec<String>) {
        let mut config = Self::default();
        let mut warnings = Vec::new();
        let table: toml::Table = match contents.parse() {
            Ok(table) => table,
            Err(error) => {
                warnings.push(format!("not valid TOML: {error}"));
                return (config, warnings);
            }
        };
        let flags: [(&str, Flag); 8] = [
            ("overlay_visible", |c| &mut c.overlay_visible),
            ("cheat_mode", |c| &mut c.cheat_mode),
            ("allow_give_command", |c| &mut c.allow_give_command),
            ("edit_mode", |c| &mut c.edit_mode),
            ("history_enabled", |c| &mut c.history_enabled),
            ("show_tag_recipes", |c| &mut c.show_tag_recipes),
            ("scrolling", |c| &mut c.scrolling),
            ("show_stats", |c| &mut c.show_stats),
        ];
        for (key, value) in &table {
            let flag = flags.iter().find(|(name, _)| name == key);
            match (key.as_str(), flag) {
                (_, Some((_, field))) => match value.as_bool() {
                    Some(on) => *field(&mut config) = on,
                    None => warnings.push(format!("{key}: use true or false")),
                },
                ("language", _) => match value.as_str() {
                    Some("game") => config.language = LangChoice::Game,
                    Some(code) => config.language = LangChoice::Fixed(Lang::from_code(code)),
                    None => warnings.push("language: use a text".to_owned()),
                },
                ("sort_stages", _) => {
                    let stages: Option<Vec<SortStage>> = value.as_array().and_then(|list| {
                        list.iter()
                            .map(|v| v.as_str().and_then(SortStage::parse))
                            .collect()
                    });
                    match stages {
                        Some(stages) => config.sort_stages = stages,
                        None => warnings.push(
                            "sort_stages: use a list of mod_name, alphabetical, creative_menu, tag"
                                .to_owned(),
                        ),
                    }
                }
                (
                    "search_mods"
                    | "search_tags"
                    | "search_tooltips"
                    | "search_creative_tabs"
                    | "search_colors"
                    | "search_identifiers",
                    _,
                ) => {}
                _ => warnings.push(format!("{key}: unknown setting")),
            }
        }
        let defaults = SearchModes::default();
        config.modes = SearchModes {
            mods: mode_from(&table, "search_mods", defaults.mods, &mut warnings),
            tags: mode_from(&table, "search_tags", defaults.tags, &mut warnings),
            tooltips: mode_from(&table, "search_tooltips", defaults.tooltips, &mut warnings),
            creative_tabs: mode_from(
                &table,
                "search_creative_tabs",
                defaults.creative_tabs,
                &mut warnings,
            ),
            colors: mode_from(&table, "search_colors", defaults.colors, &mut warnings),
            identifiers: mode_from(
                &table,
                "search_identifiers",
                defaults.identifiers,
                &mut warnings,
            ),
        };
        (config, warnings)
    }

    /// The file's text, with what each setting does.
    pub fn to_toml(&self) -> String {
        let language = match self.language {
            LangChoice::Game => "game",
            LangChoice::Fixed(Lang::English) => "en_us",
            LangChoice::Fixed(Lang::Portuguese) => "pt_br",
        };
        let stages: Vec<String> = self
            .sort_stages
            .iter()
            .map(|s| format!("\"{}\"", s.name()))
            .collect();
        format!(
            "# DctrXD's JEI settings. The game reads this file again when it changes.\n\
             overlay_visible = {}\n\
             # Clicks on the list give items instead of showing recipes.\n\
             cheat_mode = {}\n\
             # Lets cheat mode send /give in your name when a plain gift isn't possible.\n\
             allow_give_command = {}\n\
             # Clicks on the list hide items.\n\
             edit_mode = {}\n\
             history_enabled = {}\n\
             show_tag_recipes = {}\n\
             # Move the list by rows instead of by pages.\n\
             scrolling = {}\n\
             show_stats = {}\n\
             # game, en_us or pt_br\n\
             language = \"{language}\"\n\
             # mod_name, alphabetical, creative_menu, tag\n\
             sort_stages = [{}]\n\
             # enabled, require_prefix or disabled\n\
             search_mods = \"{}\"\n\
             search_tags = \"{}\"\n\
             search_tooltips = \"{}\"\n\
             search_creative_tabs = \"{}\"\n\
             search_colors = \"{}\"\n\
             search_identifiers = \"{}\"\n",
            self.overlay_visible,
            self.cheat_mode,
            self.allow_give_command,
            self.edit_mode,
            self.history_enabled,
            self.show_tag_recipes,
            self.scrolling,
            self.show_stats,
            stages.join(", "),
            self.modes.mods.name(),
            self.modes.tags.name(),
            self.modes.tooltips.name(),
            self.modes.creative_tabs.name(),
            self.modes.colors.name(),
            self.modes.identifiers.name(),
        )
    }

    /// The buttons of the mod's settings page.
    pub fn switch_labels(&self, lang: Lang) -> Vec<String> {
        let mut copy = self.clone();
        SWITCHES
            .iter()
            .map(|(msg, field)| {
                let state = if *field(&mut copy) { Msg::On } else { Msg::Off };
                format!("{}: {}", text(lang, *msg), text(lang, state))
            })
            .collect()
    }

    /// Flips a button's setting; false for a button that doesn't exist.
    pub fn flip_switch(&mut self, button: usize) -> bool {
        let Some((_, field)) = SWITCHES.get(button) else {
            return false;
        };
        let flag = field(self);
        *flag = !*flag;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cheating_is_off_unless_asked_for() {
        let config = Config::default();
        assert!(!config.cheat_mode && !config.allow_give_command && !config.history_enabled);
        assert!(config.overlay_visible);
    }

    #[test]
    fn a_config_survives_a_round_trip() {
        let config = Config {
            cheat_mode: true,
            scrolling: true,
            language: LangChoice::Fixed(Lang::Portuguese),
            sort_stages: vec![SortStage::Tag, SortStage::Alphabetical],
            modes: SearchModes {
                identifiers: SearchMode::Enabled,
                ..SearchModes::default()
            },
            ..Config::default()
        };
        let (read, warnings) = Config::parse(&config.to_toml());
        assert_eq!(read, config);
        assert_eq!(warnings, Vec::<String>::new());
    }

    #[test]
    fn the_default_file_reads_back_as_the_defaults() {
        let (read, warnings) = Config::parse(&Config::default().to_toml());
        assert_eq!(read, Config::default());
        assert!(warnings.is_empty());
    }

    #[test]
    fn a_typo_warns_and_keeps_the_rest() {
        let (read, warnings) = Config::parse(
            "cheat_mode = true\nscroling = true\nshow_stats = 3\nsearch_tags = \"maybe\"\nsort_stages = [\"armor\"]",
        );
        assert!(read.cheat_mode);
        assert!(!read.show_stats);
        assert_eq!(read.modes.tags, SearchMode::RequirePrefix);
        assert_eq!(read.sort_stages, SortStage::DEFAULT);
        assert_eq!(warnings.len(), 4);
        assert!(warnings.iter().any(|w| w.contains("scroling")));
    }

    #[test]
    fn a_broken_file_gives_the_defaults_and_a_warning() {
        let (read, warnings) = Config::parse("cheat_mode = = =");
        assert_eq!(read, Config::default());
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn switches_flip_their_setting_and_name_their_state() {
        let mut config = Config::default();
        assert_eq!(config.switch_labels(Lang::English)[1], "Cheat mode: OFF");
        assert!(config.flip_switch(1));
        assert!(config.cheat_mode);
        assert_eq!(
            config.switch_labels(Lang::Portuguese)[1],
            "Modo cheat: LIGADO"
        );
        assert!(!config.flip_switch(99));
    }
}
