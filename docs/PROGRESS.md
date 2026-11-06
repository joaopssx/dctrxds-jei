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

## Phase 0: in progress

**Status:** code written; the Linux gate and the in-game test are still open. See the checklist.

### Results

| Check | Where | Result |
|---|---|---|
| `cargo fmt --all --check` | Windows | Passed |
| `rustfmt --check` on the adapter (outside the workspace) | Windows | Passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | Windows | Passed |
| `cargo test --workspace` | Windows | 26 passed |
| `horizon-mixins plan` and `apply` at the pins | Windows | `status: ok`, 8 injections, 1 module, 1 crate |
| `cargo check` of the game with the mod | Windows, MSVC, no default features | Passed |
| `cargo horizon-mod check` on the `.hnmod` | Windows | Passed; 16,484 bytes |
| `tools/horizon.py` stage, check, test, build | Linux | **Not run yet** |
| Launcher build | Windows | **Not run: needs the owner's trust decision** |
| In-game test | Windows | **Not run** |

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

None yet: the draw time and FPS are shown on screen for the in-game test.
