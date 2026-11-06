# Decision: which kind of mod

## Recommendation: (B) a native `.hnmod` with horizon-mixins

Build a native mod using **named hooks** wherever possible, with all game access kept in one thin adapter module.

If a gap truly blocks a feature, propose a small helper upstream for that one feature (a small part of option D). This
is an add-on, never a precondition: the mod's core must ship without waiting for any PR.

## Why, in simple terms

A viewer has to draw over screens, take clicks and keys, read recipes and save files. Only native code can do all of
that. The evidence:

- All 8 critical needs have a native route, and none is missing outright ([CAPABILITY_MATRIX.md](CAPABILITY_MATRIX.md)).
- The riskiest piece compiled: a probe using the overlay, mouse, keyboard, recipe, give and config APIs passed
  `cargo check` on current Horizon on your Windows machine. A deliberate typo failed, so the check was real
  ([ANALYSIS.md](ANALYSIS.md) section 2.1). **It has not yet run inside a game.**
- Two mods already work this way (official Recipes and "JEI for Horizon"), so the path is proven by others **[READ]**.

## Alternatives rejected

| Option | Why not |
|---|---|
| **(A) Pure `.hmod`** | Impossible. `.hmod` has no code: it cannot draw, read input or react to anything |
| **(C) Mixed** | Not needed now. It would only add textures, and a native mod can compile its own files in (`native_data!`, `native_mod_image`) or use vanilla sprites. Mixed `.hmod` publishing is also not open yet: "the registry still publishes native mods as `.hnmod`" (`docs/api/mod-files.md`). Revisit if we ship many textures |
| **(D) Upstream first** | Too slow and risky as a base. Your mod would wait on other people's review, and the features are mostly reachable now. Useful as a targeted complement |

### Also considered: use the game's item panel (the Recipes route)

It is the fastest route, but it cannot become JEI:

- Right side only, and not on the creative screen
- No room for bookmarks
- Only 5 fixed recipe layouts
- No modifier keys in its events

So we draw our own overlay. Recipes' code is a useful reference for table reading and event handling (MIT).

## The main trade-off: pin or no pin

A mod can declare either of these:

| | `"hooks": N` | Exact `horizon_revision` pin |
|---|---|---|
| What it means | Builds on any Horizon with API N or later | Builds on one exact commit |
| Plays with other mods | Well | The launcher may ask the player to "Build anyway" when pins differ |
| Can use non-catalog code | Should not | Yes |
| Maintenance | Low | A new version whenever you want a newer Horizon |

**Recommendation:** ship as `"hooks": N`, using only catalog hooks and helpers plus Horizon's public library types. That
leaves some features partial until a helper exists (creative tabs, full tooltips, fuel and compost). If you prefer those
features sooner, the alternative is a pin. See [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) Q4.

**[ASSUMED]** Public types like `ItemStack` used from a helper are not formally in the catalog promise, so a Horizon
refactor could still break a `hooks` build. The registry's daily check (`broken_on`) would flag this.

## Risks of the chosen path

| Risk | Plain meaning | Mitigation |
|---|---|---|
| Native mods run with the player's full permissions | No sandbox: a bug or bad code can touch the player's files. Reviewers read each version's source | Keep to our config folder, no hidden network use, say all this in the README. Sign releases |
| Horizon changes fast | 923 commits separate the installed Nightly from `main`. Non-hook code can vanish | Catalog hooks only, and one adapter module. Compile against each new Nightly |
| The template only validates on Linux | `tools/horizon.py` imports `fcntl`, which fails on Windows **[RAN]**. Apply needs a fresh folder each time on Windows **[RAN]** | Own small Windows-friendly script, or WSL. See Q3. **The launcher's real Windows build is not yet tested by us** |
| Pinned revisions go stale | The template's pin is API 3 | Do not copy its pin. Use the commit of a Nightly and the launcher's mixins revision |
| Different toolchains | Your Rust is 1.95 (MSVC); the launcher's is 1.99.1 (`windows-gnu`) | The player's path is the launcher, so Phase 0 builds through it |
| Long first build | The launcher's first build "can take 20 to 30 minutes" | Use the `dev` native profile during development |
| Mod collisions | Other overlays can draw in the same space. `recipes` and `jei` would both be active if installed | Document it. Consider detecting them through `native_mods` **[ASSUMED possible]** |
| Performance | Hundreds of item icons per frame, plus search on each key | Measure in Phase 1; cache search results |
| Trait mods are a "draft" | The public API may change under us | Public API comes last, and starts data-driven |
