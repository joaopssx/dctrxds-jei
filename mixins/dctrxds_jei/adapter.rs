// The only code that knows both the game and the core: it turns hook calls into
// core calls, paints the core's drawing commands and runs its actions. Injected
// as `game::dctrxds_jei`. Module payloads cannot use inner attributes or `//!`.

use crate::game::Game;
use crate::game::native_configs::{NativeConfig, NativeConfigFile, NativeConfigKind};
use crate::game::native_hooks::{NativeRecipeKind, NativeRecipeStack, NativeRecipes};
use crate::game::native_reload_listeners::NativeReloadListener;
use dctrxds_jei_core::app::{
    Action, App, DISPLAY_NAME, DrawCmd, Handled, Input, KEYS, KeyBinds, Saved, Stats,
};
use dctrxds_jei_core::categories::CategoryDecl;
use dctrxds_jei_core::config::{Config, LangChoice};
use dctrxds_jei_core::grid::{ScreenInfo, ScreenKind};
use dctrxds_jei_core::ingredient::{RawItem, Stack};
use dctrxds_jei_core::lang::{self, Lang, Msg};
use dctrxds_jei_core::persist::{self, DecodeError};
use dctrxds_jei_core::recipe::{CustomCategory, CustomRecipe, RawKind, RawRecipe, Tables};
use horizon_client::hud::HudBatch;
use horizon_client::items::ItemStack;
use horizon_client::mod_api::{NativeMod, NativeScreenView};
use horizon_client::text::Text;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub const ID: &str = "dctrxds_jei";
const VERSION: &str = "0.1.0";
const DESCRIPTION: &str =
    "Unofficial port of Just Enough Items by mezz: items, search, recipes and bookmarks.";
const ITEM_SIZE: f32 = 16.0;
const CATEGORIES_LISTENER: &str = "dctrxds_jei:categories";
const CATEGORIES_FOLDER: &str = "dctrxds_jei/categories";

// A world's state while it is open, on the window's thread. Input and draw
// hooks only get `&Game`, so what must change the game waits as an `Action` for
// the next frame.
struct Session {
    app: App,
    tables: Tables,
    source: Arc<NativeRecipes>,
    categories_revision: u64,
    known: Arc<[i32]>,
}

std::thread_local! {
    static SESSION: RefCell<Option<Session>> = const { RefCell::new(None) };
    static CONFIG: RefCell<Option<Config>> = const { RefCell::new(None) };
    static LAST_DRAW_MICROS: Cell<u64> = const { Cell::new(0) };
    // Input hooks have no batch: the window's GUI size as the last drawing saw it.
    static LAST_WINDOW: Cell<[f32; 2]> = const { Cell::new([0.0, 0.0]) };
    static CLOCK: Instant = Instant::now();
    // A mod stack's components cost a parse: each kind is read once.
    static STACKS: RefCell<HashMap<Stack, Option<ItemStack>>> = RefCell::new(HashMap::new());
}

// Config files and data folders may be read on other threads than the window.
static CONFIG_TEXT: Mutex<Option<String>> = Mutex::new(None);
static CONFIG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);
static CATEGORY_FILES: Mutex<Vec<(String, serde_json::Value)>> = Mutex::new(Vec::new());
static CATEGORIES_REVISION: AtomicU64 = AtomicU64::new(0);

fn log(message: &str) {
    eprintln!("[{ID}] {message}");
}

/// Runs `f` on the state. `None` with no world open, or when a hook re-enters
/// during another: the game can fire hooks from inside hooks, and this must not
/// panic for it.
fn with_app<R>(f: impl FnOnce(&mut App) -> R) -> Option<R> {
    SESSION.with(|session| {
        let mut guard = session.try_borrow_mut().ok()?;
        guard.as_mut().map(|s| f(&mut s.app))
    })
}

fn current_config() -> Config {
    CONFIG.with(|config| {
        config
            .try_borrow()
            .ok()
            .and_then(|c| c.clone())
            .unwrap_or_default()
    })
}

fn game_lang(game: &Game<'_>) -> Lang {
    std::fs::read_to_string(game.native_instance_dir().join("options.txt"))
        .map_or(Lang::English, |options| Lang::from_options(&options))
}

fn data_file(dir: &Path, name: &str) -> PathBuf {
    dir.join(ID).join(name)
}

/// Reads a saved file. A broken one is kept beside it as `.bak`, and the mod
/// starts without it rather than losing it or stopping the game.
fn load_list<T>(name: &str, decode: fn(&str) -> Result<Vec<T>, DecodeError>) -> Vec<T> {
    let path = match Game::native_config_dir() {
        Ok(dir) => data_file(&dir, name),
        Err(error) => {
            log(&format!("no config folder: {error}"));
            return Vec::new();
        }
    };
    let contents = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(error) => {
            log(&format!("cannot read {}: {error}", path.display()));
            return Vec::new();
        }
    };
    decode(&contents).unwrap_or_else(|error| {
        log(&format!("cannot use {}: {error}", path.display()));
        if let Err(error) = std::fs::rename(&path, path.with_extension("json.bak")) {
            log(&format!("cannot keep a backup: {error}"));
        }
        Vec::new()
    })
}

/// Writes through a temporary file, so a crash cannot leave half a file.
fn write_atomic(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let temporary = path.with_extension("tmp");
    std::fs::write(&temporary, contents).map_err(|e| format!("{}: {e}", temporary.display()))?;
    std::fs::rename(&temporary, path).map_err(|e| format!("{}: {e}", path.display()))
}

fn save_data(name: &str, contents: &str) {
    let result =
        Game::native_config_dir().and_then(|dir| write_atomic(&data_file(&dir, name), contents));
    if let Err(error) = result {
        log(&format!("cannot save {name}: {error}"));
    }
}

fn save_config(text: &str) {
    let known = CONFIG_PATH.lock().ok().and_then(|path| path.clone());
    let path = known.map_or_else(
        || Game::native_config_dir().map(|dir| dir.join(format!("{ID}-client.toml"))),
        Ok,
    );
    if let Err(error) = path.and_then(|path| write_atomic(&path, text)) {
        log(&format!("cannot save the settings: {error}"));
    }
}

fn screen_info(window: [f32; 2], screen: &NativeScreenView<'_>) -> Option<ScreenInfo> {
    let kind = match screen.id {
        "inventory" => ScreenKind::Inventory,
        "creative" => ScreenKind::Creative,
        _ => return None,
    };
    Some(ScreenInfo {
        kind,
        rect: screen.rect,
        window,
        mouse: screen.mouse,
    })
}

// ------------------------------------------------------------- the game's tables

fn stack_of(tables: &NativeRecipes, stack: &NativeRecipeStack) -> Option<Stack> {
    let item = tables.items.get(stack.item as usize)?;
    Some(Stack {
        item: item.id.clone(),
        count: stack.count,
        components: stack.components.clone(),
    })
}

fn convert_kind(kind: NativeRecipeKind) -> RawKind {
    match kind {
        NativeRecipeKind::CraftingShaped => RawKind::CraftingShaped,
        NativeRecipeKind::CraftingShapeless => RawKind::CraftingShapeless,
        NativeRecipeKind::Smelting => RawKind::Smelting,
        NativeRecipeKind::Blasting => RawKind::Blasting,
        NativeRecipeKind::Smoking => RawKind::Smoking,
        NativeRecipeKind::Campfire => RawKind::Campfire,
        NativeRecipeKind::Stonecutting => RawKind::Stonecutting,
        NativeRecipeKind::Smithing => RawKind::Smithing,
        NativeRecipeKind::Brewing => RawKind::Brewing,
    }
}

fn custom_categories(game: &Game<'_>) -> Vec<CustomCategory> {
    let files = CATEGORY_FILES.lock().map(|f| f.clone()).unwrap_or_default();
    let mut categories = Vec::new();
    for (id, value) in &files {
        match CategoryDecl::parse(id, value) {
            Ok(decl) => categories.push(custom_category(game, &decl)),
            Err(error) => log(&format!("category {id}: {error}")),
        }
    }
    categories
}

fn custom_category(game: &Game<'_>, decl: &CategoryDecl) -> CustomCategory {
    let recipes = game
        .native_recipes_of_type(&decl.recipe_type)
        .iter()
        .map(|recipe| {
            let inputs = decl
                .inputs
                .iter()
                .filter_map(|field| recipe.json.get(field))
                .filter_map(|value| game.native_recipe_ingredient(value))
                .map(|ingredient| ingredient.order.iter().map(|i| Stack::of(i)).collect())
                .collect();
            let outputs = decl
                .outputs
                .iter()
                .filter_map(|field| recipe.json.get(field))
                .filter_map(|value| game.native_recipe_result(value))
                .map(|stack| Stack {
                    item: stack.item.to_string(),
                    count: stack.count,
                    components: components_of(&stack),
                })
                .collect();
            CustomRecipe {
                id: recipe.id.clone(),
                inputs,
                outputs,
            }
        })
        .collect();
    CustomCategory {
        id: decl.id.clone(),
        title: decl.title.clone(),
        icon: Stack::of(&decl.icon),
        catalysts: decl.catalysts.iter().map(|i| Stack::of(i)).collect(),
        recipes,
    }
}

fn build_tables(game: &Game<'_>, source: &NativeRecipes) -> Tables {
    let items = source
        .items
        .iter()
        .map(|item| RawItem {
            id: item.id.clone(),
            name: item.name.clone(),
            tags: item.tags.clone(),
        })
        .collect();
    let recipes = source
        .recipes
        .iter()
        .map(|recipe| RawRecipe {
            id: recipe.id.clone(),
            kind: convert_kind(recipe.kind),
            width: recipe.width,
            height: recipe.height,
            slots: recipe
                .slots
                .iter()
                .map(|slot| slot.iter().filter_map(|s| stack_of(source, s)).collect())
                .collect(),
            results: recipe
                .results
                .iter()
                .filter_map(|s| stack_of(source, s))
                .collect(),
            cooking_time: recipe.cooking_time,
            experience: recipe.experience,
            book_id: recipe.book_id,
        })
        .collect();
    Tables {
        items,
        recipes,
        custom: custom_categories(game),
    }
}

/// A stack's components beyond its item's, in the game's item syntax.
fn components_of(stack: &ItemStack) -> String {
    use horizon_world::nbt::Tag;
    let Some(Tag::Compound(entries)) = horizon_client::item_data::encode_components(stack) else {
        return String::new();
    };
    let parts: Vec<String> = entries
        .iter()
        .map(|(key, value)| {
            format!(
                "{}={}",
                key.to_string_lossy(),
                horizon_client::snbt::to_snbt(value)
            )
        })
        .collect();
    format!("[{}]", parts.join(","))
}

/// The game's stack for one with components; read once per kind.
fn item_stack(stack: &Stack) -> Option<ItemStack> {
    STACKS.with(|stacks| {
        let mut stacks = stacks.try_borrow_mut().ok()?;
        if let Some(known) = stacks.get(stack) {
            return known.clone();
        }
        let made = horizon_client::item_data::command_components(&stack.components)
            .ok()
            .map(|tag| {
                let (damage, extra) = horizon_client::item_data::decode_components(Some(&tag));
                ItemStack {
                    item: stack.item.as_str().into(),
                    count: 1,
                    damage,
                    extra,
                }
            });
        stacks.insert(stack.clone(), made.clone());
        made
    })
}

// --------------------------------------------------------------------- the hooks

pub(super) fn listed(mods: &mut Vec<NativeMod>) {
    mods.push(NativeMod {
        id: ID.into(),
        title: format!("{DISPLAY_NAME} {VERSION}"),
        description: DESCRIPTION.into(),
        keys: KEYS.iter().map(|key| key.map(String::from)).collect(),
    });
}

pub(super) fn settings_page(game: &Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    if mod_id == ID {
        let config = current_config();
        let language = match config.language {
            LangChoice::Game => game_lang(game),
            LangChoice::Fixed(language) => language,
        };
        buttons.extend(config.switch_labels(language));
    }
}

pub(super) fn settings_click(mod_id: &str, button: usize) {
    if mod_id != ID {
        return;
    }
    let mut config = current_config();
    if config.flip_switch(button) {
        let text = config.to_toml();
        save_config(&text);
        if let Ok(mut pending) = CONFIG_TEXT.lock() {
            *pending = Some(text);
        }
    }
}

pub(super) fn register_configs(configs: &mut Vec<NativeConfig>) {
    configs.push(NativeConfig::new(
        ID,
        NativeConfigKind::Client,
        &Config::default().to_toml(),
    ));
}

pub(super) fn config_read(config: &NativeConfigFile) {
    if config.mod_id != ID {
        return;
    }
    if let Ok(mut path) = CONFIG_PATH.lock() {
        *path = Some(config.path.clone());
    }
    if let Ok(mut pending) = CONFIG_TEXT.lock() {
        *pending = Some(config.text.clone());
    }
}

pub(super) fn add_listeners(listeners: &mut Vec<NativeReloadListener>) {
    listeners.push(NativeReloadListener::json(
        CATEGORIES_LISTENER,
        CATEGORIES_FOLDER,
    ));
}

pub(super) fn listener_files(
    listener: &NativeReloadListener,
    files: &[(String, serde_json::Value)],
) {
    if listener.id != CATEGORIES_LISTENER {
        return;
    }
    if let Ok(mut kept) = CATEGORY_FILES.lock() {
        *kept = files.to_vec();
        CATEGORIES_REVISION.fetch_add(1, Ordering::Relaxed);
    }
}

fn apply_pending_config(game: &Game<'_>) {
    let pending = CONFIG_TEXT.lock().ok().and_then(|mut text| text.take());
    let Some(text) = pending else {
        return;
    };
    let (config, warnings) = Config::parse(&text);
    for warning in warnings {
        log(&format!("settings: {warning}"));
    }
    CONFIG.with(|slot| {
        if let Ok(mut slot) = slot.try_borrow_mut() {
            *slot = Some(config.clone());
        }
    });
    SESSION.with(|session| {
        if let Ok(mut guard) = session.try_borrow_mut()
            && let Some(session) = guard.as_mut()
            && session.app.config() != &config
        {
            session
                .app
                .set_config(config, &session.tables, game_lang(game));
        }
    });
}

fn start_session(game: &Game<'_>, source: Arc<NativeRecipes>, revision: u64) -> Session {
    let started = Instant::now();
    let tables = build_tables(game, &source);
    let saved = Saved {
        bookmarks: load_list("bookmarks.json", persist::decode_bookmarks),
        hidden: load_list("hidden.json", persist::decode_stacks),
        history: load_list("history.json", persist::decode_stacks),
    };
    let app = App::new(&tables, current_config(), game_lang(game), saved);
    log(&format!(
        "loaded {} ingredients and {} recipes in {} ms",
        app.ingredient_count(),
        tables.recipes.len(),
        started.elapsed().as_millis()
    ));
    Session {
        app,
        tables,
        source,
        categories_revision: revision,
        known: Arc::from([]),
    }
}

/// Follows the world and the mod's switch: the state is built when a world
/// opens and when the game makes its tables again.
fn follow_world(game: &mut Game<'_>) {
    let source = if game.native_on(ID) {
        game.native_recipes()
    } else {
        None
    };
    let revision = CATEGORIES_REVISION.load(Ordering::Relaxed);
    let game: &Game<'_> = game;
    SESSION.with(|session| {
        let Ok(mut guard) = session.try_borrow_mut() else {
            return;
        };
        match (source, guard.as_mut()) {
            (None, _) => *guard = None,
            (Some(source), None) => *guard = Some(start_session(game, source, revision)),
            (Some(source), Some(session)) => {
                if !Arc::ptr_eq(&source, &session.source) || revision != session.categories_revision
                {
                    session.tables = build_tables(game, &source);
                    session.source = source;
                    session.categories_revision = revision;
                    session.app.set_tables(&session.tables, game_lang(game));
                }
            }
        }
    });
}

fn give(game: &mut Game<'_>, stack: &Stack, count: u32, allow_command: bool, language: Lang) {
    let refusal = if stack.components.is_empty() {
        match game.native_give(&stack.item, count) {
            Ok(()) => return,
            Err(error) => error,
        }
    } else {
        lang::text(language, Msg::GiveNeedsCommand).to_owned()
    };
    if allow_command {
        let command = format!("/give @s {}{} {count}", stack.item, stack.components);
        if let Err(error) = game.native_send_chat(&command) {
            log(&format!("cannot send {command}: {error}"));
            game.native_chat(&format!("{DISPLAY_NAME}: {error}"));
        }
    } else {
        log(&format!("cannot give {}: {refusal}", stack.item));
        game.native_chat(&format!("{DISPLAY_NAME}: {refusal}"));
    }
}

fn run_action(game: &mut Game<'_>, action: Action, allow_command: bool, language: Lang) {
    match action {
        Action::Give { stack, count } => give(game, &stack, count, allow_command, language),
        Action::PlaceRecipe { book_id, all } => {
            if let Err(error) = game.native_place_recipe(book_id, all) {
                log(&format!("cannot place recipe {book_id}: {error}"));
                game.native_chat(&format!("{DISPLAY_NAME}: {error}"));
            }
        }
        Action::SaveBookmarks { contents } => save_data("bookmarks.json", &contents),
        Action::SaveHidden { contents } => save_data("hidden.json", &contents),
        Action::SaveHistory { contents } => save_data("history.json", &contents),
        Action::SaveConfig { contents } => {
            save_config(&contents);
            CONFIG.with(|slot| {
                if let Ok(mut slot) = slot.try_borrow_mut() {
                    *slot = Some(Config::parse(&contents).0);
                }
            });
        }
    }
}

/// Each frame, where the game can be changed.
pub(super) fn frame(game: &mut Game<'_>) {
    apply_pending_config(game);
    follow_world(game);
    let known = game.native_known_recipes();
    let actions = SESSION.with(|session| {
        let mut guard = session.try_borrow_mut().ok()?;
        let session = guard.as_mut()?;
        if !Arc::ptr_eq(&known, &session.known) {
            session.app.set_known(&known);
            session.known = known;
        }
        Some((
            session.app.take_actions(),
            session.app.config().allow_give_command,
            session.app.lang(),
        ))
    });
    if let Some((actions, allow_command, language)) = actions {
        for action in actions {
            run_action(game, action, allow_command, language);
        }
    }
}

pub(super) fn draw(game: &Game<'_>, batch: &mut HudBatch, screen: &NativeScreenView<'_>) {
    let size = [batch.width, batch.height];
    LAST_WINDOW.with(|window| window.set(size));
    let Some(info) = screen_info(size, screen) else {
        return;
    };
    let started = Instant::now();
    let stats = Stats {
        frames_per_second: game.native_frame_rate().0,
        draw_micros: LAST_DRAW_MICROS.with(Cell::get),
        time_millis: CLOCK.with(|clock| u64::try_from(clock.elapsed().as_millis()).unwrap_or(0)),
    };
    let measure = |text: &str| game.native_text_width(text);
    let Some(commands) = with_app(|app| app.draw(&info, stats, &measure)) else {
        return;
    };
    for command in commands {
        match command {
            DrawCmd::Fill { rect, argb } => game.native_fill(batch, rect, argb),
            DrawCmd::Item { stack, at } => {
                if stack.components.is_empty() {
                    game.native_item(batch, &stack.item, at, ITEM_SIZE);
                } else if let Some(made) = item_stack(&stack) {
                    game.native_item_stack(batch, &made, at, ITEM_SIZE);
                } else {
                    game.native_item(batch, &stack.item, at, ITEM_SIZE);
                }
            }
            DrawCmd::Text { text, at, rgb } => game.native_text(batch, &text, at, rgb, true),
            DrawCmd::Tooltip { lines, at } => {
                let lines: Vec<Text> = lines.iter().map(|line| Text::literal(line)).collect();
                game.native_gui_tooltip(batch, &lines, at);
            }
        }
    }
    let micros = u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX);
    LAST_DRAW_MICROS.with(|last| last.set(micros));
}

fn input(screen: &NativeScreenView<'_>, canceled: &mut bool, input: Input<'_>) {
    let Some(info) = screen_info(LAST_WINDOW.with(Cell::get), screen) else {
        return;
    };
    if with_app(|app| app.on_input(&info, input)) == Some(Handled::Consumed) {
        *canceled = true;
    }
}

pub(super) fn press(
    screen: &NativeScreenView<'_>,
    button: i32,
    modifiers: u32,
    canceled: &mut bool,
) {
    input(screen, canceled, Input::MousePress { button, modifiers });
}

pub(super) fn release(screen: &NativeScreenView<'_>, button: i32, canceled: &mut bool) {
    input(screen, canceled, Input::MouseRelease { button });
}

pub(super) fn scroll(screen: &NativeScreenView<'_>, amount: [f64; 2], canceled: &mut bool) {
    input(screen, canceled, Input::Scroll { y: amount[1] });
}

pub(super) fn key(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    key: &str,
    modifiers: u32,
    canceled: &mut bool,
) {
    let binds = KeyBinds {
        keys: KEYS.map(|[name, _, default]| game.native_mod_key(ID, name, default)),
    };
    input(
        screen,
        canceled,
        Input::Key {
            key,
            modifiers,
            binds: &binds,
        },
    );
}

pub(super) fn typed(screen: &NativeScreenView<'_>, c: char, canceled: &mut bool) {
    input(screen, canceled, Input::Char(c));
}

pub(super) fn screen_closed(screen: &NativeScreenView<'_>) {
    if matches!(screen.id, "inventory" | "creative") {
        with_app(App::on_screen_closed);
    }
}

pub(super) fn tooltip_shown(stack: &ItemStack) {
    let stack = Stack {
        item: stack.item.to_string(),
        count: 1,
        components: components_of(stack),
    };
    with_app(|app| app.note_tooltip_stack(stack));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_files_live_in_the_mods_own_folder() {
        let path = data_file(Path::new("config"), "bookmarks.json");
        assert_eq!(
            path,
            Path::new("config")
                .join("dctrxds_jei")
                .join("bookmarks.json")
        );
    }

    #[test]
    fn only_inventory_and_creative_screens_get_the_overlay() {
        let view = |id: &'static str| NativeScreenView {
            id,
            menu: None,
            title: None,
            rect: [0.0; 4],
            slots: 0,
            mouse: [0.0; 2],
            partial_tick: 0.0,
        };
        assert!(screen_info([1.0, 1.0], &view("inventory")).is_some());
        assert!(screen_info([1.0, 1.0], &view("creative")).is_some());
        assert!(screen_info([1.0, 1.0], &view("options")).is_none());
    }

    #[test]
    fn a_stacks_components_are_written_as_the_game_reads_them() {
        let plain = ItemStack::new("minecraft:stick", 1);
        assert_eq!(components_of(&plain), "");
    }
}
