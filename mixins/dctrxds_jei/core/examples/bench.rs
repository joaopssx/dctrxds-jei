//! Times the slow parts on a modpack-sized table with made-up names:
//! `cargo run --release --example bench -p dctrxds-jei-core`.
//! No item here comes from the game.

use dctrxds_jei_core::app::{App, DrawCmd, Input, KeyBinds, Saved, Stats};
use dctrxds_jei_core::config::Config;
use dctrxds_jei_core::grid::{ScreenInfo, ScreenKind};
use dctrxds_jei_core::ingredient::{RawItem, Stack};
use dctrxds_jei_core::lang::Lang;
use dctrxds_jei_core::recipe::{RawKind, RawRecipe, Tables};
use std::hint::black_box;
use std::time::{Duration, Instant};

const WORDS: [&str; 16] = [
    "iron", "gold", "oak", "copper", "dust", "ingot", "plate", "gear", "block", "stone", "red",
    "blue", "sword", "machine", "frame", "wire",
];

fn tables(items: usize, recipes: usize) -> Tables {
    let word = |n: usize| WORDS[n % WORDS.len()];
    let items: Vec<RawItem> = (0..items)
        .map(|n| RawItem {
            id: format!("mod{}:{}_{}_{n}", n % 200, word(n), word(n / 16)),
            name: format!("{} {} {n}", word(n / 3), word(n / 7)),
            tags: vec![
                format!("c:{}s", word(n / 11)),
                format!("mod{}:all", n % 200),
            ],
        })
        .collect();
    let stack = |n: usize| Stack::of(&items[n % items.len()].id);
    let recipes = (0..recipes)
        .map(|n| RawRecipe {
            id: format!("r:{n}"),
            kind: RawKind::CraftingShaped,
            width: 3,
            height: 3,
            slots: (0..9)
                .map(|s| vec![stack(n * 7 + s), stack(n * 13 + s)])
                .collect(),
            results: vec![stack(n)],
            cooking_time: 0,
            experience: 0.0,
            book_id: n as i32,
        })
        .collect();
    Tables {
        items,
        recipes,
        custom: Vec::new(),
    }
}

fn median(mut times: Vec<Duration>) -> Duration {
    times.sort();
    times[times.len() / 2]
}

fn screen(mouse: [f32; 2]) -> ScreenInfo {
    ScreenInfo {
        kind: ScreenKind::Inventory,
        rect: [152.0, 52.0, 176.0, 166.0],
        window: [480.0, 270.0],
        mouse,
    }
}

fn main() {
    let measure = |text: &str| text.chars().count() as f32 * 6.0;
    for (items, recipes) in [
        (2_000, 1_500),
        (20_000, 20_000),
        (50_000, 50_000),
        (100_000, 100_000),
    ] {
        let tables = tables(items, recipes);
        let started = Instant::now();
        let mut app = App::new(&tables, Config::default(), Lang::English, Saved::default());
        let build = started.elapsed();
        let binds = KeyBinds::default();
        let at = screen([200.0, 100.0]);
        let focus = Input::Key {
            key: "key.keyboard.f",
            modifiers: 2,
            binds: &binds,
        };
        app.on_input(&at, focus);
        let mut search = Vec::new();
        for query in [
            "iron",
            "o",
            "iron gear -sword",
            "@mod1 #dusts|#gears",
            "zzz",
        ] {
            let times = (0..20)
                .map(|_| {
                    for c in query.chars() {
                        app.on_input(&at, Input::Char(c));
                    }
                    let started = Instant::now();
                    // The last key's whole cost: searching, sorting and drawing.
                    black_box(app.draw(&at, Stats::default(), &measure));
                    let time = started.elapsed();
                    for _ in query.chars() {
                        let back = Input::Key {
                            key: "key.keyboard.backspace",
                            modifiers: 0,
                            binds: &binds,
                        };
                        app.on_input(&at, back);
                    }
                    time
                })
                .collect();
            search.push((query, median(times)));
        }
        app.on_input(&at, Input::Char(' '));
        let frames: Vec<Duration> = (0..200)
            .map(|_| {
                let started = Instant::now();
                let commands: Vec<DrawCmd> = app.draw(&at, Stats::default(), &measure);
                black_box(commands);
                started.elapsed()
            })
            .collect();
        let started = Instant::now();
        let focus_stack = Stack::of(&tables.items[0].id);
        let opened = app.open(&focus_stack, dctrxds_jei_core::recipe::Role::Input);
        let lookup = started.elapsed();
        println!(
            "{items} items, {recipes} recipes ({} ingredients): build {build:?}; lookup {lookup:?} (found: {opened}); frame core {:?}",
            app.ingredient_count(),
            median(frames),
        );
        for (query, time) in search {
            println!("  search {query:?}: {time:?}");
        }
    }
}
