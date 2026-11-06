# Notice

DctrXD´s JEI is an unofficial port of Just Enough Items to Horizon.

## Just Enough Items

- Project: <https://github.com/mezz/JustEnoughItems>, by mezz
- Licence: MIT, in [LICENSES/JustEnoughItems-MIT.txt](LICENSES/JustEnoughItems-MIT.txt), and in the bundle as
  `mixins/dctrxds_jei/NOTICE-JustEnoughItems`
- Read at commit `fb7428472f019a0c13fb3fdc7fb5c4250ae869bc`, branch `26.3`

**Translated from JEI, with mezz's notice in the file's header:**

| Here | There |
|---|---|
| `core/src/search/token.rs` | `Gui/.../search/SearchTokenizer.java`, `Token.java` |
| `core/src/search/token_tests.rs` | `Gui/.../search/SearchTokenizerTest.java` (all 41 cases) |
| `core/src/search/filter.rs` | The evaluation in `Gui/.../ingredients/IngredientFilter.java` |
| `core/src/scroll_math.rs` | `Common/.../gui/GridScrollMath.java` and its test |
| `core/src/nav.rs` | `Gui/.../recipes/NavigationHistory.java` and its test |

**Written anew, following JEI's behaviour, with no code taken:** the search prefixes and their defaults, the sort stages, recipe
lookups by role, overlay placement, default keys, bookmarks and history.

**Not used:** JEI's textures, sounds and translation files.

## horizon-native-mod-template

- Project: <https://github.com/horizon-rust/horizon-native-mod-template>
- Licence: MIT, in [LICENSES/horizon-native-mod-template-MIT.txt](LICENSES/horizon-native-mod-template-MIT.txt)
- `tools/horizon.py` is adapted from it: the native test runs in the library and must match at least one test.
- `tools/package.py` is adapted from it, to ship crate folders and notice files and to name the hook points version.

## Horizon

The mod is built against Horizon's hook points and contains none of its code. Its source is applied only into a separate
generated copy, never committed here.

## Minecraft

This repository contains no code, assets or data from Minecraft, Mojang or Microsoft. Items, names and recipes are read at
run time from the player's own game. Test data is made up, and `assets/icon.png` is original.
