// The only code that knows both the game and the core: it turns hook calls into
// core calls, paints the core's drawing commands and runs its actions. Injected
// as `game::dctrxds_jei`. Module payloads cannot use inner attributes or `//!`.

use crate::game::Game;
use dctrxds_jei_core::app::{Action, App, DrawCmd, Handled, Input, Item, Stats};
use dctrxds_jei_core::grid::ScreenInfo;
use dctrxds_jei_core::persist;
use horizon_client::hud::HudBatch;
use horizon_client::mod_api::{NativeMod, NativeScreenView};
use horizon_client::text::Text;
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};

pub const ID: &str = "dctrxds_jei";
const VERSION: &str = "0.0.1";
const DESCRIPTION: &str = "Unofficial port of Just Enough Items by mezz. Proof of concept.";
/// The overlay's toggle: Control plus this key.
const TOGGLE_KEY: [&str; 3] = ["toggle", "Toggle overlay (with Ctrl)", "key.keyboard.o"];
const ITEM_SIZE: f32 = 16.0;

std::thread_local! {
    // The state while a world is open. Input and draw hooks only get `&Game`,
    // so what must change the game waits as an `Action` for the next frame.
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
    static LAST_DRAW_MICROS: Cell<u64> = const { Cell::new(0) };
    // Input hooks have no batch: the window's GUI size as the last drawing saw it.
    static LAST_WINDOW: Cell<[f32; 2]> = const { Cell::new([0.0, 0.0]) };
}

/// Runs `f` on the state. `None` with no world open, or when a hook re-enters
/// during another: the game can fire hooks from inside hooks, and this must not
/// panic for it.
fn with_app<R>(f: impl FnOnce(&mut App) -> R) -> Option<R> {
    APP.with(|app| {
        let mut guard = app.try_borrow_mut().ok()?;
        guard.as_mut().map(f)
    })
}

fn log(message: &str) {
    eprintln!("[{ID}] {message}");
}

fn config_file(dir: &Path) -> PathBuf {
    dir.join(ID).join("probe.json")
}

/// Reads the saved text. A broken file is kept beside it as `.bak`, and the
/// overlay starts empty rather than losing it or stopping the game.
fn load_text() -> String {
    let path = match Game::native_config_dir() {
        Ok(dir) => config_file(&dir),
        Err(error) => {
            log(&format!("no config folder: {error}"));
            return String::new();
        }
    };
    let contents = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return String::new(),
        Err(error) => {
            log(&format!("cannot read {}: {error}", path.display()));
            return String::new();
        }
    };
    persist::decode(&contents).unwrap_or_else(|error| {
        log(&format!("cannot use {}: {error}", path.display()));
        if let Err(error) = std::fs::rename(&path, path.with_extension("json.bak")) {
            log(&format!("cannot keep a backup: {error}"));
        }
        String::new()
    })
}

/// Writes through a temporary file, so a crash cannot leave half a file.
fn save_text(contents: &str) -> Result<(), String> {
    let path = config_file(&Game::native_config_dir()?);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, contents).map_err(|e| format!("{}: {e}", temporary.display()))?;
    std::fs::rename(&temporary, &path).map_err(|e| format!("{}: {e}", path.display()))
}

fn screen_info(batch_size: [f32; 2], screen: &NativeScreenView<'_>) -> Option<ScreenInfo> {
    matches!(screen.id, "inventory" | "creative").then_some(ScreenInfo {
        rect: screen.rect,
        window: batch_size,
        mouse: screen.mouse,
    })
}

pub(super) fn listed(mods: &mut Vec<NativeMod>) {
    mods.push(NativeMod {
        id: ID.into(),
        title: format!("{} {VERSION}", dctrxds_jei_core::app::DISPLAY_NAME),
        description: DESCRIPTION.into(),
        keys: vec![TOGGLE_KEY.map(String::from)],
    });
}

/// Each frame: follow the world and the mod's switch, then run what waits.
pub(super) fn frame(game: &mut Game<'_>) {
    let tables = if game.native_on(ID) {
        game.native_recipes()
    } else {
        None
    };
    let loaded = APP.with(|app| app.try_borrow().map(|a| a.is_some()).unwrap_or(true));
    match (tables, loaded) {
        (Some(tables), false) => {
            let started = std::time::Instant::now();
            let items = tables
                .items
                .iter()
                .map(|item| Item {
                    id: item.id.clone(),
                    name: item.name.clone(),
                })
                .collect();
            let app = App::new(items, load_text());
            log(&format!(
                "loaded {} items in {} us",
                app.item_count(),
                started.elapsed().as_micros()
            ));
            APP.with(|slot| {
                if let Ok(mut slot) = slot.try_borrow_mut() {
                    *slot = Some(app);
                }
            });
        }
        (None, true) => APP.with(|slot| {
            if let Ok(mut slot) = slot.try_borrow_mut() {
                *slot = None;
            }
        }),
        _ => {}
    }
    for action in with_app(App::take_actions).unwrap_or_default() {
        match action {
            Action::Give { item, count } => {
                // Refused outside creative mode: say so, and go on.
                if let Err(error) = game.native_give(&item, count) {
                    log(&format!("cannot give {item}: {error}"));
                    game.native_chat(&format!("DctrXD\u{b4}s JEI: {error}"));
                }
            }
            Action::SaveText { contents } => {
                if let Err(error) = save_text(&contents) {
                    log(&format!("cannot save: {error}"));
                }
            }
        }
    }
}

pub(super) fn draw(game: &Game<'_>, batch: &mut HudBatch, screen: &NativeScreenView<'_>) {
    let size = [batch.width, batch.height];
    LAST_WINDOW.with(|window| window.set(size));
    let Some(info) = screen_info(size, screen) else {
        return;
    };
    let started = std::time::Instant::now();
    let stats = Stats {
        frames_per_second: game.native_frame_rate().0,
        draw_micros: LAST_DRAW_MICROS.with(Cell::get),
    };
    let Some(commands) = with_app(|app| app.draw(&info, stats)) else {
        return;
    };
    for command in commands {
        match command {
            DrawCmd::Fill { rect, argb } => game.native_fill(batch, rect, argb),
            DrawCmd::Item { id, at } => {
                game.native_item(batch, &id, at, ITEM_SIZE);
            }
            DrawCmd::Text { text, at, rgb } => game.native_text(batch, &text, at, rgb, true),
            DrawCmd::Tooltip { lines, at } => {
                let lines: Vec<Text> = lines.iter().map(|l| Text::literal(l)).collect();
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

pub(super) fn press(screen: &NativeScreenView<'_>, button: i32, canceled: &mut bool) {
    input(screen, canceled, Input::MousePress { button });
}

pub(super) fn release(screen: &NativeScreenView<'_>, button: i32, canceled: &mut bool) {
    input(screen, canceled, Input::MouseRelease { button });
}

pub(super) fn key(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    key: &str,
    modifiers: u32,
    canceled: &mut bool,
) {
    let toggle_key = game.native_mod_key(ID, TOGGLE_KEY[0], TOGGLE_KEY[2]);
    input(
        screen,
        canceled,
        Input::Key {
            key,
            toggle_key: &toggle_key,
            modifiers,
        },
    );
}

pub(super) fn typed(screen: &NativeScreenView<'_>, c: char, canceled: &mut bool) {
    input(screen, canceled, Input::Char(c));
}

pub(super) fn tooltip_shown(item: &str) {
    with_app(|app| app.note_tooltip_stack(item));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_saved_file_lives_in_the_mods_own_folder() {
        let path = config_file(Path::new("config"));
        assert_eq!(
            path,
            Path::new("config").join("dctrxds_jei").join("probe.json")
        );
    }
}
