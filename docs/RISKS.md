# Legal and quality risks

This is an engineering checklist, not legal advice.

## Licence and credit

| Duty | Source | What we do |
|---|---|---|
| Keep mezz's copyright and permission notice | JEI `LICENSE.txt` (MIT) | Include the full notice in `LICENSE`/`NOTICE.md`, and shipped inside the bundle |
| Notice in derived files | Your rule | Header on each file translated from JEI: original path, `Copyright (c) 2014-2015 mezz`, MIT |
| Keep the notice in the `.hnmod` | A bundle holds only `mixins/<id>/`; `LICENSE*` is allowed in the bundle or crate folder (`docs/native-mods.md`) | Put a copy in `mixins/dctrxds_jei/` and `core/`. The root `LICENSE` would **not** ship |
| Credit and link the original on the page | `MOD-POLICY.md` promise 2 | Registry description names mezz and links `github.com/mezz/JustEnoughItems` |
| Licence must be reviewable | `MOD-POLICY.md` | MIT is on the list |

**Care with these JEI files:**

- `Library/.../color/ColorThief.java` and `MMCQ.java` have their **own licence headers** **[RAN]** (found by search; the
  licence itself was not reviewed). Read them first if colour search is ever ported. Not planned.
- **JEI's textures (51 PNGs under `assets/jei/textures`).** MIT covers the repository as far as I can tell, as no separate
  licence was found by a search of `README.md` and `CONTRIBUTING.md`. Whether any are derived from Mojang's art is **not
  determined**. Safe default: use vanilla sprites from the player's game by name, or draw our own. See
  [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) Q5.
- **Translations (`assets/jei/lang`, 31 files, including `pt_br.json`).** Part of the same repository, written by community
  members. Keep the notice; the keys may be restructured.
- **Do not copy** third-party mod code without checking. The community `horizon-jei` is MIT per GitHub's API. If we read or
  use its code, we credit it.

## Stating what this is

The README and the registry page must say, near the top:

> **Unofficial port.** DctrXD´s JEI is an independent Rust port of Just Enough Items by mezz. It is not made, endorsed or
> supported by mezz, the JEI project, Horizon, Mojang or Microsoft. Please do not report its bugs to them.
> NOT AN OFFICIAL MINECRAFT PRODUCT. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.

Also, because native mods are unsandboxed:

> Native mods run with your account's permissions. This one reads the game's tables and writes only to
> `config/dctrxds_jei/`. It has no hidden network use. Read the source before building.

**[ASSUMED]** "JEI" in a name is common for ports, but MIT does not grant any right to a name. `JEI for Horizon` is already
listed with it. A courtesy message to mezz is cheap and is your decision (Q2).

## Mojang and game files

- Never commit jars, assets, decompiled code or prepared resources (`.local/`, `native-data` output).
- Add these to `.gitignore` from the first commit: `.local/`, `target/`, `*.jar`, stages, and logs.
- `MOD-POLICY.md` promise 3: no code, assets or data from Minecraft beyond what the EULA and guidelines allow.
- **Recipes and item names at runtime come from the player's game**, never stored in our repository. Test tables use
  made-up data or small handwritten samples with no copied JSON.
- **Do not embed a recipe database** for offline use.
- Vanilla-looking textures of our own need care too: do not trace Mojang's art.

## Registry policy (`MOD-POLICY.md`, `docs/api/mod-index.md`)

Before submitting:

- [ ] Id `dctrxds_jei` valid and unused (`[a-z0-9_]{1,64}`)
- [ ] Licence MIT, notices kept, original credited and linked
- [ ] Source public, and exactly what the file was built from
- [ ] No Minecraft content, malware, or hidden network access
- [ ] Description ≥ 50 characters, 1 to 3 categories, square icon 64 to 1024 px ≤ 256 KiB
- [ ] `cargo horizon-mod check` passes
- [ ] Version in `horizon-native.json` matches `horizon-mixin.toml`
- [ ] Each version gets a review; a published file never changes
- [ ] Minisign key recommended, and **kept safe and out of the repository**

You, as submitter, personally promise these. An agent can prepare it; you submit it.

**The README in `horizon-mods` and `docs/mods.md` disagree** on whether versions come from GitHub releases or through the
form, and `docs/api/mod-index.md`'s changelog says new entries may not use `repo` since 2026-11-05. Follow `mod-index.md`,
and re-read it at submission time.

## Safety rules while developing

- **Trust flags are yours.** `--trust-native-code` and `--trust-native-mods` are never supplied by an agent.
- Keep tests away from real data: temporary `HORIZON_HOME`, `HORIZON_LAUNCHER_HOME`, and `--game-dir` (`AGENTS.md`). On
  Windows, confirm these variables also work, and use a separate launcher instance.
- Do not edit `../horizon`; `horizon-mixins` works on a separate stage.
- Upstream PRs follow `CONTRIBUTING.md`: `/claim`, small, `pr_ready`, declare what was read, and mark them as agent-opened.

## Code quality

| Risk | Rule |
|---|---|
| A panic takes down the game | No `unwrap` on input or files in the adapter; handle bad data and continue |
| Corrupt user files | Write to a temp file then rename; on a parse error keep a `.bak` and start fresh, and warn |
| Slowness | No file access or heavy work in draw hooks; run searches once per change |
| A ported test proves nothing | Use independent expected values, as `AGENTS.md` asks; never derive them from the code under test |
| Claims beyond evidence | README and changelog list only what was seen working, with a "not tested" section |
| Dead weight | Do not port features before their phase; do not keep Java-isms like plugin wrappers |
| Abandonment | Daily `broken_on` in the registry: watch it, or say plainly that the mod is unmaintained |
