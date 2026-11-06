# Progress

## Decisions made by the owner (2026-11-06)

- Only Phase 0 now; Phase 1 starts after its acceptance.
- Mod id: `dctrxds_jei`. Folder and repository stay `dctrxds-jei`.
- The Linux gate runs in Ubuntu on WSL (`tools/wsl_gate.sh`). The in-game test runs through the launcher on Windows.

## Pinned revisions

| What | Revision | Why |
|---|---|---|
| Horizon | `66f3981f522b24c250b21523a70080965b940424` (Nightly 2026-11-05, API 411) | The one installed here, instead of the template's API 3 pin |
| horizon-mixins | `e8cd0aa38cffef49cb2e088ef716baa107148c02` | `MIXINS_REV` of the launcher at that Nightly |
| Rust | 1.95.0 | The template's. The launcher builds players' copies with its own Rust |

## Phase 0: gates passed, waiting for the in-game test

**Status:** code written and every automated gate passed. The owner's launcher build and in-game test are still open, so the
phase is **not accepted yet**.

### Results

| Check | Where | Result |
|---|---|---|
| `cargo fmt --all --check` | Ubuntu on WSL | Passed |
| `rustfmt --check` on the adapter (outside the workspace) | Ubuntu on WSL | Passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | Ubuntu on WSL | Passed |
| `cargo test --workspace` | Ubuntu on WSL | 26 passed |
| `tools/horizon.py stage` | Ubuntu on WSL, Rust 1.95.0 | Passed |
| `tools/horizon.py check` | Ubuntu on WSL | Passed |
| `tools/horizon.py test` | Ubuntu on WSL | 1 passed (`game::dctrxds_jei::tests`), 1016 filtered out |
| `tools/horizon.py build` | Ubuntu on WSL, no default features | Passed; binary built, never run |
| `cargo horizon-mod check` on the `.hnmod` | Windows | Passed; 16,484 bytes; SHA-256 `73166cc5...` |
| Clippy on the game with the mod | Windows, not `-D` | No warning in the mod's code; Horizon has warnings of its own |
| Launcher build | Windows | **Not run: needs the owner's trust decision** |
| In-game test | Windows | **Not run** |

The first gate run reported green with `0 passed` for the native test: at this Horizon, `src/main.rs` does not hold the `game`
module, so the template's filter matched nothing. `tools/horizon.py` now tests the library and fails when no test matches.

### Acceptance (from the roadmap)

- [ ] `horizon-launcher native-build` succeeds on Windows
- [ ] The grid appears on the survival inventory, a chest and the creative screen
- [ ] A click on the grid does not reach the screen below; a click elsewhere works normally
- [ ] Creative click gives the item; survival refuses without a crash
- [ ] The saved text is back after restart
- [ ] `´` renders, or we switch to `'`
- [ ] Hovering a chest slot: does the `tooltip` hook tell us the stack?
- [ ] Build time noted; frame rate compared

### `unsafe`

None.

### Measurements

Compile times on Ubuntu on WSL (32 threads, 15 GB), first run, no default features. These are not the launcher's times.

| Step | Seconds |
|---|---|
| Stage (export, plan, apply) | 58 |
| `cargo check` | 53 |
| `cargo build --profile fast`, after check and test | 170 |

The overlay's draw time and FPS are not measured yet: they show on screen during the in-game test.
