# Porting notes

Every difference from the original JEI: what changed, why, and what was left out. JEI reference: commit `fb7428472`, branch
`26.3`.

## Behaviour that differs

| Topic | JEI | This mod | Why |
|---|---|---|---|
| Mod id | `jei` | `dctrxds_jei` | Horizon ids allow only `[a-z0-9_]`, and `jei` is already taken in the registry |
| Recipes view | A screen of its own | A layer over the open screen; takes all input while open | `native_mod_screen_open` only opens from gameplay, not over an inventory |
| Search storage | Lucene wildcard queries | Own substring index (`search/index.rs`) | A bundle crate can only use crates in Horizon's `Cargo.lock` |
| `@mod` | Mod names, ids and aliases | The item's namespace | No list of mod names for items is exposed |
| `$tooltip` | All tooltip lines | The name only | Proposal 2 in [UPSTREAM_PROPOSALS.md](UPSTREAM_PROPOSALS.md) |
| `%tab`, `^colour` | Creative tabs; colours by MMCQ | Accepted, find nothing | Proposal 1; no texture pixels available |
| List content | Creative tabs, so every variant | Item table, plus variants named by recipes (potions, tipped arrows) | Proposal 1. Variants no recipe names don't appear (enchanted books, most damaged items) |
| Variant names | The game's item name | `Potion (swiftness)`, read from its components | No helper for the name of a stack |
| Sort stages | 7 | 4: mod name, alphabetical, creative menu, tag | Armor and durability have no data; there is only one ingredient type |
| Tag sort | Tag with the most items | First tag by name | Count not computed |
| Creative menu order | Creative tab position | The item table's order | Proposal 1 |
| Tooltip on our slots | Full vanilla tooltip, plus mod name | Name and namespace | Proposal 2 |
| R and U over container slots | Hovered slot from the platform | The stack the game's `tooltip` hook named in the last frame | Proposal 3. **Not checked in the game yet** |
| R and U on the creative screen's slots | Works | Only over our panels | Its search box can't be seen as focused or not, so R there must stay a letter (proposal 9) |
| Transfer `+` | Own handlers, needs JEI on the server; any container | The game's recipe book placement: inventory or crafting table, known recipes only | No server part required. Works on vanilla servers, but less capable |
| Transfer to furnace and others | Yes, with JEI on the server | No | Would need hand-written click packets; left out on purpose |
| Cheat on a server | JEI's packet, if the server has JEI | `native_give` in creative; `/give` when the player turned that on | JEI has no `/give` fallback. Works on servers without the mod, by the server's permissions |
| Cheat with components, creative | Yes | Needs the `/give` setting | `native_give` takes no components (proposal 6) |
| Delete items in cheat mode | Yes, with JEI on the server | No | Needs a server half |
| Hover highlight, ghost drag, pinned tooltips | Yes | No | Cut for scope |
| Pausing slot cycling (Shift) | Yes | No | Cut for scope |
| Search completion and syntax colours | Yes | No | Cut for scope |
| Overlay on chat screen | Yes | Only inventory and creative ids | Cut for scope |
| Key chords | Any key with modifiers, bound as a chord | Ctrl is required in code for some keys; the bound key is one key | `NativeMod.keys` holds a key name, not a chord |
| Mouse bindings | Click actions can be rebound | Fixed | Options > Mods only binds keys |
| Grid config | Rows, columns, alignment, drag to resize | Fills the free space, up to 9 columns | Cut for scope |
| Exclusion areas | Effects, recipe book, toasts | Only the screen's own rectangle | Proposal 8. **Overlap with effects is possible** |
| Language | Follows the game; 31 languages | Follows `lang:` in `options.txt`, or a setting; English and Portuguese | JEI's other translations are for its own keys |
| Interface art | JEI's textures | Plain rectangles and text | Open question Q5: no Mojang-derived risk |
| Config files | `jei-client.ini` and others | `config/dctrxds_jei-client.toml`; `config/dctrxds_jei/*.json` | No MezzConfig |
| Bookmarks and history per world | Per world or server | One set per instance | Simplification |

## Kept on purpose, though surprising

- `zzz -iron`, words finding nothing with an exclusion, starts over from every ingredient. It looks like a bug, but
  `IngredientFilter.getSearchResults` does exactly that, and a port should give the same results. There is a test for it.
- `%` and `^` are ordinary characters while their modes are disabled, as in JEI.

## Left out: not possible today

| Feature | Why | Alternative |
|---|---|---|
| Fuel, compost, anvil, grindstone categories | No stable helper for this data | Proposals 4 and 5 |
| Fluids as ingredients | The game's tables list only items | None for now |
| Colour search | No texture pixels through a helper | None |
| Plugins found at run time (`IModPlugin`) | Native mods are compiled in; no run-time loader exists | Data-driven categories |
| Rust API for other mods | The `[api]` format is "draft", and it depends on the game's library while our state lives in a module of the game itself. Exporting it would need a redesign | Data-driven categories, below |
| Dynamic recipes (`IRecipeManagerPlugin`) | Same as plugins | Declare them as a recipe type |
| Categories on a remote server | The folder is read by the integrated server only | Single player and LAN host work; **not checked in the game yet** |

## Data-driven categories

JEI's plugin API has no equivalent here, so another mod declares a category with JSON and no Rust. See
[examples/sample_category](../examples/sample_category/README.md).
