# Roadmap

Sizes are rough **[ASSUMED]**: **Small** = days, **Medium** = 1 to 2 weeks, **Large** = several weeks, for one
student working part time with an agent's help. Each phase is usable by itself.

Every phase also has to pass: `cargo fmt`, `cargo clippy -D warnings`, `cargo test` on the core, and `plan` + `apply` +
`cargo check` on the bundle.

## Overview

| Phase | Result | Size |
|---|---|---|
| 0 | Proof: real overlay in the real game on Windows | Small |
| 1 | Item list with JEI's search | Medium |
| 2 | Recipes and uses | Large |
| 3 | First public release (registry) | Small |
| 4 | Fill the grid, give items | Medium |
| 5 | Bookmarks, hiding, settings, keys | Medium |
| 6 | Item variants and missing categories | Medium to Large |
| 7 | API for other mods | Large |

## Phase 0: proof of concept, end to end

**Goal:** close the gap between "it compiles" and "it works": the smallest thing that touches every risky part.

**Deliverables**
- Workspace, `LICENSE`, `NOTICE.md`, bundle with id `dctrxds_jei`
- Grid of the first items from `native_recipes`, right of the screen, on inventory and creative
- Click on an item: consumed by us, and gives one in creative
- Typing a letter while hovering: appended to a text shown on screen; written to `config/dctrxds_jei/probe.json`, reloaded on restart
- Title `DctrXD´s JEI` drawn on screen
- Options > Mods row with a key

**Acceptance (checked by you, in a throwaway instance)**
- [ ] `horizon-launcher native-build` succeeds on Windows. **You run it**: `--trust-native-code` is your decision
- [ ] The grid appears on the survival inventory, a chest and the creative screen
- [ ] A click on the grid does not reach the screen below; a click elsewhere works normally
- [ ] Creative click gives the item; survival refuses without a crash
- [ ] The saved text is back after restart
- [ ] `´` renders, or we switch to `'` and tell you why
- [ ] Hovering a chest slot: does the `tooltip` hook tell us the stack? Yes or no decides the R and U route
- [ ] Note the build time, and whether the frame rate visibly drops

**Tests:** small core test for grid math.

**Risks:** the launcher's build fails on Windows; draw order is wrong; hooks do not fire on some screen. Each outcome
changes the plan, which is why this phase comes first.

**Size:** Small.

## Phase 1: item list and search

**Goal:** the list you open daily is useful.

**Deliverables**
- `search::*` with full grammar and prefixes `@`, `#`, `&`; `$` on names only for now
- Own search box: cursor, selection, focus, Ctrl+F, right click to clear
- Grid with pages and scroll, exclusion areas, resizing with GUI scale
- Sort stages except CREATIVE_MENU, which falls back to registry order
- Toggle overlay key, tooltip with name and mod

**Acceptance**
- [ ] The ported `SearchTokenizerTest` (41), `LimitedStringStorageTest` (4), `GridScrollMathTest` (11) and the applicable
      grid tests pass
- [ ] `sword -iron` and `@minecraft #logs|#planks` return what JEI returns on the same inputs (a table in the tests)
- [ ] Typing does not reach the game: `e` in the box does not close the inventory
- [ ] Results update within one frame on the vanilla item list, measured and written down
- [ ] Overlay does not cover the container at any GUI scale from 1 to 4; no crash on a tiny window

**Tests:** ported JEI tests, tokenizer edge cases, index against a brute-force search on random data.

**Risks:** recipe book and effects overlap (`rect` might not include them); performance with mods adding thousands of items.

**Size:** Medium.

## Phase 2: recipes and uses

**Goal:** the main reason JEI exists.

**Deliverables**
- Recipe index by role; lookups
- Recipe layer over the open screen, with its input consumed
- R and U over list items; over container slots if the Phase 0 route worked; left and right click on the list
- Categories: crafting (shaped and shapeless), smelting, smoking, blasting, campfire, stonecutting, smithing, brewing
- Category tabs with catalyst icons, page navigation, back and forward, slot option cycling, cooking time and XP, tag information on tooltips

**Acceptance**
- [ ] For a test table, recipes and uses of an item match expected lists (ported `IngredientUidIndexTest` and the applicable `InternalRecipeManagerPluginTest` cases)
- [ ] Backspace goes back; Escape closes our layer and **not** the container below
- [ ] In-game: oak planks show the log recipe; stick shows uses across categories; every vanilla category opens
- [ ] A clicked slot in a recipe opens that item; three levels then back three times returns to the start
- [ ] Reloading resources and joining another world leave no stale data
- [ ] Vanilla server and own world both tested. On a vanilla server, say on screen that the list shows this game's recipes

**Tests:** index, navigation (`NavigationHistoryTest`), slot layouts.

**Risks:** special recipes shown oddly; the drawing layer is the largest UI work; tooltip route may fail.

**Size:** Large.

## Phase 3: first public release

**Goal:** real users, sooner than a "complete" version.

**Deliverables**
- `README.md` with the required statements; `.hnmod` via a packaging tool; icon (64 to 1024 px, ≤ 256 KiB); description ≥ 50 characters; minisign key
- Honest list of what is missing, including differences from JEI
- **You** open the submission, since it is your name and promise

**Acceptance**
- [ ] `cargo horizon-mod check` passes on the file (run its output into the PR)
- [ ] A clean instance installs and builds it from the file
- [ ] Checklist in [RISKS.md](RISKS.md) is complete

**Risks:** reviewer asks for changes; a similar mod is already listed (see Q1).

**Size:** Small.

## Phase 4: fill the grid, give items

**Goal:** turn "look at a recipe" into "do the recipe".

**Deliverables**
- `+` button using `native_place_recipe`, Shift for max, greyed when not in the book, with a reason
- Cheat mode toggle; creative give; `/give` route for ops, with the command's error shown to the user
- Stretch: server half for the same permission rules JEI uses (`ServerCommandUtil`)

**Acceptance**
- [ ] `+` fills inventory and crafting table on single player and a vanilla server
- [ ] Not offered on screens that cannot take it
- [ ] Non-op on a server: cheat refuses cleanly, sends nothing repeatedly
- [ ] Cheat mode is off by default

**Risks:** servers with anti-spam; the `/give` syntax with components.

**Size:** Medium.

## Phase 5: bookmarks, hiding, settings

**Goal:** daily comfort and lasting preferences.

**Deliverables**
- Bookmark overlay on the left, ingredient then recipe bookmarks, drag to reorder; A key
- History row (off by default, as JEI has it)
- Edit mode and `hidden.json`
- `client.toml` with hot reload, settings page, all keys rebindable, translations with `pt_br` and `en_us` first

**Acceptance**
- [ ] Bookmarks survive restarts; a corrupt file gives a warning and a backup, and no crash
- [ ] Removing a mod that provided a bookmark does not break the list
- [ ] Editing `client.toml` while playing takes effect
- [ ] Key conflicts within a context are shown

**Size:** Medium.

## Phase 6: item variants and missing categories

**Goal:** closer parity. **This depends on upstream or on a pin** ([OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) Q4).

**Deliverables**
- Variants in the list (potions, enchanted books) through creative tabs
- `%tab` search; CREATIVE_MENU sort; `$tooltip` on real lines
- Full vanilla tooltip on our slots
- Fuel, compost, anvil and grindstone categories; information pages; tag categories
- Upstream: small PRs, each under about 500 lines and with its issue claimed

**Acceptance**
- [ ] Every potion type shows in the list; each opens its brewing recipes
- [ ] Fuel values match the game for a sample (furnace test in game)
- [ ] Without the upstream helper, the mod degrades cleanly instead of failing to build

**Risks:** maintainers decline a helper; anvil computation is complex in JEI (fake menus).

**Size:** Medium to Large.

## Phase 7: API for other mods

**Goal:** other mods can show their recipes.

**Deliverables**
- Data-driven categories, with a documented JSON format and a sample content mod
- After that, a Rust `[api]` with `register_category`, plus an example mod

**Acceptance**
- [ ] A sample `.hmod` with a custom recipe type appears in its own category with no Rust
- [ ] A sample native mod registers a category, with and without our mod installed (optional dependency)
- [ ] Removing the API's user leaves everything working

**Risks:** "draft" trait-mod format; no runtime plugin discovery.

**Size:** Large.

## Not planned

Fluids as ingredients, colour search, plugin-compatible Java API, and hand-made click packets for furnace-style transfer on
vanilla servers. Each can be reopened if you ask.
