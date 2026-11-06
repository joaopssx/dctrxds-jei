# Open questions

## Answered by the owner

Q1 (new mod), Q3 (Ubuntu on WSL for the Linux gate, launcher on Windows for the game), Q4 (stable hooks, as in the approved plan),
Q6's id (`dctrxds_jei`). Taken from the approved plan's recommendation: Q5 (own simple art, no JEI textures), Q7 (publish early,
still your call) and Q9 (`/give` only after an explicit toggle). Still open below: Q2, Q8, the GitHub branches, and the unknowns.

## Decisions for you

Each has my recommendation. Q1 should be answered first, as it can change the project's direction.

### Q1. Should this be a new mod, given that two already exist?

Found in the registry: `jei` ("JEI for Horizon" by enderliker, MIT, 0.1.0, on hooks 2, no updates since its repo's last push
on 2026-10-30) and the official `recipes`.

Its own parity doc lists "needs from Horizon" that have since been added (screen drawing, item drawing, tooltips, and more),
so that gap is likely smaller than the doc says **[READ, not tested]**.

| Option | For | Against |
|---|---|---|
| **A. New mod, as planned (recommended)** | You learn the most and own the result. A true overlay would go beyond both | Duplicates effort; players have similar mods to choose from |
| B. Contribute to `horizon-jei` | Faster for players; shared work | It isn't yours under your name; depends on its author; panel-based |
| C. Fork it under your name (MIT allows this) | Quick start | Its design rests on the panel we want to leave |

Nothing in `MOD-POLICY.md` forbids a similar mod **[READ]**. Reviewers may still ask why.

### Q2. Do you want to tell mezz?

MIT requires no permission. A short courtesy message before publishing, especially with "JEI" in the name, avoids
surprises. **Recommended, and it must come from you.** It may also be wise to think about the name, e.g. if asked to change it.

### Q3. How will you develop: Windows or WSL?

| Option | For | Against |
|---|---|---|
| **Windows, launcher plus our own small script (recommended)** | Same path as players on your PC. `plan`, `apply` and `cargo check` work **[RAN]** | Template scripts unusable. Each `apply` needs a new folder. The launcher's build is untested by us |
| WSL with Ubuntu | Template and CI-like conditions | Needs install and disk; running the game there needs a GPU setup; further from your players |

### Q4. How strictly should we stay on stable hooks?

| Option | Effect |
|---|---|
| **Stable only, ask upstream for helpers (recommended)** | Plays well with other mods. Variants, full tooltips and some categories wait for PRs that may be slow or refused |
| Pin to one commit, use private code | Features sooner. More frequent breakage and "Build anyway" prompts for players |

A mix may be possible later.

### Q5. Whose textures?

**Recommended: vanilla sprites from the player's game, plus our own simple art.** JEI's textures look fine under MIT, but
I could not determine if any derive from Mojang's art. If you want to reuse them, look at them and decide.

### Q6. Confirm the names

- Mod id: **`dctrxds_jei`**, since the hyphen is invalid.
- Display name: `DctrXD´s JEI`, tested in Phase 0, with `'` as fallback.
- GitHub repository name: `dctrxds-jei`?
- Should this folder become a git repo now? It isn't one yet.

### Q7. How much of the plan do you want?

Is Phase 3 (publishing after recipes) the right first release? The alternative is waiting for bookmarks and grid-filling.
**Recommended: publish early.** Do you want Phases 6 and 7 at all? They hold the most uncertainty.

### Q8. Which test instance?

Phase 0 needs you to run the launcher build, with the trust flag, on a **separate instance** so your `horizon` instance and
saves are not affected. OK? Do you have time for the first build ("20 to 30 minutes")?

### Q9. Who should be able to cheat on servers?

Off by default, as JEI has it. Should the `/give` route exist at all? It sends commands in the player's name when asked.
**Recommended: yes, but only after an explicit toggle.**

## What I could not determine

| Unknown | Where I looked | How to resolve |
|---|---|---|
| Does the overlay work in a running game? | The probe compiled only | Phase 0 |
| Does the launcher's `windows-gnu` build succeed with our bundle? | Docs only | Phase 0 |
| Does U+00B4 render? | Font code exists; not run | Phase 0 |
| Does `tooltip` reveal the hovered slot's stack reliably, including on empty-tooltip cases? | Doc comment only | Phase 0 |
| Do draws in `screen_render_post` land over tooltips acceptably? It runs after them, per its doc comment | Doc comment | Phase 0 |
| Does `rect` move with the recipe book? Does it leave room for effects? | `rect` doc only | Phase 1 |
| Frame cost of hundreds of item icons | Nothing measured | Phase 1 |
| Can `client_packet` track inventory changes well enough? | Hook doc | When needed |
| Is there a helper for the game's language? | Not in the catalog's helper list; full source not searched | Search before Phase 5 |
| Is `creative_screen.tabs` filled before opening the creative screen? | `creative.rs`, `game.rs` partly | Before Phase 6 |
| Do temporary env vars isolate data on Windows? | `AGENTS.md` written with Linux in mind | Phase 0 |
| Are JEI textures free of Mojang-derived art? | Search of README and CONTRIBUTING only; art not inspected | Q5 |
| What do `MMCQ.java` and `ColorThief.java` licences say? | Only found that headers exist | Only if colour search is wanted |
| Will reviewers accept a second JEI-like mod? | Policy text | Ask on Discord, or at submission |
| Open upstream issues or PRs for helpers we want | `gh` is not logged in, so no search was made | Run `! gh auth login`, then search before Phase 6 |
| Full reading of `docs/native-mods.md`, `docs/native-client.md`, `ROADMAP.md` | Read in part; see [ANALYSIS.md](ANALYSIS.md) | Read further when a phase needs it |
