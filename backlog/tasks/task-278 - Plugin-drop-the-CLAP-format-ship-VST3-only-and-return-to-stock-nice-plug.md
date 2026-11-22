---
id: TASK-278
title: 'Plugin: drop the CLAP format, ship VST3 only, and return to stock nice-plug'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-22T07:48:00Z'
updated_date: '2026-11-22T07:59:00Z'
labels:
  - player
  - tests
  - docs
milestone: m-7
dependencies:
  - TASK-275
references:
  - docs/plugin.md
  - crates/orb-plugin
  - Cargo.toml
  - .github/workflows/release.yml
  - README.md
  - site/
  - https://github.com/pingw33n/nice-plug
priority: medium
type: task
ordinal: 259720
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user decided on 2026-11-22 to drop CLAP support. The plugin ships as VST3 only from the next release.

**What CLAP costs:**
- **The nice-plug fork:** task-275 made the 0..127 parameters continuous (GitHub issue 21, Studio One's stepped automation). VST3 needs nothing more, but keeping CLAP compatible with the released v2026.1118.0919 needed a patched nice-plug: the fork github.com/pingw33n/nice-plug, branch OpenRB-338, taken through `[patch.crates-io]` in Cargo.toml (task-276, docs/plugin.md "The forked nice-plug").
- **The test surface:** half the plugin's bundle tests, the CLAP REAPER fixtures, and REAPER's non-repeatable CLAP render of an enveloped project (task-275 notes).

The plugin uses no CLAP-only feature: no polyphonic modulation, no note expressions.

**Scope:**
- **Code:**
  - Remove the CLAP export and the CLAP-only code in crates/orb-plugin (ClapPlugin impl, the CLAP feature list, CLAP ids).
  - `cargo xtask bundle orb-plugin --release` then makes only `OpenRB-338.vst3`.
- **Stock nice-plug:**
  - Remove the three `[patch.crates-io]` git entries for nice-plug, nice-plug-core and nice-plug-derive, the Cargo.toml comment, and docs/plugin.md "The forked nice-plug".
  - Cargo.lock then takes nice-plug 0.4.2 and the rest from crates.io. Check with `cargo tree -d`.
  - The VST3 must behave exactly as with the fork: the patch touched only the CLAP wrapper. Keep task-275's VST3 tests, the stepCount split and the normalized mapping, all passing.
  - Leave the GitHub fork repository as it is; the user decides what to do with it.
- **Tests:**
  - Remove the CLAP cases from tests/exports.rs, tests/reaper.rs and its cases.rs, and the CLAP fixtures under tests/reaper/, regenerating them with examples/reaper_fixtures.rs if the generator lists formats.
  - Every VST3 case stays, with the same results.
- **Packaging and CI:**
  - release.yml, the macOS .pkg installer (task-238, scripts/package-pkg.sh), the Windows NSIS installer (scripts/installer/openrb.nsi, scripts/package-installer.sh), the .dmg (scripts/package-dmg.sh), the Linux archives and scripts/package*.sh carry only the VST3.
  - **The update removes the old CLAP** (the user's decision, 2026-11-22):
    - Windows: the NSIS installer deletes `$COMMONFILES64\CLAP\OpenRB-338.clap`, the folder the earlier installer put it in (openrb.nsi's SecClap). Do this whether or not the user picks the VST3 component. The uninstaller also removes it if present, so an old install's leftovers go.
    - macOS .pkg: a preinstall or postinstall script removes `/Library/Audio/Plug-Ins/CLAP/OpenRB-338.clap`. Only that bundle: never the CLAP folder or any other plugin. Make sure the pkg receipt of the earlier release doesn't keep claiming it, if that matters for pkgutil.
    - The .dmg (a drag install) and the Linux archive can't remove anything. Their instructions (the .dmg's first page, the archive's README, docs/plugin.md) tell the user to delete the old OpenRB-338.clap from /Library/Audio/Plug-Ins/CLAP, or from ~/.clap and /usr/lib/clap on Linux.
    - Test what can be tested: the NSIS script builds (makensis in CI), and the pkg scripts are checked by a unit or script test or a dry run. Do not run an installer against this Mac's /Library; the user runs installs.
- **Docs:**
  - docs/plugin.md (formats, install paths, section 7), docs/building.md, README (Features, install), the site (site/), and AGENTS.md where it names CLAP.
  - Say plainly that CLAP was dropped, that the installers remove the old CLAP, and that a project saved with the CLAP version of v2026.1118.0919 needs the plugin replaced by its VST3 by hand in the host.
- **Release note:** add a line for the changelog or release notes: CLAP removed, the installers delete the old CLAP, and what CLAP users do.

**Check:** grep the repository for "clap" and "CLAP" (case-insensitive) when done. What is left is either the `clap` command-line crate of orb-cli and orb-ui, or history in backlog/ and docs where it reads as history.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The plugin builds and bundles as VST3 only; no CLAP export, CLAP code or CLAP bundle is left; the installers and release.yml ship only the VST3
- [ ] #2 nice-plug comes from crates.io again: no [patch.crates-io] entry for nice-plug crates, cargo tree -d clean, and the VST3 behaves as with the fork (task-275's VST3 tests pass unchanged)
- [ ] #3 The CLAP test cases and fixtures are removed and every VST3 test, including the REAPER test with ORB_REAPER set, passes as before
- [ ] #4 docs/plugin.md, docs/building.md, README, the site and AGENTS.md describe a VST3-only plugin, say that CLAP was dropped and how a project saved with the CLAP version of the release is moved to the VST3; a release-note line is written
- [ ] #5 The Windows installer and the macOS .pkg remove the earlier release's OpenRB-338.clap (only that bundle) on install, the Windows uninstaller removes it too, and the .dmg and Linux instructions tell the user to delete it; the installer scripts are checked without installing on this Mac
- [ ] #6 cargo test --workspace passes, clippy -D warnings and nightly fmt clean
<!-- AC:END -->
