# Porting notes

Every difference from the original JEI: what changed, why, and what was left out. JEI reference: commit `fb7428472`, branch
`26.3`.

## Phase 0

Phase 0 is a proof of concept, not a port of JEI's behaviour. These differences are temporary.

| Topic | JEI | This mod now | Why | Until |
|---|---|---|---|---|
| Item list | Creative tabs, with variants, all pages | First items that fit, by registry order, no variants | Proves drawing only. `native_recipes` has ids, not variants | Phases 1 and 6 |
| Left click on an item | Shows recipes, or gives in cheat mode | Gives one item | Proves the give path | Phase 2 |
| Typing over the grid | No such feature | Keeps text in `probe.json` | Proves keyboard capture and files. Will be removed | Phase 1 |
| Diagnostic rows | None | Mod name, text, tooltip slot, FPS, draw time | Answers the phase's questions | Phase 1 |
| Toggle key | `Ctrl+O`, bound as a chord | Ctrl is required in code; the bound key is only `O` | `NativeMod.keys` holds a key name, not a chord | To revisit in Phase 5 |
| Interface text | Translated, 31 languages | English only, hardcoded | Temporary text | Phase 5 |
| Screens | Any screen with a handler, including chat | `inventory` and `creative` ids only | The ids this phase must prove. Each container reports `inventory` | Later phases |

## Differences expected to stay

| Topic | JEI | This mod | Why |
|---|---|---|---|
| Mod id | `jei` | `dctrxds_jei` | Horizon ids allow only `[a-z0-9_]`, and `jei` is already taken in the registry |
| Recipes view | A screen of its own | Planned as a layer over the open screen | `native_mod_screen_open` only opens from gameplay, not over an inventory |
| Search backend | Lucene | Planned own substring index | A bundle crate can only use crates in Horizon's `Cargo.lock` |
| Plugin discovery | Annotation or entrypoint at run time | None | Native mods are compiled in; no run-time loader exists |
