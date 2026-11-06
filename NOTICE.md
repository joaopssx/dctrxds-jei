# Notice

DctrXD´s JEI is an unofficial port of Just Enough Items to Horizon.

## Just Enough Items

- Project: <https://github.com/mezz/JustEnoughItems>, by mezz
- Licence: MIT, in [LICENSES/JustEnoughItems-MIT.txt](LICENSES/JustEnoughItems-MIT.txt)
- Read at commit `fb7428472f019a0c13fb3fdc7fb5c4250ae869bc`, branch `26.3`

**What is derived so far: no code.** Phase 0 holds no code, text or texture translated from JEI. The overlay's position beside
the screen and the Ctrl+O toggle follow JEI's behaviour, written anew. Each later file translated from JEI will carry mezz's
copyright notice in its header and will be listed here.

## horizon-native-mod-template

- Project: <https://github.com/horizon-rust/horizon-native-mod-template>
- Licence: MIT, in [LICENSES/horizon-native-mod-template-MIT.txt](LICENSES/horizon-native-mod-template-MIT.txt)
- `tools/horizon.py` is copied from it, with the name of the native test filter changed.
- `tools/package.py` is adapted from it, to ship crate folders and notice files and to name the hook points version.

## Horizon

The mod is built against Horizon's hook points and contains none of its code. Its source is applied only into a separate
generated copy, never committed here.

## Minecraft

This repository contains no code, assets or data from Minecraft, Mojang or Microsoft. Items, names and recipes are read at
run time from the player's own game.
