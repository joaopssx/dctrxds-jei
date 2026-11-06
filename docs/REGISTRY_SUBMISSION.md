# Registry submission (prepared, not sent)

**Do not submit before the in-game test passes.** Reviewers check that a mod builds, and you personally promise what
`MOD-POLICY.md` asks. You send it, at <https://playhorizon.dev/mods/submit>; re-read `docs/api/mod-index.md` first, since the
registry's rules changed as recently as 2026-11-05.

## Checklist

- [x] Id `dctrxds_jei`: valid and not in use (checked against `mods/` at `d2de4c1`)
- [x] Licence MIT; mezz's notice kept in the bundle (`NOTICE-JustEnoughItems`)
- [x] Original credited and linked (text below)
- [x] No Minecraft content in the repository
- [x] `cargo horizon-mod check` passes on the `.hnmod`
- [x] Icon: `assets/icon.png`, 128 x 128, original
- [ ] Mod seen working in the game
- [ ] Public source at the tagged commit: push it and tag it
- [ ] Minisign key made by you and kept outside the repository (optional, recommended)
- [ ] Courtesy message to mezz (open question Q2, your call)

## Page

- **Name:** DctrXD´s JEI
- **Summary:** Unofficial port of Just Enough Items: items, JEI's search, recipes, uses and bookmarks over your screens.
- **Kind:** `native-mod`
- **Categories:** `utility`
- **Tags:** `recipes`, `item-browser`, `recipe-viewer`, `search`, `port`
- **Licence:** MIT
- **Source:** <https://github.com/joaopssx/dctrxds-jei>

### Description

```markdown
An unofficial Rust port of [Just Enough Items](https://github.com/mezz/JustEnoughItems) by mezz, for
[Horizon](https://playhorizon.dev). Not made or endorsed by mezz, the Horizon project, Mojang or Microsoft: please report its
bugs here, not to them.

- An item list beside your inventory, with JEI's search: `"phrases"`, `-exclude`, `a|b`, `@mod`, `#tag`, `&id`.
- R or a click for recipes, U or a right click for uses, with a tab per station and history.
- `+` fills the crafting grid with a recipe from your book.
- Bookmarks of items and recipes, drawn on the left of the screen.
- Cheat mode, off by default. Edit mode to hide items.
- Other mods can add categories with data files.

Not yet: fuel, composting, anvil and grindstone pages, fluids, and full tooltips on its own slots. They need helpers Horizon
doesn't expose today. [Everything that differs from JEI](https://github.com/joaopssx/dctrxds-jei/blob/main/docs/PORTING_NOTES.md).

Native mods are not sandboxed. This one reads the game's tables, and writes only to `config/dctrxds_jei/` and
`config/dctrxds_jei-client.toml`. It only sends `/give` if you turn that on and ask for an item.

Original work © mezz, MIT. This port © DctrXD, MIT.
```

## Build the file

```powershell
python tools\package.py
```

It writes `target\dctrxds_jei-<version>.hnmod` and prints its SHA-256. The same sources give the same bytes.
