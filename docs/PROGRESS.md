# Progress

## Decisions made by the owner (2026-11-06)

- At first: only Phase 0, with Phase 1 after its acceptance. Replaced on the same day: "faça todas as fazes logo". All phases
  were written one after the other, **on a Phase 0 nobody has seen running**. A failure in the game test may mean rework.
- Mod id: `dctrxds_jei`. Folder and repository stay `dctrxds-jei`.
- The Linux gate runs in Ubuntu on WSL (`tools/wsl_gate.sh`). The in-game test runs through the launcher on Windows.

## Pinned revisions

| What | Revision | Why |
|---|---|---|
| Horizon | `66f3981f522b24c250b21523a70080965b940424` (Nightly 2026-11-05, API 411) | The one installed here, instead of the template's API 3 pin |
| horizon-mixins | `e8cd0aa38cffef49cb2e088ef716baa107148c02` | `MIXINS_REV` of the launcher at that Nightly |
| Rust | 1.95.0 | The template's. The launcher builds players' copies with its own Rust |

## Status: code written, **never run inside the game**

Every automated gate passes. No phase is accepted, because each one's acceptance includes the game, and the launcher build is
the owner's decision (`--trust-native-code`). The test to run is [MANUAL_TEST.md](MANUAL_TEST.md).

| Phase | Code | What is missing from its acceptance |
|---|---|---|
| 0 Proof | Written, then replaced by the real overlay | Launcher build; everything in the game |
| 1 List and search | Done | Game test. Ported: all of `SearchTokenizerTest` (41) and `GridScrollMathTest` (11); **not** `IngredientGridConfigTest` (60), `IngredientGridPageStateTest` (19), `IngredientListOverlayLayoutTest` (9): they test JEI's own config classes, so new tests cover the behaviour. Of `LimitedStringStorageTest` (4), only the behaviour that applies. Exclusion areas: only the screen's rectangle |
| 2 Recipes | Done | Game test. New tests stand in for `InternalRecipeManagerPluginTest` (16) |
| 3 Release | Prepared in [REGISTRY_SUBMISSION.md](REGISTRY_SUBMISSION.md), **not sent** | Game test; the owner submits |
| 4 Grid, cheat | Done, without the optional server half | Game test |
| 5 Bookmarks, settings | Done | Game test. **Not done:** showing key conflicts (the game's Options > Mods page owns the keys) |
| 6 Variants, categories | **Partial** | Variants only when a recipe names them; tag pages done. Fuel, compost, anvil, grindstone, information pages, tab search and full tooltips wait for [UPSTREAM_PROPOSALS.md](UPSTREAM_PROPOSALS.md) |
| 7 API | **Partial** | Data-driven categories done, with a sample. Rust API **not done**: see [PORTING_NOTES.md](PORTING_NOTES.md) |

## Gate results

| Check | Where | Result |
|---|---|---|
| `cargo fmt --all --check`, `rustfmt --check` on the adapter | Ubuntu on WSL | Passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | Ubuntu on WSL | Passed |
| `cargo test --workspace` | Ubuntu on WSL | 185 passed |
| `tools/horizon.py stage`, `check`, `build` | Ubuntu on WSL, Rust 1.95.0, no default features | Passed; binary built, never run |
| `tools/horizon.py test` | Ubuntu on WSL | 3 passed (`game::dctrxds_jei::tests`), 1016 filtered out |
| `cargo horizon-mod check` on the `.hnmod` | Windows | Passed; 74,025 bytes |
| `cargo horizon-mod build`, `check` on `examples/sample_category` | Windows | Passed. Its loading in the game isn't checked |
| Launcher build, in-game test | Windows | **Not run** |

## `unsafe`

None.

## Measurements

`cargo run --release --example bench -p dctrxds-jei-core`, on this Windows machine, with made-up items. "Search" is the last
key's whole cost in the core: search, sort and building the drawing commands. A frame at 60 FPS has 16.7 ms.

| Items, recipes | Build at world open | Search `iron` | Search `o` | Search `iron gear -sword` | Lookup | Frame in the core |
|---|---|---|---|---|---|---|
| 2,000, 1,500 | 8.8 ms | 31 us | 73 us | 69 us | 11 us | 4.8 us |
| 20,000, 20,000 | 156 ms | 285 us | 880 us | 881 us | 12 us | 4.5 us |
| 50,000, 50,000 | 412 ms | 712 us | 2.3 ms | 2.3 ms | 12 us | 4.6 us |
| 100,000, 100,000 | 1.13 s | 1.4 ms | 4.8 ms | 4.5 ms | 17 us | 6.7 us |

The build at world open runs on the frame hook, so a big pack gets one hitch of that size when a world opens. **Not measured:**
drawing the icons in the game, which is what the on-screen `show_stats` is for.

First-run compile times on Ubuntu on WSL (32 threads, no default features), Phase 0: stage 58 s, `cargo check` 53 s, `build`
170 s. These are not the launcher's times.

## History of a false pass

The first Linux gate reported green with `0 passed` for the native test: at this Horizon, `src/main.rs` does not hold the `game`
module, so the template's filter matched nothing. `tools/horizon.py` now tests the library and fails when no test matches.

## Risks to look at first in the game

1. The `tooltip` hook as the slot under the mouse (R and U on container slots).
2. A letter bound to R, U or A pressed over a slot while typing in an anvil's name box: the mod takes it, as it can't see that box's focus.
3. Overlap with effects and the recipe book, as `rect` is the only exclusion.
4. Draw order: the hook runs after the screen's tooltips, so a panel could cover one.
5. A stack with components drawn from `command_components`, which is public library code but not a promised helper.
