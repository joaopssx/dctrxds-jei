# Upstream proposals

Small helpers that would close the gaps below. **None has been opened.** Under `CONTRIBUTING.md` each needs an issue claimed
with `/claim`, a PR of about 500 lines at most, `tools/pr_ready.py`, and a statement that it was opened by an agent if one
does it. The code they would expose already exists in Horizon; the mod just has no stable way to reach it.

| # | Proposed helper | Exists today as (not in the catalog) | What it unlocks here |
|---|---|---|---|
| 1 | `native_creative_tabs() -> Vec<NativeCreativeTab { id, title, stacks }>` | `creative_screen.tabs`, `creative.rs` | Every variant in the list, `%tab` search, JEI's creative menu order |
| 2 | `native_stack_tooltip(stack) -> Vec<Text>` | `stack_tooltip`, `stations.rs` | Real tooltip on our slots, and `$tooltip` search on real lines |
| 3 | `native_slot_under_cursor() -> Option<ItemStack>` | `slot_at`, `slot_view`, `game.rs` | Clean R and U over slots, replacing the `tooltip` hook route, and on the creative screen |
| 4 | `native_item_fuel(item) -> Option<i32>`, `native_item_compost(item) -> Option<f32>` | `Items.fuel`, `providers.rs`; `compostable`, `composter.rs` | Fuel and compost categories |
| 5 | Repair and enchanting facts for an item | `repairable` component, anvil code | Anvil and grindstone categories |
| 6 | `native_give_stack_client(stack)` with components, in creative | `mod_give`, `native_recipes.rs` | Creative gifts of potions without `/give` |
| 7 | `native_game_language() -> String` | `settings` | No need to read `options.txt` |
| 8 | `native_screen_exclusions() -> Vec<[f32; 4]>` | The recipe book's and effects' rectangles in the game's drawing | JEI's exclusion areas |
| 9 | Search box focus on the creative screen | `Creative.search` | R and U on the creative screen's slots |

Not proposed: `recipes` in the trait mods' `Host`, until a Rust API for other mods is really wanted.

## Why the mod does not reach for the private code

A helper from the catalog is a stable promise; a private function can vanish in any commit. The owner approved the plan that
stays on stable hooks ([DECISION.md](DECISION.md)), and using private code would force an exact Horizon pin for every player.
