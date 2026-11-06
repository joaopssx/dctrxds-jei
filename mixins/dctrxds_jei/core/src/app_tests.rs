use super::*;
use crate::ingredient::RawItem;
use crate::recipe::{RawKind, RawRecipe};

fn item(id: &str, name: &str, tags: &[&str]) -> RawItem {
    RawItem {
        id: id.to_owned(),
        name: name.to_owned(),
        tags: tags.iter().map(|t| (*t).to_owned()).collect(),
    }
}

fn tables() -> Tables {
    let mut items = vec![
        item("minecraft:air", "Air", &[]),
        item("minecraft:oak_log", "Oak Log", &["minecraft:logs"]),
        item("minecraft:oak_planks", "Oak Planks", &["minecraft:planks"]),
        item("minecraft:stick", "Stick", &[]),
        item("minecraft:potion", "Potion", &[]),
    ];
    items.extend((0..200).map(|n| item(&format!("test:item_{n:03}"), &format!("Item {n}"), &[])));
    let planks = RawRecipe {
        id: "t:planks".into(),
        kind: RawKind::CraftingShapeless,
        width: 0,
        height: 0,
        slots: vec![vec![Stack::of("minecraft:oak_log")]],
        results: vec![Stack {
            count: 4,
            ..Stack::of("minecraft:oak_planks")
        }],
        cooking_time: 0,
        experience: 0.0,
        book_id: 7,
    };
    let stick = RawRecipe {
        id: "t:stick".into(),
        kind: RawKind::CraftingShaped,
        width: 1,
        height: 2,
        slots: vec![vec![Stack::of("minecraft:oak_planks")]; 2],
        results: vec![Stack::of("minecraft:stick")],
        book_id: 8,
        ..planks.clone()
    };
    let brew = RawRecipe {
        id: "t:brew".into(),
        kind: RawKind::Brewing,
        slots: vec![
            vec![Stack::of("minecraft:potion")],
            vec![Stack::of("minecraft:stick")],
        ],
        results: vec![Stack::with_components(
            "minecraft:potion",
            "[minecraft:potion_contents={potion:\"minecraft:swiftness\"}]",
        )],
        book_id: -1,
        ..planks.clone()
    };
    Tables {
        items,
        recipes: vec![planks, stick, brew],
        custom: Vec::new(),
    }
}

fn app() -> App {
    app_with(Config::default())
}

fn app_with(config: Config) -> App {
    App::new(&tables(), config, Lang::English, Saved::default())
}

fn screen(mouse: [f32; 2]) -> ScreenInfo {
    ScreenInfo {
        kind: ScreenKind::Inventory,
        rect: [152.0, 52.0, 176.0, 166.0],
        window: [480.0, 270.0],
        mouse,
    }
}

fn list_slot(n: usize) -> [f32; 2] {
    let grid = Grid::list(&screen([0.0, 0.0])).expect("room");
    let [x, y] = grid.slot_position(n);
    [x + 2.0, y + 2.0]
}

fn search_box_point() -> [f32; 2] {
    let grid = Grid::list(&screen([0.0, 0.0])).expect("room");
    let [x, y, _, _] = App::search_box(&grid);
    [x + 2.0, y + 2.0]
}

const OUTSIDE: [f32; 2] = [200.0, 100.0];

fn press(app: &mut App, at: [f32; 2], button: i32) -> Handled {
    app.on_input(
        &screen(at),
        Input::MousePress {
            button,
            modifiers: 0,
        },
    )
}

fn key_at(app: &mut App, at: [f32; 2], key: &str, modifiers: u32) -> Handled {
    let binds = KeyBinds::default();
    app.on_input(
        &screen(at),
        Input::Key {
            key,
            modifiers,
            binds: &binds,
        },
    )
}

fn measure(text: &str) -> f32 {
    text.chars().count() as f32 * 6.0
}

fn type_text(app: &mut App, text: &str) {
    for c in text.chars() {
        assert_eq!(
            app.on_input(&screen(OUTSIDE), Input::Char(c)),
            Handled::Consumed
        );
    }
}

fn listed(app: &mut App) -> Vec<String> {
    app.refresh_listing();
    app.listing
        .iter()
        .map(|&n| app.ingredients[n as usize].stack.item.clone())
        .collect()
}

fn texts(cmds: &[DrawCmd]) -> Vec<&str> {
    cmds.iter()
        .filter_map(|c| match c {
            DrawCmd::Text { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

#[test]
fn air_is_left_out_and_variants_join_the_list_beside_their_item() {
    let mut app = app();
    let items = listed(&mut app);
    assert!(!items.iter().any(|i| i == "minecraft:air"));
    assert_eq!(items[3], "minecraft:potion");
    assert_eq!(items[4], "minecraft:potion");
    let variant = app
        .ingredients
        .iter()
        .find(|i| !i.stack.components.is_empty())
        .unwrap();
    assert_eq!(variant.name, "Potion (swiftness)");
}

#[test]
fn clicks_on_the_search_box_focus_it_and_a_right_click_clears_it() {
    let mut app = app();
    assert_eq!(press(&mut app, search_box_point(), 0), Handled::Consumed);
    assert!(app.search.focused);
    type_text(&mut app, "oak");
    assert_eq!(app.search_text(), "oak");
    assert_eq!(listed(&mut app).len(), 2);
    assert_eq!(press(&mut app, search_box_point(), 1), Handled::Consumed);
    assert_eq!(app.search_text(), "");
}

#[test]
fn typing_in_the_box_does_not_reach_the_game() {
    let mut app = app();
    press(&mut app, search_box_point(), 0);
    // E would close the inventory.
    assert_eq!(
        key_at(&mut app, OUTSIDE, "key.keyboard.e", 0),
        Handled::Consumed
    );
    assert_eq!(
        app.on_input(&screen(OUTSIDE), Input::Char('e')),
        Handled::Consumed
    );
    assert_eq!(app.search_text(), "e");
    // Function keys stay the window's.
    assert_eq!(
        key_at(&mut app, OUTSIDE, "key.keyboard.f3", 0),
        Handled::Pass
    );
}

#[test]
fn escape_only_leaves_the_box_and_a_click_elsewhere_too() {
    let mut app = app();
    press(&mut app, search_box_point(), 0);
    assert_eq!(
        key_at(&mut app, OUTSIDE, "key.keyboard.escape", 0),
        Handled::Consumed
    );
    assert!(!app.search.focused);
    assert_eq!(
        key_at(&mut app, OUTSIDE, "key.keyboard.escape", 0),
        Handled::Pass
    );
    press(&mut app, search_box_point(), 0);
    assert_eq!(press(&mut app, OUTSIDE, 0), Handled::Pass);
    assert!(!app.search.focused);
}

#[test]
fn control_f_focuses_the_box() {
    let mut app = app();
    assert_eq!(
        key_at(&mut app, OUTSIDE, "key.keyboard.f", MODIFIER_CONTROL),
        Handled::Consumed
    );
    assert!(app.search.focused);
}

#[test]
fn a_new_search_goes_back_to_the_first_page() {
    let mut app = app();
    let grid = Grid::list(&screen(OUTSIDE)).unwrap();
    app.refresh_listing();
    app.move_list(&grid, true);
    assert_eq!(app.first, grid.capacity());
    press(&mut app, search_box_point(), 0);
    type_text(&mut app, "i");
    assert_eq!(app.first, 0);
}

#[test]
fn pages_wrap_around_as_in_jei() {
    let mut app = app();
    let grid = Grid::list(&screen(OUTSIDE)).unwrap();
    app.refresh_listing();
    app.move_list(&grid, false);
    assert_eq!(app.first, app.last_first(&grid));
    app.move_list(&grid, true);
    assert_eq!(app.first, 0);
}

#[test]
fn scrolling_mode_moves_by_rows_and_stops_at_the_ends() {
    let mut app = app_with(Config {
        scrolling: true,
        ..Config::default()
    });
    let grid = Grid::list(&screen(OUTSIDE)).unwrap();
    app.refresh_listing();
    app.move_list(&grid, false);
    assert_eq!(app.first, 0);
    app.move_list(&grid, true);
    assert_eq!(app.first, grid.columns);
    for _ in 0..100 {
        app.move_list(&grid, true);
    }
    assert_eq!(app.first, app.last_first(&grid));
    assert!(app.listing.len() - app.first <= grid.capacity());
}

#[test]
fn the_wheel_turns_the_list_only_over_the_list() {
    let mut app = app();
    assert_eq!(
        app.on_input(&screen(list_slot(0)), Input::Scroll { y: -1.0 }),
        Handled::Consumed
    );
    assert!(app.first > 0);
    assert_eq!(
        app.on_input(&screen(OUTSIDE), Input::Scroll { y: -1.0 }),
        Handled::Pass
    );
}

#[test]
fn a_click_on_an_item_shows_recipes_and_a_right_click_uses() {
    let mut app = app();
    // The list: oak log, planks, stick, potion...
    assert_eq!(
        press(&mut app, list_slot(1), BUTTON_LEFT),
        Handled::Consumed
    );
    let lookup = app.lookup.as_ref().expect("open");
    assert_eq!(lookup.role, Role::Output);
    assert_eq!(lookup.focus, Some(Stack::of("minecraft:oak_planks")));
    app.close_layer();
    press(&mut app, list_slot(1), BUTTON_RIGHT);
    assert_eq!(app.lookup.as_ref().expect("open").role, Role::Input);
}

#[test]
fn an_item_with_no_recipes_opens_nothing() {
    let mut app = app();
    press(&mut app, list_slot(0), BUTTON_LEFT);
    assert!(!app.is_recipes_open());
}

#[test]
fn r_and_u_work_over_the_list() {
    let mut app = app();
    assert_eq!(
        key_at(&mut app, list_slot(1), "key.keyboard.r", 0),
        Handled::Consumed
    );
    assert!(app.is_recipes_open());
    app.close_layer();
    assert_eq!(
        key_at(&mut app, list_slot(1), "key.keyboard.u", 0),
        Handled::Consumed
    );
    assert!(app.is_recipes_open());
}

#[test]
fn keys_over_nothing_stay_the_games() {
    let mut app = app();
    for key in [
        "key.keyboard.r",
        "key.keyboard.u",
        "key.keyboard.a",
        "key.keyboard.e",
    ] {
        assert_eq!(key_at(&mut app, OUTSIDE, key, 0), Handled::Pass, "{key}");
    }
}

#[test]
fn r_over_a_slot_of_the_screen_follows_the_games_tooltip() {
    let mut app = app();
    app.note_tooltip_stack(Stack::of("minecraft:stick"));
    app.draw(&screen(OUTSIDE), Stats::default(), &measure);
    assert_eq!(
        key_at(&mut app, OUTSIDE, "key.keyboard.r", 0),
        Handled::Consumed
    );
    assert_eq!(
        app.lookup.as_ref().and_then(|l| l.focus.clone()),
        Some(Stack::of("minecraft:stick"))
    );
}

#[test]
fn a_tooltip_only_counts_for_the_frame_after_it() {
    let mut app = app();
    app.note_tooltip_stack(Stack::of("minecraft:stick"));
    app.draw(&screen(OUTSIDE), Stats::default(), &measure);
    app.draw(&screen(OUTSIDE), Stats::default(), &measure);
    assert_eq!(
        key_at(&mut app, OUTSIDE, "key.keyboard.r", 0),
        Handled::Pass
    );
}

#[test]
fn on_the_creative_screen_r_stays_a_letter_outside_the_panels() {
    let mut app = app();
    app.note_tooltip_stack(Stack::of("minecraft:stick"));
    let creative = ScreenInfo {
        kind: ScreenKind::Creative,
        ..screen(OUTSIDE)
    };
    app.draw(&creative, Stats::default(), &measure);
    let binds = KeyBinds::default();
    let handled = app.on_input(
        &creative,
        Input::Key {
            key: "key.keyboard.r",
            modifiers: 0,
            binds: &binds,
        },
    );
    assert_eq!(handled, Handled::Pass);
}

#[test]
fn the_page_takes_all_clicks_and_keys_and_escape_closes_only_it() {
    let mut app = app();
    assert!(app.open(&Stack::of("minecraft:stick"), Role::Output));
    assert_eq!(press(&mut app, OUTSIDE, 0), Handled::Consumed);
    assert_eq!(
        key_at(&mut app, OUTSIDE, "key.keyboard.e", 0),
        Handled::Consumed
    );
    assert_eq!(
        app.on_input(&screen(OUTSIDE), Input::Char('e')),
        Handled::Consumed
    );
    assert_eq!(
        app.on_input(&screen(OUTSIDE), Input::Scroll { y: 1.0 }),
        Handled::Consumed
    );
    assert_eq!(
        key_at(&mut app, OUTSIDE, "key.keyboard.f3", 0),
        Handled::Pass
    );
    assert_eq!(
        key_at(&mut app, OUTSIDE, "key.keyboard.escape", 0),
        Handled::Consumed
    );
    assert!(!app.is_recipes_open());
    assert_eq!(
        key_at(&mut app, OUTSIDE, "key.keyboard.escape", 0),
        Handled::Pass
    );
}

#[test]
fn three_levels_then_back_three_times_returns_to_the_start() {
    let mut app = app();
    let focus = |app: &App| app.lookup.as_ref().and_then(|l| l.focus.clone());
    app.open(&Stack::of("minecraft:stick"), Role::Output);
    app.open(&Stack::of("minecraft:oak_planks"), Role::Output);
    app.open(&Stack::of("minecraft:oak_log"), Role::Input);
    key_at(&mut app, OUTSIDE, "key.keyboard.backspace", 0);
    assert_eq!(focus(&app), Some(Stack::of("minecraft:oak_planks")));
    key_at(&mut app, OUTSIDE, "key.keyboard.backspace", 0);
    assert_eq!(focus(&app), Some(Stack::of("minecraft:stick")));
    app.go_forward();
    assert_eq!(focus(&app), Some(Stack::of("minecraft:oak_planks")));
    app.go_back();
    key_at(&mut app, OUTSIDE, "key.keyboard.backspace", 0);
    assert!(!app.is_recipes_open());
}

#[test]
fn clicking_a_slot_of_a_recipe_opens_that_item() {
    let mut app = app();
    app.open(&Stack::of("minecraft:stick"), Role::Output);
    let view = app.layer_view(&screen(OUTSIDE)).unwrap();
    let at = view.recipes[0].slots[0].at;
    press(&mut app, [at[0] + 2.0, at[1] + 2.0], BUTTON_LEFT);
    assert_eq!(
        app.lookup.as_ref().and_then(|l| l.focus.clone()),
        Some(Stack::of("minecraft:oak_planks"))
    );
}

#[test]
fn the_close_button_and_a_closed_screen_shut_the_page() {
    let mut app = app();
    app.open(&Stack::of("minecraft:stick"), Role::Output);
    let view = app.layer_view(&screen(OUTSIDE)).unwrap();
    press(&mut app, [view.close[0] + 1.0, view.close[1] + 1.0], 0);
    assert!(!app.is_recipes_open());
    app.open(&Stack::of("minecraft:stick"), Role::Output);
    app.on_screen_closed();
    assert!(!app.is_recipes_open());
}

fn place_button(app: &mut App) -> [f32; 2] {
    let view = app.layer_view(&screen(OUTSIDE)).unwrap();
    let rect = view.recipes[0].place.expect("a + button");
    [rect[0] + 1.0, rect[1] + 1.0]
}

#[test]
fn plus_places_a_known_recipe_and_closes_the_page() {
    let mut app = app();
    app.set_known(&[8]);
    app.open(&Stack::of("minecraft:stick"), Role::Output);
    let at = place_button(&mut app);
    app.on_input(
        &screen(at),
        Input::MousePress {
            button: 0,
            modifiers: MODIFIER_SHIFT,
        },
    );
    assert_eq!(
        app.take_actions(),
        [Action::PlaceRecipe {
            book_id: 8,
            all: true
        }]
    );
    assert!(!app.is_recipes_open());
}

#[test]
fn plus_does_nothing_for_an_unknown_recipe() {
    let mut app = app();
    app.open(&Stack::of("minecraft:stick"), Role::Output);
    let at = place_button(&mut app);
    press(&mut app, at, 0);
    assert!(app.take_actions().is_empty());
    assert!(app.is_recipes_open());
}

#[test]
fn cheat_mode_is_off_and_a_click_gives_nothing() {
    let mut app = app();
    press(&mut app, list_slot(1), BUTTON_LEFT);
    assert!(
        !app.take_actions()
            .iter()
            .any(|a| matches!(a, Action::Give { .. }))
    );
}

#[test]
fn in_cheat_mode_a_click_gives_one_and_shift_a_stack() {
    let mut app = app_with(Config {
        cheat_mode: true,
        ..Config::default()
    });
    press(&mut app, list_slot(2), BUTTON_LEFT);
    app.on_input(
        &screen(list_slot(2)),
        Input::MousePress {
            button: 0,
            modifiers: MODIFIER_SHIFT,
        },
    );
    press(&mut app, list_slot(2), BUTTON_MIDDLE);
    let stick = Stack::of("minecraft:stick");
    let give = |count| Action::Give {
        stack: stick.clone(),
        count,
    };
    assert_eq!(app.take_actions(), [give(1), give(64), give(64)]);
    assert!(!app.is_recipes_open());
}

#[test]
fn the_cheat_button_and_key_flip_it_and_save() {
    let mut app = app();
    let grid = Grid::list(&screen(OUTSIDE)).unwrap();
    let [x, y, _, _] = App::cheat_button(&grid);
    press(&mut app, [x + 1.0, y + 1.0], 0);
    assert!(app.config().cheat_mode);
    assert!(matches!(
        app.take_actions()[..],
        [Action::SaveConfig { .. }]
    ));
    key_at(&mut app, OUTSIDE, "key.keyboard.k", MODIFIER_CONTROL);
    assert!(!app.config().cheat_mode);
    assert_eq!(
        key_at(&mut app, OUTSIDE, "key.keyboard.k", 0),
        Handled::Pass
    );
}

#[test]
fn only_a_release_whose_press_was_ours_is_consumed() {
    let mut app = app();
    let release = Input::MouseRelease { button: 0 };
    assert_eq!(app.on_input(&screen(list_slot(0)), release), Handled::Pass);
    press(&mut app, list_slot(0), 0);
    assert_eq!(app.on_input(&screen(OUTSIDE), release), Handled::Consumed);
    assert_eq!(app.on_input(&screen(OUTSIDE), release), Handled::Pass);
}

#[test]
fn a_press_that_went_to_the_game_clears_a_stale_capture() {
    let mut app = app();
    press(&mut app, list_slot(0), 0);
    press(&mut app, OUTSIDE, 0);
    let release = Input::MouseRelease { button: 0 };
    assert_eq!(app.on_input(&screen(OUTSIDE), release), Handled::Pass);
}

#[test]
fn control_o_hides_the_list_and_frees_its_area() {
    let mut app = app();
    let at = list_slot(0);
    assert_eq!(
        key_at(&mut app, at, "key.keyboard.o", MODIFIER_CONTROL),
        Handled::Consumed
    );
    assert!(!app.config().overlay_visible);
    assert!(app.draw(&screen(at), Stats::default(), &measure).is_empty());
    assert_eq!(press(&mut app, at, 0), Handled::Pass);
    assert_eq!(key_at(&mut app, at, "key.keyboard.r", 0), Handled::Pass);
    key_at(&mut app, at, "key.keyboard.o", MODIFIER_CONTROL);
    assert!(app.config().overlay_visible);
}

#[test]
fn a_key_bound_elsewhere_is_followed() {
    let mut app = app();
    let mut binds = KeyBinds::default();
    binds.keys[BIND_RECIPE] = "key.keyboard.g".into();
    let at = screen(list_slot(1));
    let input = |key| Input::Key {
        key,
        modifiers: 0,
        binds: &binds,
    };
    assert_eq!(app.on_input(&at, input("key.keyboard.r")), Handled::Pass);
    assert_eq!(
        app.on_input(&at, input("key.keyboard.g")),
        Handled::Consumed
    );
    assert!(app.is_recipes_open());
}

#[test]
fn a_bookmarks_key_adds_and_removes_and_saves() {
    let mut app = app();
    key_at(&mut app, list_slot(2), "key.keyboard.a", 0);
    assert_eq!(
        app.bookmarks(),
        [Bookmark::Ingredient(Stack::of("minecraft:stick"))]
    );
    assert_eq!(app.take_actions().len(), 1);
    key_at(&mut app, list_slot(2), "key.keyboard.a", 0);
    assert!(app.bookmarks().is_empty());
}

fn bookmark_slot(n: usize) -> [f32; 2] {
    let grid = Grid::bookmarks(&screen(OUTSIDE)).unwrap();
    let [x, y] = grid.slot_position(n);
    [x + 2.0, y + 2.0]
}

fn with_bookmarks(items: &[&str]) -> App {
    let saved = Saved {
        bookmarks: items
            .iter()
            .map(|i| Bookmark::Ingredient(Stack::of(i)))
            .collect(),
        ..Saved::default()
    };
    App::new(&tables(), Config::default(), Lang::English, saved)
}

#[test]
fn a_click_on_a_bookmark_opens_it_on_release() {
    let mut app = with_bookmarks(&["minecraft:stick"]);
    assert_eq!(press(&mut app, bookmark_slot(0), 0), Handled::Consumed);
    assert!(!app.is_recipes_open());
    let release = Input::MouseRelease { button: 0 };
    assert_eq!(
        app.on_input(&screen(bookmark_slot(0)), release),
        Handled::Consumed
    );
    assert!(app.is_recipes_open());
}

#[test]
fn dragging_a_bookmark_onto_another_moves_it() {
    let mut app = with_bookmarks(&["minecraft:stick", "minecraft:oak_log", "minecraft:potion"]);
    press(&mut app, bookmark_slot(0), 0);
    app.on_input(&screen(bookmark_slot(2)), Input::MouseRelease { button: 0 });
    assert_eq!(
        app.bookmarks(),
        [
            Bookmark::Ingredient(Stack::of("minecraft:oak_log")),
            Bookmark::Ingredient(Stack::of("minecraft:potion")),
            Bookmark::Ingredient(Stack::of("minecraft:stick")),
        ]
    );
    assert!(matches!(
        app.take_actions()[..],
        [Action::SaveBookmarks { .. }]
    ));
    assert!(!app.is_recipes_open());
}

#[test]
fn a_bookmark_of_a_removed_mod_stays_saved_but_is_not_shown() {
    let mut app = with_bookmarks(&["gone:thing", "minecraft:stick"]);
    assert_eq!(app.bookmarks().len(), 2);
    assert_eq!(app.visible_bookmarks().len(), 1);
    let cmds = app.draw(&screen(OUTSIDE), Stats::default(), &measure);
    let drawn: Vec<_> = cmds
        .iter()
        .filter(|c| matches!(c, DrawCmd::Item { stack, .. } if stack.item == "gone:thing"))
        .collect();
    assert!(drawn.is_empty());
}

#[test]
fn a_recipe_bookmark_opens_that_one_recipe() {
    let mut app = app();
    app.open(&Stack::of("minecraft:stick"), Role::Output);
    let view = app.layer_view(&screen(OUTSIDE)).unwrap();
    let rect = view.recipes[0].bookmark;
    press(&mut app, [rect[0] + 1.0, rect[1] + 1.0], 0);
    assert_eq!(
        app.bookmarks(),
        [Bookmark::Recipe {
            id: "t:stick".into()
        }]
    );
    app.close_layer();
    press(&mut app, bookmark_slot(0), 0);
    app.on_input(&screen(bookmark_slot(0)), Input::MouseRelease { button: 0 });
    let lookup = app.lookup.as_ref().expect("open");
    assert_eq!(
        (lookup.focus.as_ref(), lookup.found[0].recipes.len()),
        (None, 1)
    );
}

#[test]
fn edit_mode_hides_items_and_they_stay_hidden_after_it() {
    let mut app = app_with(Config {
        edit_mode: true,
        ..Config::default()
    });
    press(&mut app, list_slot(2), 0);
    assert!(matches!(
        app.take_actions()[..],
        [Action::SaveHidden { .. }]
    ));
    assert!(!app.is_recipes_open());
    // Edit mode still shows it, marked.
    assert!(listed(&mut app).iter().any(|i| i == "minecraft:stick"));
    let cmds = app.draw(&screen(OUTSIDE), Stats::default(), &measure);
    assert!(
        cmds.iter()
            .any(|c| matches!(c, DrawCmd::Fill { argb, .. } if *argb == SLOT_HIDDEN))
    );
    key_at(&mut app, OUTSIDE, "key.keyboard.j", MODIFIER_CONTROL);
    assert!(!listed(&mut app).iter().any(|i| i == "minecraft:stick"));
}

#[test]
fn saved_hidden_items_are_hidden_from_the_start() {
    let saved = Saved {
        hidden: vec![Stack::of("minecraft:stick")],
        ..Saved::default()
    };
    let mut app = App::new(&tables(), Config::default(), Lang::English, saved);
    assert!(!listed(&mut app).iter().any(|i| i == "minecraft:stick"));
}

#[test]
fn history_is_off_by_default_and_most_recent_first_when_on() {
    let mut off = app();
    off.open(&Stack::of("minecraft:stick"), Role::Output);
    assert!(off.history.is_empty());
    let mut on = app_with(Config {
        history_enabled: true,
        ..Config::default()
    });
    on.open(&Stack::of("minecraft:stick"), Role::Output);
    on.open(&Stack::of("minecraft:oak_planks"), Role::Output);
    on.open(&Stack::of("minecraft:stick"), Role::Output);
    assert_eq!(
        on.history,
        [
            Stack::of("minecraft:stick"),
            Stack::of("minecraft:oak_planks")
        ]
    );
    assert!(
        on.take_actions()
            .iter()
            .all(|a| matches!(a, Action::SaveHistory { .. }))
    );
}

#[test]
fn the_drawing_has_the_list_buttons_and_a_tooltip_on_hover() {
    let mut app = app();
    let cmds = app.draw(&screen(list_slot(2)), Stats::default(), &measure);
    assert!(texts(&cmds).contains(&"Search..."));
    assert!(cmds.contains(&DrawCmd::Tooltip {
        lines: vec!["Stick".into(), "minecraft".into()],
        at: list_slot(2),
    }));
    let icons = cmds
        .iter()
        .filter(|c| matches!(c, DrawCmd::Item { .. }))
        .count();
    assert_eq!(icons, Grid::list(&screen(OUTSIDE)).unwrap().capacity());
}

#[test]
fn the_page_draws_above_the_list_and_cycles_its_slots() {
    let mut app = app();
    app.open(&Stack::of("minecraft:stick"), Role::Output);
    let cmds = app.draw(&screen(OUTSIDE), Stats::default(), &measure);
    let dim = cmds
        .iter()
        .position(|c| matches!(c, DrawCmd::Fill { argb, .. } if *argb == DIM));
    let list_box = cmds
        .iter()
        .position(|c| matches!(c, DrawCmd::Text { text, .. } if text == "Search..."));
    assert!(list_box < dim);
    assert!(texts(&cmds).contains(&"Recipes: Stick"));
}

#[test]
fn a_slot_with_several_options_shows_them_in_turn() {
    let stacks = [Stack::of("a"), Stack::of("b")];
    assert_eq!(App::cycle(&stacks, 0), Some(&stacks[0]));
    assert_eq!(App::cycle(&stacks, 1500), Some(&stacks[1]));
    assert_eq!(App::cycle(&stacks, 2000), Some(&stacks[0]));
    assert_eq!(App::cycle(&[], 0), None);
}

#[test]
fn stats_show_only_when_asked_for() {
    let stats = Stats {
        frames_per_second: 60,
        draw_micros: 12,
        time_millis: 0,
    };
    let mut hidden = app();
    let cmds = hidden.draw(&screen(OUTSIDE), stats, &measure);
    assert!(!texts(&cmds).contains(&"60 fps, draw 12 us"));
    let mut shown = app_with(Config {
        show_stats: true,
        ..Config::default()
    });
    let cmds = shown.draw(&screen(OUTSIDE), stats, &measure);
    assert!(texts(&cmds).contains(&"60 fps, draw 12 us"));
}

#[test]
fn a_long_search_shows_its_end() {
    let mut app = app();
    press(&mut app, search_box_point(), 0);
    let long = "x".repeat(60) + "END";
    type_text(&mut app, &long);
    let cmds = app.draw(&screen(OUTSIDE), Stats::default(), &measure);
    assert!(
        texts(&cmds)
            .iter()
            .any(|t| t.ends_with("END") && t.len() < long.len())
    );
}

#[test]
fn no_room_draws_nothing_and_takes_nothing() {
    let mut app = app();
    let tiny = ScreenInfo {
        kind: ScreenKind::Inventory,
        rect: [0.0, 0.0, 176.0, 166.0],
        window: [176.0, 166.0],
        mouse: [170.0, 10.0],
    };
    assert!(app.draw(&tiny, Stats::default(), &measure).is_empty());
    assert_eq!(
        app.on_input(
            &tiny,
            Input::MousePress {
                button: 0,
                modifiers: 0
            }
        ),
        Handled::Pass
    );
}

#[test]
fn new_tables_keep_the_search_and_the_bookmarks() {
    let mut app = with_bookmarks(&["minecraft:stick"]);
    press(&mut app, search_box_point(), 0);
    type_text(&mut app, "oak");
    app.set_tables(&tables(), Lang::English);
    assert_eq!(app.search_text(), "oak");
    assert_eq!(app.bookmarks().len(), 1);
    assert_eq!(listed(&mut app).len(), 2);
}

#[test]
fn new_settings_apply_at_once() {
    let mut app = app();
    assert_eq!(listed(&mut app).len(), 205);
    let config = Config {
        show_tag_recipes: false,
        language: LangChoice::Fixed(Lang::Portuguese),
        ..Config::default()
    };
    app.set_config(config, &tables(), Lang::English);
    let cmds = app.draw(&screen(OUTSIDE), Stats::default(), &measure);
    assert!(texts(&cmds).contains(&"Buscar..."));
    assert!(!app.config().show_tag_recipes);
}

#[test]
fn function_keys_are_told_apart() {
    assert!(is_function_key("f1") && is_function_key("f12"));
    assert!(!is_function_key("f") && !is_function_key("e") && !is_function_key("fx"));
}
