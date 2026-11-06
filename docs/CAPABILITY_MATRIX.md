# Capability matrix

JEI features against what Horizon offers at `41f822b` (API 411).

- **`.hmod` column:** `.hmod` has no code (see [ANALYSIS.md](ANALYSIS.md)), so it can only provide data or textures.
  "None" means it cannot help at all.
- **Status:** OK = the stable catalog is enough. PARCIAL = works with limits, or needs non-catalog code. FALTA = no way today.
- **Evidence:** everything is **[READ]** unless marked. **[RAN]** means it compiled in the scratch probe (it was still
  never run in a game). **[ASSUMED]** is an untested idea.

**Non-catalog** means game code that exists and a mod can reach, but that is not promised stable. Using it forces an
exact Horizon pin or an upstream helper. See [DECISION.md](DECISION.md).

## The 8 critical needs

| # | Need | What JEI uses | `.hmod` | `.hnmod` | Status | Proposed solution |
|---|---|---|---|---|---|---|
| 1 | Overlay over inventory and container screens | `IngredientListOverlay`, `GuiEventHandler`, screen events | None | `native_screen_render_post` with `HudBatch` and `NativeScreenView.rect` **[RAN]** | **OK** | Draw our own overlay. Works on `inventory` and `creative` ids. Every container reports `inventory`; tell them apart by `menu` and `slots` |
| 2 | Registries: items, tags, creative tabs, recipes, components | MC registries, `ItemStackListFactory`, `RecipeMap` | None | Items, tags and recipes through `native_recipes` **[RAN]**. Components are a string per stack. **Creative tabs: no catalog helper** (`creative_screen` is private) | **PARCIAL** | Phase 1 uses item ids and recipe stacks. Variants like potions need the tabs: a small upstream helper, or a pin |
| 3 | Item icons and tooltips in our UI | `IIngredientRenderer`, `GuiGraphicsExtractor` | None | `native_item` **[RAN]**, `native_item_stack` (API 313), `native_gui_tooltip` **[RAN]**, `native_text`, `native_fill` **[RAN]**. **Full vanilla tooltip of a stack: non-catalog** (`stack_tooltip`) | **PARCIAL** | Icons OK. Start with name and mod lines; ask upstream for a tooltip-lines helper |
| 4 | Input: mouse, scroll, keyboard, configurable keys | `UserInput`, `GuiInputController`, key mappings | None | `screen_mouse_*_pre` **[RAN pressed, scrolled]**, `screen_key_*_pre` **[RAN pressed]**, `screen_char_typed_pre` **[RAN]**, `NativeMod.keys`, `native_mod_key`. Modifier bits provided | **OK** | Cancel what we use. The screen hooks do not fire for gamepad clicks |
| 5 | Persist config and bookmarks | MezzConfig, `BookmarkJsonConfig`, files | None. And no code to do it | `native_config_dir` **[RAN]**, `native_register_configs`, `native_setting` **[RAN]**. Native code has normal file access | **OK** | `config/dctrxds_jei/*.json`. Bookmarks use `std::fs` with atomic-style writes |
| 6 | Give items; move to the grid | `PacketGiveItemStack`, `RecipeTransferUtil`, server handler | None | See the split below | **PARCIAL** | Four-tier approach, below |
| 7 | Events: open/close screen, inventory, reload | Screen events, recipe sync, reload listener | None | `screen_opening`, `screen_init_post`, `screen_closing`, `client_resources_reloaded`, `recipes_received`, `client_level_load`, `client_disconnecting`, `client_packet` (sees inventory packets) | **OK** | `client_packet` for inventory changes **[ASSUMED enough]** |
| 8 | API so other mods register categories | `IModPlugin`, `A/registration/*`, loader discovery | **Possibly data only**: recipe JSON of a custom type, read with `native_recipes_of_type` | `[api]` trait mods with `[dependencies]`; still marked "draft". Both sides need an `[api]` package | **PARCIAL** | Later phase: data-driven categories first, Rust API after. Plugin-style discovery does not exist |

### Need 6, split up

| Case | Way | Limit | Status |
|---|---|---|---|
| Fill the crafting grid from a known recipe | `native_place_recipe(book_id, all)` | Inventory or crafting table only; must be in the recipe book; special recipes have `book_id` -1 | OK |
| Creative: give a plain item | `native_give` **[RAN]** | At most 99 and **no components**, so no potion variants | OK |
| Creative with components | `mod_give` (non-catalog) | Needs a pin, or upstream change | PARCIAL |
| Survival cheat, op on any server | `native_send_chat("/give @s ...")` | Needs permission; components use `/give` syntax. **JEI itself has no such fallback** **[ASSUMED it works]** | PARCIAL |
| Survival cheat, Horizon server with the mod | `native_net_send_to_server` + `native_give_stack`, checking `permission` | Server must run the mod | PARCIAL |
| Fill furnace, smoker and others | Server half with `native_net_*`, or hand-made click packets | `native_send_packet` leaves correctness to the mod. Risky on vanilla servers | FALTA at first |
| Delete an item with the cursor | Server half only | Same as JEI: server must have it | Later |

## Other JEI features

| Feature | JEI uses | `.hmod` | `.hnmod` | Status | Solution |
|---|---|---|---|---|---|
| Paged grid, navigation | `IngredientGridWithNavigation` | None | Our drawing | OK | Port layout math |
| Scrolling mode | `GridScrollMath` | None | `screen_mouse_scrolled_pre` | OK | Port with tests |
| Exclusion areas | `getGuiExtraAreas`, effects, recipe book | None | `rect` only. **Recipe book and effect areas are not exposed** | PARCIAL | `rect` may already follow the recipe book **[ASSUMED]**; test in Phase 1 |
| Search box | `GuiTextFieldFilter` | None | `screen_char_typed_pre`, keys, own drawing | OK | Own edit box in the core, including cursor and selection |
| Search words, quotes, `-`, `\|` | `SearchTokenizer` | None | Pure logic | OK | Port and its 41 tests |
| `@mod` | Mod index | None | Item namespace | PARCIAL | Namespace, not the mod's display name |
| `#tag` | Tag index | None | `NativeItem.tags` | OK | |
| `$tooltip` | `ListElementInfo` | None | Non-catalog `stack_tooltip` | PARCIAL | Index names until a helper exists |
| `%creative tab` | Tab index | None | Non-catalog | PARCIAL | With need 2 |
| `^colour` | MMCQ on textures | None | No pixel access found | FALTA | Off by default in JEI; skip |
| `&identifier` | Identifier index | None | `NativeItem.id` | OK | |
| Sorting | `IngredientSorter` | None | Pure logic | PARCIAL | CREATIVE_MENU needs tabs; registry order until then |
| Recipes view, tabs, history | `RecipesGui`, `RecipeGuiLogic` | None | Our layer over the open screen | OK | Own layer; `mod_screen_open` cannot open over a screen |
| Crafting, furnace family, stonecutter, smithing, brewing | Categories | None | `native_recipes` | OK | |
| Fuel, compost | Computed | None | Non-catalog `Items` fields | PARCIAL | Upstream helper, or a pin |
| Anvil, grindstone | Fake menus | None | No helper found. A rough version from item tags and stack components may be possible **[ASSUMED]** | PARCIAL | Later, from repair data |
| Mod's custom recipe types | `IRecipeCategory` | Data | `native_recipes_of_type`, `native_datapack_sync` | PARCIAL | With need 8 |
| Datapack recipes on a server | Server sync | None | `native_datapack_sync` with a server half. **Without it, the client's own recipes are shown** | PARCIAL | Same limit as JEI without a server side |
| Catalysts | `registerRecipeCatalysts` | None | Pure data | OK | |
| Information pages | `addIngredientInfo` | Could carry text | Own data | OK | |
| Tag categories | `JeiInternalPlugin` | None | `NativeItem.tags` | OK | |
| R and U over screen slots | Hovered slot via platform helper | None | **No catalog helper.** Ideas: remember the stack seen in `tooltip` this frame **[ASSUMED]**; panel `Key` event only when the panel is on; `slot_at` is non-catalog | PARCIAL | Test the tooltip route in Phase 0; upstream helper is the clean fix |
| Bookmarks: ingredient and recipe | `BookmarkOverlay`, `BookmarkList` | None | Our drawing and files | OK | |
| Lookup history | `LookupHistory` | None | Pure logic | OK | |
| Hide ingredients, edit mode | `IngredientVisibility`, `blacklist.json` | None | Pure logic | OK | |
| Ghost-ingredient drag | `G/ghost` | None | Draw on cursor; dragging hook | PARCIAL | Late, low priority |
| Chat item links | `JeiChatItemLinks` | None | `native_send_chat`, `client_chat_received` | PARCIAL | Optional |
| Translation | `lang/*.json` (31 files) | Could carry them | Own table in `data/`, compiled in | PARCIAL | **The game's current language is not exposed as a helper** **[not found]**; own setting. `pt_br.json` exists in JEI |
| Config screen | `gui/config/*` | None | `native_settings` label buttons, or our own layer | PARCIAL | Buttons first, then a layer |
| Fluids | Fluid types | None | `native_item_fluid_tank` only for tanks; no list | FALTA | Out of scope for now |
| Plugin discovery | Annotation or entrypoint | None | No loader; only compile-time dependencies | FALTA | Not portable. A different API instead |
| Dynamic recipe plugins | `IRecipeManagerPlugin` | None | Same | FALTA | Not portable as is |

## Totals for the 8 needs

**OK:** 1, 4, 5, 7. **PARCIAL:** 2, 3, 6, 8. **FALTA:** none.

The partial ones share one cause: the game has the data, but no stable helper exposes it. That is what drives
[DECISION.md](DECISION.md).

## Search locations for "not found"

- Creative tabs, slot under the cursor, and item tooltip helpers: names searched in `native_hooks.rs`, `native-hooks.json`
  and `docs/native-mods.md`.
- Game language helper: **not found** in the `native-hooks.json` helper list. The search was not exhaustive across the full
  `native_hooks.rs`, so confirm before relying on this.
