//! The interface's words in each language. Item names come from the game.

use crate::recipe::Title;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lang {
    #[default]
    English,
    Portuguese,
}

impl Lang {
    /// From a game language code such as `pt_br`; others fall back to English.
    pub fn from_code(code: &str) -> Self {
        if code.trim().to_lowercase().starts_with("pt") {
            Self::Portuguese
        } else {
            Self::English
        }
    }

    /// The game's language, from the text of its `options.txt`.
    pub fn from_options(options: &str) -> Self {
        options
            .lines()
            .find_map(|line| line.strip_prefix("lang:"))
            .map_or(Self::English, Self::from_code)
    }
}

/// Everything the interface says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Msg {
    SearchHint,
    Recipes,
    Uses,
    Bookmarks,
    Shapeless,
    Seconds,
    Experience,
    CheatOn,
    EditOn,
    NoRecipes,
    NotInBook,
    PlaceRecipe,
    AddBookmark,
    Hidden,
    GiveNeedsCommand,
    Tags,
    Crafting,
    Stonecutting,
    Smelting,
    Smoking,
    Blasting,
    Campfire,
    Smithing,
    Brewing,
    OverlayShown,
    CheatMode,
    GiveCommand,
    EditMode,
    History,
    TagRecipes,
    Scrolling,
    Stats,
    On,
    Off,
}

pub fn text(lang: Lang, msg: Msg) -> &'static str {
    let (english, portuguese) = match msg {
        Msg::SearchHint => ("Search...", "Buscar..."),
        Msg::Recipes => ("Recipes", "Receitas"),
        Msg::Uses => ("Uses", "Usos"),
        Msg::Bookmarks => ("Bookmarks", "Favoritos"),
        Msg::Shapeless => ("Shapeless", "Sem forma"),
        Msg::Seconds => ("s", "s"),
        Msg::Experience => ("XP", "XP"),
        Msg::CheatOn => (
            "Cheat mode: click gives items",
            "Modo cheat: clique dá itens",
        ),
        Msg::EditOn => (
            "Edit mode: click hides items",
            "Modo edição: clique esconde itens",
        ),
        Msg::NoRecipes => ("No recipes", "Sem receitas"),
        Msg::NotInBook => ("Not in your recipe book", "Fora do seu livro de receitas"),
        Msg::PlaceRecipe => ("Fill the crafting grid", "Preencher a grade"),
        Msg::AddBookmark => ("Bookmark this recipe", "Favoritar esta receita"),
        Msg::Hidden => ("Hidden", "Escondido"),
        Msg::GiveNeedsCommand => (
            "This needs /give: turn on \"Cheat may use /give\" in the settings",
            "Isto precisa de /give: ligue \"Cheat pode usar /give\" nas configurações",
        ),
        Msg::Tags => ("Tags", "Tags"),
        Msg::Crafting => ("Crafting", "Fabricação"),
        Msg::Stonecutting => ("Stonecutting", "Cortador de pedra"),
        Msg::Smelting => ("Smelting", "Fornalha"),
        Msg::Smoking => ("Smoking", "Defumador"),
        Msg::Blasting => ("Blasting", "Alto-forno"),
        Msg::Campfire => ("Campfire", "Fogueira"),
        Msg::Smithing => ("Smithing", "Ferraria"),
        Msg::Brewing => ("Brewing", "Poções"),
        Msg::OverlayShown => ("Item list", "Lista de itens"),
        Msg::CheatMode => ("Cheat mode", "Modo cheat"),
        Msg::GiveCommand => ("Cheat may use /give", "Cheat pode usar /give"),
        Msg::EditMode => ("Edit mode", "Modo edição"),
        Msg::History => ("Lookup history", "Histórico de buscas"),
        Msg::TagRecipes => ("Tag pages", "Páginas de tags"),
        Msg::Scrolling => ("Scroll by row", "Rolar por linha"),
        Msg::Stats => ("Show FPS and draw time", "Mostrar FPS e tempo de desenho"),
        Msg::On => ("ON", "LIGADO"),
        Msg::Off => ("OFF", "DESLIGADO"),
    };
    match lang {
        Lang::English => english,
        Lang::Portuguese => portuguese,
    }
}

/// A category's title in the interface's language.
pub fn title(lang: Lang, title: &Title) -> &str {
    let msg = match title {
        Title::Crafting => Msg::Crafting,
        Title::Stonecutting => Msg::Stonecutting,
        Title::Smelting => Msg::Smelting,
        Title::Smoking => Msg::Smoking,
        Title::Blasting => Msg::Blasting,
        Title::Campfire => Msg::Campfire,
        Title::Smithing => Msg::Smithing,
        Title::Brewing => Msg::Brewing,
        Title::Tags => Msg::Tags,
        Title::Literal(text) => return text,
    };
    text(lang, msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_language_comes_from_the_games_options() {
        assert_eq!(
            Lang::from_options("version:1\nlang:pt_br\nfov:0"),
            Lang::Portuguese
        );
        assert_eq!(Lang::from_options("lang:en_us"), Lang::English);
        assert_eq!(Lang::from_options("lang:de_de"), Lang::English);
        assert_eq!(Lang::from_options(""), Lang::English);
    }

    #[test]
    fn words_are_translated_and_other_mods_titles_stay_as_written() {
        assert_eq!(text(Lang::Portuguese, Msg::Recipes), "Receitas");
        assert_eq!(text(Lang::English, Msg::Recipes), "Recipes");
        assert_eq!(title(Lang::Portuguese, &Title::Smelting), "Fornalha");
        assert_eq!(title(Lang::Portuguese, &Title::Literal("X".into())), "X");
    }
}
