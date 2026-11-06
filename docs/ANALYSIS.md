# Analysis: porting JEI to Horizon

Scope: steps 1 to 3 of the port brief. No mod code was written.

**Evidence labels used in every document of this folder**

| Label | Meaning |
|---|---|
| **[RAN]** | I executed it and show or describe the result |
| **[READ]** | I read it in the code or docs, but did not execute it |
| **[ASSUMED]** | My inference. Not proven yet |

**How the JEI part was read.** A read-only helper agent mapped the JEI code and I then spot-checked its main claims
(search defaults, category list, cheat permission, removal of the suffix tree). The other JEI line references in
section 3 come from that map: treat them as [READ], second hand.

---

## 0. Five facts that change the brief

The brief was a good starting point, but these points in it do not match the code today.

1. **`.hmod` no longer runs code.** The brief describes `.hmod` as a sandbox/Wasm mod. Today it is data and assets only.
   `docs/mods.md:7` says "No code"; `cargo horizon-mod build` refuses a folder with a `Cargo.toml` (`docs/mods.md:55`);
   a file with native code carrying `mod.wasm` is refused (`docs/native-mods.md`, "One `.hmod` for content and code").
   The sandbox text in `horizon-mods/README.md` is out of date. **[READ]**
2. **`dctrxds-jei` is not a valid mod id.** Ids must match `[a-z0-9_]{1,64}` (`docs/api/mod-index.md:39`,
   `docs/api/mod-files.md`), so the hyphen is refused. The proposed id is **`dctrxds_jei`**. The folder can keep its
   hyphen. **[READ]**
3. **Two JEI-like mods already exist in the registry.** `horizon-mods/mods/jei.toml` ("JEI for Horizon" by enderliker,
   MIT, 0.1.0, hooks 2) and `horizon-mods/mods/recipes.toml` (official "Recipes", 2.0.0). The id `jei` is taken.
   See [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) Q1. **[READ]**
4. **The template's pin is old.** `compatibility.json` pins Horizon `ba6344a`, where `NATIVE_HOOKS_API` is **3**. Today
   it is **411**. Almost every hook this port needs came after API 3. **[RAN]** (`git show`)
5. **JEI no longer uses a suffix tree.** `GeneralizedSuffixTree` was removed in favour of Lucene (commit `c6800981e`).
   The Rust port needs its own substring index. **[RAN]** (`find` returns nothing) **[READ]** (commit, from the map)

---

## 1. Environment (step 1)

### Folders and revisions **[RAN]**

The sibling repositories are in the parent folder (`..`), not inside `./dctrxds-jei`.

| Folder | Branch | Commit | Last commit date |
|---|---|---|---|
| `horizon` | main | `41f822b659789780f58d32c63ddfb6887ea1b8bf` | 2026-11-06 |
| `horizon-mixins` | main | `5a64af938ec86b00d53d16a408aabc0038fb3e57` | 2026-11-05 |
| `horizon-mods` | main | `d2de4c18de0cfc4c1e4f193f2ec09b47adb0d236` | 2026-11-06 |
| `horizon-native-mod-template` | main | `cc8ce2d896782099abfa9f0abf26fa80c9e4bddb` | 2026-10-30 |
| `JustEnoughItems` | 26.3 | `fb7428472f019a0c13fb3fdc7fb5c4250ae869bc` | 2026-11-05 |
| `dctrxds-jei` | not a git repo | none | holds an empty `README.md` |

### `horizon-native-mod-template/compatibility.json` **[RAN]**

```json
{
  "horizon_revision": "ba6344a59cb9e755b730dcf02285fc0d33b97d3c",
  "source_sha256": {
    "crates/horizon-client/src/game.rs": "043c7853552104e8250b89987f0ff5de86cbcec0abece1d319c268e66644015f"
  },
  "horizon_repository": "https://github.com/horizon-rust/horizon",
  "horizon_mixins_repository": "https://github.com/horizon-rust/horizon-mixins",
  "horizon_mixins_revision": "d9c83d7096b7ea5b83678da2efdfd6161f74997e",
  "mixin_schema": 6
}
```

### Tools **[RAN]**

| Tool | Found | Note |
|---|---|---|
| OS | Windows 11 Pro | The template asks for Linux |
| `rustc` / `cargo` | 1.95.0 (`x86_64-pc-windows-msvc`) | Matches the template. No `rustup`, so the MSVC host is the only toolchain |
| `python3` | **Not working** (Microsoft Store stub) | `python` is 3.14.4. Scripts calling `python3` fail as written |
| `git` | 2.56.0 | |
| JDK | 25 present in `PATH`, but `java -version` answers 1.8 | Only matters for building Horizon from source |
| WSL | 2.7.3, only the `docker-desktop` distro | No usable Linux distro |
| `gh` | Installed, **not logged in** | Only matters for upstream PRs and issue searches |
| Minecraft | `%APPDATA%\.minecraft\versions\26.3` exists | |
| Horizon launcher | Installed, `nightly-2026-11-05-66f3981` (API 411) | `%APPDATA%\horizon-launcher` |
| `.local/` in `./horizon` | Absent | Harness and source runs need it. The launcher has its own install |

**To install (nothing is blocking analysis):** for the template's scripts, a Linux distro (`wsl --install -d Ubuntu`),
because `tools/horizon.py:5` imports `fcntl`, which does not exist on Windows **[RAN]**. Whether this is needed at all
is [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) Q3: the launcher builds native mods on Windows
(`docs/native-mods.md`, "What the launcher does", step 1) **[READ]**.

### Revisions disagree, and that matters

| Who | horizon-mixins pin |
|---|---|
| Template | `d9c83d7` |
| Official Recipes mod | `e8cd0aa` |
| Launcher (`launcher/src/native_mods.rs:42`) | `5a64af9`, the same as the local checkout |

The launcher uses the newest pin among the mods and its own (`docs/native-mods.md`, "Pinned to a Nightly"), so we should
follow the launcher, not the template. The launcher also compiles with **its own Rust 1.99.1**, not your 1.95.0
(`docs/native-mods.md`, step 1). Mod code must compile on both.

---

## 2. Horizon (step 2)

### What was read

| Read fully | Read in part (say so honestly) |
|---|---|
| `README.md`, `docs/README.md`, `docs/mods.md`, `docs/api/mod-index.md`, `CONTRIBUTING.md`, `AGENTS.md`, `CLAUDE.md` (one line, pointing to `AGENTS.md`), all of the template's docs and example, `MOD-POLICY.md`, `native/recipes/*`, `horizon-mixins/README.md` | `docs/native-mods.md` (3,861 lines: the API list to 46, bundle format, launcher, trust, trait mods, and each API note that a search matched), `native_hooks.rs` (11,551 lines: the signatures of relevant functions), `docs/api/mod-files.md` (first 50 lines), `horizon-mixins/docs/effects-and-state.md` (about the first 150 lines) |

Only the headings and search hits were read in `docs/native-client.md` and `ROADMAP.md`: nothing in them defines a mod
API. `mods/horizon/src` was only listed: it is Java test oracles and benchmarks (`AGENTS.md`: "Java oracles"), not a mod
API, so it does not help this port.

### Crates that matter here **[READ]** (`AGENTS.md`)

Horizon has 39 crates. These are the ones this port touches:

| Crate | Responsibility | Why it matters |
|---|---|---|
| `horizon-client` | Gameplay: `game.rs` + ~180 `game/*.rs`, and `mod_api.rs` | All hook points live here |
| `horizon-client-render` | Screens, HUD, `ui.rs`, fonts, `providers.rs` | Drawing, `HudBatch`, text |
| `horizon-client-input` | Key bindings and key names | Input |
| `horizon-items` | Item facts and components (`item_data.rs`) | `ItemStack`, components |
| `horizon-mods`, `cargo-horizon-mod` | `.hmod`, registry index, checks | Packaging and publishing |
| `horizon-mod-api-gen` | Generates the typed mod API | "Trait mods" |
| `horizon-protocol` | Network protocol | Client packets |
| `launcher/` (separate workspace) | `native_mods.rs`: builds native mods | How players get our mod |

Other inventory and recipe code is in `horizon-client`: `menu.rs`, `crafting.rs`, `creative.rs`, `recipe_book.rs`.

### What `.hmod` offers **[READ]**

**No code API exists.** An `.hmod` holds `horizon-mod.toml`, `assets/` and `data/`. Blocks, items, creative tabs, mob
effects, entities and menu types are declared as JSON under `data/<id>/horizon/` (`docs/mods.md:82-369`). Nothing there
can draw, read input or react to events. It can help a native mod by carrying GUI textures: `native_sprite` accepts "a
content mod's GUI texture `mymod:gui/...`" (`native_hooks.rs`, doc comment of `native_sprite`), and one `.hmod` may
hold both content and native code in the future (`docs/api/mod-files.md`).

### What native mods (`.hnmod`) offer

A **hook point** is an empty function the game calls at a fixed moment. A mod adds code to it. A **helper** is a
function the mod calls to ask the game for something. The catalog is `native-hooks.json`: **432 hooks and 174 helpers**
**[RAN]**. All paths below are in `crates/horizon-client/src/game/` unless noted, and all are **[READ]**. The ones
marked ★ also compiled in my experiment (section 2.1).

| Need | Hooks and helpers | Location |
|---|---|---|
| Draw over any screen | ★`native_screen_render_post(&self, batch, screen)` | `native_hooks.rs:3582` |
| Screen facts | ★`NativeScreenView { id, menu, title, rect, slots, mouse, partial_tick }` | `mod_api.rs:200` |
| Draw primitives | ★`native_fill`, ★`native_text`, `native_text_width`, `native_sprite`, `native_tiled_sprite`, `native_quad`, `native_mod_image` | `native_hooks.rs` (around 9168-9350) |
| Draw items | ★`native_item`, `native_item_stack` (API 313), `native_item_model` | `native_hooks.rs:9281,9300,9324` |
| Widgets, tooltips | `native_gui_paint`, `native_gui_nine_slice`, ★`native_gui_tooltip` (API 411) | `native_client_services.rs:224,228,248` |
| Mouse in a screen | ★`native_screen_mouse_button_pressed_pre`, `_released_pre`, `_dragged_pre`, ★`_scrolled_pre` (all can cancel) | `native_hooks.rs:3610,3639,3658,3701` |
| Keyboard in a screen | ★`native_screen_key_pressed_pre`, `_key_released_pre`, ★`_char_typed_pre` (all can cancel) | `native_hooks.rs:3741,3761,3800` |
| Mouse and keys anywhere | `native_mouse_button_pre/post`, `native_input_key`, `native_key` | `native_hooks.rs:2711,2728,2706,2700` |
| Own keys in Options | `NativeMod.keys` through `native_mods`; `native_mod_key`, `native_set_mod_key` | `mod_api.rs:316`, `native_hooks.rs:3133,9576`, `native_client_services.rs:252` |
| Items, tags, recipes | ★`native_recipes() -> Option<Arc<NativeRecipes>>` | `native_hooks.rs:9742`, structs at `:2094-2140` |
| Custom recipe types | `native_recipes_of_type`, `native_recipe_ingredient`, `native_recipe_result` | `native_hooks.rs` (API 36) |
| Known recipes, placing | `native_known_recipes`, `native_place_recipe(book_id, all)` | `native_hooks.rs:9748,9759` |
| Give items | ★`native_give(item, count)`: **creative only, no components** | `native_hooks.rs:9766`, `native_recipes.rs:119` |
| Ready-made item panel | `native_panel_items`, `native_panel_recipes`, `native_panel_event` | `native_hooks.rs:9642,9650,3148`, `native_panel.rs` |
| Tooltip edits | `native_tooltip(&self, stack, lines)` and related | `native_hooks.rs:6667-6708` |
| Files | ★`native_config_dir()` (API 26), `native_data_dir`, ★`native_setting`, `native_set_setting`, `native_register_configs` and `native_config_*` (API 397) | `native_hooks.rs:9634,9618,9623,2492-2515` |
| Screen events | `native_screen_opening`, `native_screen_init_post`, `native_screen_closing` | `native_hooks.rs:3551,3556,3561` |
| Reload and join events | `native_client_resources_reloaded`, `native_recipes_received`, `native_client_level_load`, `client_start`, `client_disconnecting` | `native_hooks.rs:2290,2390,2300` |
| Every server packet | `native_client_packet` (read only, API 408) | `native_hooks.rs:7153` |
| Frame and tick (`&mut self`) | ★`native_frame`, `native_tick` | `native_hooks.rs:2251,2314` |
| Talk to the server | `native_send_chat` (works on vanilla servers), `native_send_packet`, `native_net_send_to_server`, `native_net_send_to_player` | `native_net.rs:174`, `docs/native-mods.md` |
| Server-side give | `native_give_stack`, `native_give_items`, `NativeServerPlayer.permission` | `docs/native-mods.md`, `mod_api.rs` |
| Mods using each other | Trait mods: `[api]` package, `[dependencies]` (**"draft"**) | `docs/native-mods.md`, "Dependencies between trait mods" |

A mod has three ways to hook in (`docs/native-mods.md`):

- **Named hook** (`hook = "frame"`, `hook_version = 1`): stable across releases. If a mod uses only these and catalog
  helpers, it can declare `"hooks": N` and build on any Horizon with API N or later, with no exact pin.
- **Raw `target`** (any function): powerful but can break on any commit, so the mod must pin an exact `horizon_revision`.
- **`[api]` trait mod**: a normal Rust crate implementing `Hooks`. The catalog's `Host` lacks `recipes` and
  `panel_recipes` **[RAN]** (helper list checked), so a trait mod alone cannot read the recipe table. The official
  Recipes mod uses helpers and modules.

### Things that shape the design **[READ]**

- **Many input and draw hooks take `&self`** (the game is read-only there). `native_give` and `native_recipes` take
  `&mut self`. So a click hook cannot give an item directly: it records an action, and `native_frame` carries it out.
- **The game's panel is limited.** It sits only to the right, and is drawn only on `Screen::Inventory`, not on the
  creative screen (`game.rs:9241-9243`). It has 5 fixed recipe layouts, 16 tabs and 256 recipes per page, and 8,192 items.
  No shift info reaches the mod through `NativePanelEvent` (`mod_api.rs:340`), and there is no place for bookmarks.
- **`native_mod_screen_open` works only from `Screen::Playing`** (`native_hooks.rs:10745-10758`). It cannot open over an
  inventory. JEI's recipe view must therefore be a layer we draw and whose input we cancel, on top of the open screen.
  This also keeps the container open underneath, which transfer needs.
- **Gaps in the stable catalog**: no helper for the hovered slot, creative tabs, a full item tooltip, fuel, compost,
  or giving items with components. The code exists but is private: `slot_at` and `slot_view` (`game.rs:9524,9593`),
  `creative_screen` (`game.rs:3045`, `creative.rs:14,68`), `stack_tooltip` (`stations.rs:2102`), `burn_time`
  (`providers.rs:108`), `compostable` (`composter.rs:24`), `mod_give` (`native_recipes.rs:148`).
- **Horizon changes fast.** 923 commits are between the installed Nightly and `main` **[RAN]**, even though both report
  API 411.

### Project rules that apply **[READ]**

- **Never commit** game files, decompiled code or `.local/`. Keep test runs away from real player data.
- **Upstream PRs** (only if we do any): `/claim` the issue first, under about 500 lines, run `tools/pr_ready.py`, mark
  agent-opened PRs, `Co-Authored-By` on commits. Horizon-only features are off by default.
- **License of what we read:** MIT is allowed by `CONTRIBUTING.md`; state what was read in any PR.
- **`--trust-native-code` is your decision, never an agent's** (`docs/native-mods.md`). I did not use it and will not.

### 2.1 Experiments **[RAN]**

All in the scratch folder. No repository changed: `git status` is empty in `horizon`, `horizon-mixins` and `horizon-mods`.
The bundle stays at
`C:\Users\jpsen\AppData\Local\Temp\claude\a8170955d3cd19f0\a97a7b5a-2934-4ba3-a70d-996c0a73b1c6\scratchpad\probe-mods\dctrxds_jei_probe`.

| # | What I ran | Result | What it proves |
|---|---|---|---|
| 1 | `cargo test` on a copy of `native/recipes` | 5 passed | Pure Rust tests run on this Windows machine |
| 2 | `cargo build -p horizon-mixins-cli` | OK, 16 s | The mixin tool builds on Windows |
| 3 | `horizon-mixins plan` with a 6-hook probe on `horizon` HEAD | `"status": "ok"`, 24 s | Named hooks resolve; `plan` works on Windows |
| 4 | `horizon-mixins apply` to a new folder | `"status": "ok"` | Staging works on Windows |
| 5 | `cargo check --locked -p horizon-client --no-default-features --bin horizon-client` on the stage | OK, 1 m 24 s | The ★ APIs compile together on HEAD |
| 6 | Same, with a misspelled method put in the probe on purpose | `error[E0599]: no method named native_fill_does_not_exist` | Experiment 5 really checks our code |
| 7 | `apply` over an existing stage | `E_ATOMIC_STAGE: ... requires Linux; use a fresh output directory` | On Windows, each apply needs a new folder |
| 8 | `python -c "import fcntl"` | `ModuleNotFoundError` | The template's `tools/horizon.py` cannot run on Windows |

**What these do NOT prove.** The game was never run with the mod, so the overlay has not been seen on screen. The probe
never executed, the default features (audio and controllers) were not checked, the launcher's real toolchain
(`windows-gnu`, Rust 1.99.1) was not used, and whether U+00B4 renders is unknown. Phase 0 tests all of this.

---

## 3. JEI (step 3)

Paths use these prefixes: `C/` = `Common/src/main/java/mezz/jei/common`, `A/` = `Common/src/api/java/mezz/jei/api`,
`L/` = `Library/src/main/java/mezz/jei/library`, `G/` = `Gui/src/main/java/mezz/jei/gui`, `V/` = `L/plugins/vanilla`.

### Size **[RAN]**

| Module | Java files | Lines | Role | Port? |
|---|---|---|---|---|
| Common | 463 | 32,924 | Public API, config, search storage, network, transfer | Logic yes; network adapted |
| Library | 268 | 27,146 | Plugin loading, ingredients, recipes, vanilla categories | Logic yes |
| Gui | 256 | 29,358 | Overlay, recipe screen, bookmarks, filter, input | Logic yes; drawing rewritten |
| Debug | 26 | 2,034 | Sample plugin | No; an API example only |
| VanillaServer | 0 | 0 | NeoForm test-server launch | No |
| Fabric | 107 | 10,374 | Loader glue | No |
| NeoForge | 117 | 16,335 | Loader glue | No |

### Data flow

```text
Recipes reach the client (server sync, or the built-in vanilla data)
  → JeiStarter runs PluginLoader, in this order:
      subtypes → ingredients → aliases → categories → extensions → catalysts
      → RecipeManagerInternal → recipes → compact → transfer → GUI handlers → runtime
  → JeiGuiStarter builds IngredientFilter, the overlays and the input handlers
  → Each frame: the overlay draws the grid; input goes through GuiInputController
  → R or U over an item: FocusGroup → RecipeManager lookup → RecipesGui shows the layouts
```

Everything rebuilds on recipe sync or datapack reload. A client resource reload rebuilds only the ingredient filter
(`G/startup/ResourceReloadHandler`).

### Ingredients

- Types: `A/ingredients/IIngredientType`, `ITypedIngredient`, `IIngredientHelper` (`getUid`, `getGroupingUid`, colours, tags).
- An item's identity is `item + subtype data` (`C/util/StackHelper.java:44-52`). The grouping uid is the item.
- **The list comes from creative tabs** (`V/ingredients/ItemStackListFactory.java:51`), then is deduplicated. This is
  how potions and enchanted books appear. Horizon's `NativeRecipes.items` has only item ids.
- Fluids are a separate type. Horizon's item panel and recipes are item-only; the port keeps fluids out of scope at first.

### Search

**Grammar** (`G/search/SearchTokenizer.java`, 144 lines): space = AND, `|` = OR, leading `-` = exclude, `"..."` = quoted,
`\` = escape. The prefix can precede a quote: `@"foo bar"`.

**Prefixes** (`G/search/ElementPrefixParser.java:47-100`; defaults in `C/config/IngredientFilterConfig.java:20-31`, checked
by me):

| Prefix | Searches | Default mode |
|---|---|---|
| `@` | Mod | REQUIRE_PREFIX |
| `#` | Tag | REQUIRE_PREFIX |
| `$` | Tooltip | ENABLED |
| `%` | Creative tab | DISABLED |
| `^` | Colour | DISABLED |
| `&` | Identifier | DISABLED |

**Evaluation** (`G/ingredients/IngredientFilter.java`, 496 lines): text in lower case; each group intersects from its
smallest set and subtracts exclusions; groups are united, filtered for visibility, and sorted by a precomputed integer index.

**Storage:** `C/search/LuceneSearchStorage` does `*token*` wildcard queries, i.e. substring search. Rust has no Lucene
here (a mod's crate may use only crates in Horizon's `Cargo.lock`), so we write a small substring index.

**Sorting:** `G/ingredients/IngredientSorter`. Stages: MOD_NAME, INGREDIENT_TYPE, ALPHABETICAL, CREATIVE_MENU, TAG, ARMOR,
MAX_DURABILITY.

### Recipes

- `L/recipes/RecipeManager` wraps `RecipeManagerInternal` and `PluginManager`.
- Index in `L/recipes/collect/`: one map per role (INPUT, OUTPUT, CRAFTING_STATION) from recipe type → ingredient type →
  uid, holding exact and grouping matches.
- Focus: `A/recipe/IFocus`, `RecipeIngredientRole`, `L/focus/FocusGroup`.
- UI state: `G/recipes/RecipeGuiLogic` (451 lines), `NavigationHistory` (38 lines, back and forward).
- Layout: `RecipeLayoutBuilder`, `RecipeSlotBuilder`, `RecipeSlot`; slots cycle their options once per second.

### Vanilla categories (`V/VanillaPlugin.java:279-293`, checked by me)

| Category | Data source in JEI | In `native_recipes`? **[READ]** |
|---|---|---|
| Crafting | Recipe map, with tipped arrow and shield expanded | Yes; special ones come with `book_id` -1 |
| Stonecutting, smelting, smoking, blasting, campfire, smithing | Recipe map | Yes |
| Brewing | Platform helper | Yes |
| Smelting, smoking, blasting fuel | Computed from `getFuelProperties` | **No** |
| Compostable | Computed from `getCompostValue` | **No** |
| Anvil | Computed via a fake menu | **No** |
| Grindstone | Computed via a fake menu | **No** |
| `jei:information` | Added by plugins | Ours to define |
| Tag categories | Computed from registries | Partly, from `NativeItem.tags` |

### Overlay and input

- `G/overlay/IngredientListOverlay` (430 lines) sits right of the screen; `BookmarkOverlay` (574) sits left.
- `IngredientGridLayout.calculateSlots` (`:71`): a slot touching an exclusion area plus 2 px is blocked; the rest stay put.
- Screens: any with a handler. Defaults are container screens, chat and the recipe screen (`G/plugins/JeiGuiPlugin.java:36-38`).
- `G/startup/JeiInputSetup` layers areas front to back: search, tooltips, recipes, ingredient list, bookmarks, container, chat.
- `G/input/GuiInputController` captures a mouse press until its release.

### Keys (`G/config/InternalKeyMappings.java:124-333`)

Main ones: toggle overlay Ctrl+O; focus search Ctrl+F; bookmark A; show recipe R or left click; show uses U or right click;
back Backspace; recipe pages PgUp and PgDn; categories Shift+PgUp and Shift+PgDn; pause cycling Left Shift; cheat one item
left or right click; cheat a stack Shift+left or middle click. Bindings conflict only within the same context
(`C/input/keys/JeiKeyConflictContext.java`).

### Transfer and cheat

- **Transfer** (`L/transfer/BasicRecipeTransferHandler.java:97`): check slots and room, assign items with backtracking
  (`C/transfer/RecipeTransferUtil.java:149`), then send packets. **It needs JEI on the server**; on a vanilla server the
  user gets an error tooltip.
- **Cheat** (`G/util/CommandUtil.giveStack`): uses JEI's packet when the server has JEI, or creative only otherwise. No
  `/give` fallback was found. Permission (`C/util/ServerCommandUtil.java:35`, checked by me): creative if enabled, or op
  if enabled, or the `give` command if enabled.

### Bookmarks and config

- `BookmarkType` is INGREDIENT or RECIPE. JSON via `G/bookmarks/BookmarkCodec`, files `bookmarks.json` and
  `lookupHistory.json`, per world or server.
- Config comes from the external library `net.mezzdev.config` (not in this repo), in `jei-client.ini` and others, plus
  `blacklist.json`. We use our own format.

### API for plugins

Main interfaces: `IModPlugin` (21 hooks), `IRecipeCategory`, `IRecipeLayoutBuilder`, `IIngredientType`, `IIngredientHelper`,
the `A/registration/*` set, `IRecipeTransferHandler`, `IGuiContainerHandler`, `IGhostIngredientHandler`, `IJeiRuntime`.
Found through a Fabric entrypoint or a NeoForge annotation.

### Features beyond your list

Recipe bookmarks, lookup history row, search completion and syntax highlighting, ingredient and mod aliases, colour search
(MMCQ), pinned tooltips, keyboard grid navigation, scrollable grids and drag resizing, chat item links, tag content
tooltips, ghost-ingredient drag, edit mode to hide ingredients, recipe sorting by bookmarked or craftable.

### Discard

- All of `Fabric/` and `NeoForge/`, including mixins, plugin discovery and loader events.
- The `C/platform/IPlatform*Helper` interfaces. They are still a useful **checklist** of what the game must supply: fuel,
  compost, brewing, render, hovered slot, recipe book.
- `VanillaServer/`, gametests, Lucene, MezzConfig, the Java codecs, and the frozen legacy packets.

### Worth porting faithfully

| Algorithm | Source | Why | Tests to port |
|---|---|---|---|
| Search tokenizer | `SearchTokenizer` | Users know this grammar | `SearchTokenizerTest` (41) |
| Prefix and mode rules | `ElementPrefixParser`, `ElementSearch` | Exact behaviour matters | `ElementSearchTest` (1) |
| AND/OR/NOT evaluation | `IngredientFilter` | Same results as JEI | `IngredientFilterTest` (22, needs fixtures rewritten) |
| Storage behaviour | `LimitedStringStorage`, Lucene storage | Behaviour, not code | `LuceneSearchStorageTest` (12), `LimitedStringStorageTest` (4) |
| Grid layout with exclusions | `IngredientGridLayout` | Hard to get right by guessing | `IngredientGridConfigTest` (60), `IngredientGridPageStateTest` (19), `GridScrollMathTest` (11) |
| Recipe index by role | `L/recipes/collect/*` | Fast lookups, correct uses | `IngredientUidIndexTest` (3), `InternalRecipeManagerPluginTest` (16, fixtures) |
| History | `NavigationHistory` | Small and exact | `NavigationHistoryTest` (3) |
| Transfer assignment | `RecipeTransferUtil` | Tricky backtracking | Written anew |
| Colours (later) | `MMCQ`, `ColorThief` | Only if `^` search is wanted; has its own licence header | `ColorThiefTest` (12) |
