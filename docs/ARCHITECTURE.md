# Architecture

Status: proposal. Nothing here exists yet.

## The idea

Split the mod so that almost everything can be tested with plain `cargo test`, without the game.

```text
┌────────────────────────── the game (Horizon) ──────────────────────────┐
│ hooks: frame, screen_render_post, screen_mouse_*, screen_key_*, ...     │
└───────────┬────────────────────────────────────────────▲───────────────┘
            │ events and tables (converted)              │ draw and actions
┌───────────▼────────────────────────────────────────────┴───────────────┐
│ ADAPTER: game::dctrxds_jei  (only code that knows Horizon's types)      │
│ tiny helpers per hook · converts types · paints DrawCmds · runs Actions │
└───────────┬────────────────────────────────────────────▲───────────────┘
            │ plain Rust types                           │ Vec<DrawCmd>, Vec<Action>
┌───────────▼────────────────────────────────────────────┴───────────────┐
│ CORE crate: dctrxds-jei-core  (no game dependency)                      │
│ ingredients · search · sort · recipes · focus/history · layout · input  │
│ bookmarks · config                                                      │
└─────────────────────────────────────────────────────────────────────────┘
```

Terms:

- **Crate:** a Rust package.
- **Hook:** a place where the game calls our code.
- **Adapter:** code translating between the game's types and ours.
- **Pure logic:** code that only depends on its inputs, so it is simple to test.

## Why this shape works with Horizon's rules **[READ]**

- A bundle may contain a `[[crate]]`. Cargo compiles it once and keeps it, so editing a helper does not recompile it.
  "The heavy code of a big mod belongs there and only the glue in helpers" (`docs/native-mods.md`, "The mod's crate").
- That crate **cannot depend on `horizon-client`** or on any path or git crate, and only on registry crates in Horizon's
  `Cargo.lock`. This enforces the separation we want. `serde`, `serde_json` and `toml` are in the lock **[RAN]**.
- The crate folder may not have its own `Cargo.lock` or `[workspace]`, so the workspace file stays at the repository root.
- Runtime modules cannot use inner attributes, `//!` comments or external `mod` files (template README). Helpers stay small.

## Repository layout

```text
dctrxds-jei/
├── Cargo.toml                     # workspace for local tests; not in the .hnmod
├── LICENSE                        # MIT: our copyright and mezz's notice
├── NOTICE.md                      # what derives from JEI, and from where
├── README.md                      # says clearly: unofficial port
├── docs/
├── compatibility.json             # exact revisions and hooks number
├── tools/                         # stage, check, package (works on Windows)
└── mixins/dctrxds_jei/            # the bundle; the folder must match the mod id
    ├── horizon-mixin.toml
    ├── adapter.rs                 # module game::dctrxds_jei
    ├── frame.rs, draw.rs, ...     # one tiny helper per hook
    ├── data/                      # compiled-in lang and our own textures
    └── core/                      # [[crate]] dctrxds-jei-core
        ├── Cargo.toml
        └── src/
```

## Crate 1: core (pure logic)

| Module | Holds | Based on (JEI) |
|---|---|---|
| `ingredient` | `Stack`, `IngredientKey` (item + subtype), grouping key | `StackHelper`, `IIngredientHelper` |
| `search::token` | Tokenizer | `SearchTokenizer` |
| `search::prefix` | Prefixes and modes | `ElementPrefixParser`, `PrefixInfo` |
| `search::index` | Substring index | Behaviour of `LuceneSearchStorage`; not its code |
| `search::filter` | AND, OR, NOT evaluation | `IngredientFilter`, `ElementSearch` |
| `sort` | Stages, precomputed order | `IngredientSorter` |
| `recipe::model` | `Recipe`, `Category`, `Slot`, `Role` | API concepts |
| `recipe::index` | Role → key → recipes | `L/recipes/collect/*` |
| `recipe::lookup` | Focus and matching | `FocusGroup`, `RecipeManagerInternal` |
| `nav` | Current page, back and forward | `RecipeGuiLogic`, `NavigationHistory` |
| `layout::grid` | Grid with exclusion areas, pages, scroll | `IngredientGridLayout`, `GridScrollMath` |
| `layout::recipe` | Slot positions per category | Category `setRecipe` methods |
| `input` | Hit testing, press capture, key contexts | `GuiInputController`, `JeiKeyConflictContext` |
| `edit_box` | Search text, cursor, selection | Our own |
| `bookmarks` | List, order, JSON | `BookmarkList`, `BookmarkCodec` |
| `visibility` | Hidden ingredients | `IngredientVisibility` |
| `config` | Settings, parsing, defaults | Our own format |
| `transfer` | Item assignment (later) | `RecipeTransferUtil` |
| `app` | The `App` state tying everything together | `JeiGuiStarter` role |

**The whole contract of core:**

```rust
// Illustrative shape only. No code exists yet.
pub enum Input { Mouse { .. }, Scroll { .. }, Key { .. }, Char(char) /* ... */ }
pub enum Handled { Pass, Consumed }           // Consumed makes the hook cancel the event

pub enum DrawCmd { Fill { .. }, Item { .. }, Text { .. }, Tooltip { .. }, Sprite { .. } }
pub enum Action  { Give { .. }, PlaceRecipe { .. }, SendCommand { .. }, SaveFile { .. } }

impl App {
    pub fn new(tables: Tables, config: Config) -> Self;
    pub fn on_input(&mut self, screen: &ScreenInfo, input: Input) -> Handled;
    pub fn draw(&self, screen: &ScreenInfo) -> Vec<DrawCmd>;
    pub fn take_actions(&mut self) -> Vec<Action>;
}
```

The core never touches the disk, the clock or the game. Time is passed in as a value, and saving is an `Action`.

## Crate 2: adapter (Horizon)

Everything specific to Horizon, and as little as possible of it.

| Hook | Self | Adapter does |
|---|---|---|
| `frame` | `&mut` | Builds tables when a world opens, runs queued actions (give, place, send chat, save) |
| `screen_render_post` | `&` | Asks the core for `DrawCmd`s and paints them on the batch |
| `screen_mouse_button_pressed_pre`, `released_pre`, `dragged_pre`, `scrolled_pre` | `&` | Sends to core, sets `canceled` when consumed |
| `screen_key_pressed_pre`, `released_pre`, `char_typed_pre` | `&` | Same |
| `screen_opening`, `screen_closing` | `&` | Layout reset, search focus |
| `tooltip` | `&` | Adds JEI lines; may record the hovered stack **[ASSUMED route]** |
| `client_resources_reloaded`, `recipes_received`, `client_level_load` | mixed | Marks tables as stale |
| `mods`, `settings`, `settings_click` | mixed | Options > Mods row, keys, buttons |
| `register_configs`, `config_loading`, `config_reloading` | mixed | Config file |

**The `&self` problem.** Input and draw hooks cannot change the game, and a hook can call back into itself. So the state
lives in a `thread_local!` `RefCell<App>` (as the Recipes mod does; `runtime.rs` there), the core queues `Action`s, and
`frame` (with `&mut self`) runs them. Compiled in the probe **[RAN]**, not executed.

**Stay rules:**

- Must honour `native_on(ID)`: mixin code is not switched off for us, so it must check (`docs/native-mods.md`).
- No hardcoded user text: all goes through a translation table.
- A helper does one call to the adapter and nothing else.
- Any non-catalog call lives in one file, `adapter_internal.rs`, so it can be listed and removed.

## Layer 3 (optional): public API for other mods

Two steps, in the last phases:

1. **Data-driven categories:** a mod declares a category and a layout in JSON, and its recipes are custom-type recipes read
   by `native_recipes_of_type`. No Rust dependency between mods, so nothing breaks at compile time.
2. **Rust API:** an `[api]` package exposing `register_category(...)` with `[dependencies] dctrxds_jei = "^1"`. This format
   is still "draft" upstream. The dependency's package needs its own design, and JEI's `IModPlugin` will not map one to one
   because there is no runtime discovery.

## Testing

| Layer | How | Where |
|---|---|---|
| Core | `cargo test` on any OS, ported JEI tests included | `core/` |
| Bundle | `horizon-mixins plan` and `apply`, then `cargo check` | `tools/` |
| In game | The launcher builds it; drive it with `horizon-ctl --offscreen` on temporary data (like `native/recipes/proof.py`) | `tools/proof.py` |
| By hand | Your eyes, in a throwaway instance | Each phase's acceptance |

Never point these at your real saves: `AGENTS.md` asks for fresh temporary folders.

## Persistence

```text
<game>/config/dctrxds_jei/
├── client.toml        # settings; hot-reloaded via register_configs
├── bookmarks.json
├── history.json
└── hidden.json
```

Each JSON file has a `schema` number, for later migration. JEI's per-world bookmarks are an option (later), since the id of
the server or world would be part of the path.
