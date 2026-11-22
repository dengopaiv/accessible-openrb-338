---
id: TASK-278
title: 'Plugin: drop the CLAP format, ship VST3 only, and return to stock nice-plug'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-22T07:48:00Z'
updated_date: '2026-11-22T08:41:00Z'
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
- [x] #1 The plugin builds and bundles as VST3 only; no CLAP export, CLAP code or CLAP bundle is left; the installers and release.yml ship only the VST3
- [x] #2 nice-plug comes from crates.io again: no [patch.crates-io] entry for nice-plug crates, cargo tree -d clean, and the VST3 behaves as with the fork (task-275's VST3 tests pass unchanged)
- [x] #3 The CLAP test cases and fixtures are removed and every VST3 test, including the REAPER test with ORB_REAPER set, passes as before
- [x] #4 docs/plugin.md, docs/building.md, README, the site and AGENTS.md describe a VST3-only plugin, say that CLAP was dropped and how a project saved with the CLAP version of the release is moved to the VST3; a release-note line is written
- [x] #5 The Windows installer and the macOS .pkg remove the earlier release's OpenRB-338.clap (only that bundle) on install, the Windows uninstaller removes it too, and the .dmg and Linux instructions tell the user to delete it; the installer scripts are checked without installing on this Mac
- [x] #6 cargo test --workspace passes, clippy -D warnings and nightly fmt clean
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Code: remove ClapPlugin/CLAP_ID/nice_export_clap, clap-sys, clap_gui test, CLAP editor-preview format, the CLAP logo part/svg/plugin_logos and PluginFormat::Clap, the vst3-only page markers. 2. Cargo: drop the three nice-plug [patch] entries, cargo update, check cargo tree -d. 3. Tests: drop CLAP cases in exports.rs, reaper.rs, cases.rs and the .clap.RPP fixtures (Format keeps Vst3 only). 4. Packaging: build.sh, package.sh, package-dmg.sh, package-pkg.sh (preinstall removes the old CLAP bundle), package-installer.sh, openrb.nsi (delete the old CLAP in install and uninstall), validate-plugin.sh, ci.yml, release.yml, changelog.sh. 5. Docs: plugin.md, building.md, README, site, AGENTS.md, assets/README.md; release note in changelog.sh. 6. Verify: cargo test --workspace, clippy, nightly fmt, bundle, pkg script dry run, makensis if available, grep clap.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-aa2c15431e5a63907

User request (2026-11-22, via the supervisor): a short visible notice in the site's download section: CLAP is no longer supported; the Windows installer and the macOS .pkg remove it on upgrade; .dmg and Linux: delete by hand. Covered under criterion 4.

Evidence: cargo test --workspace --locked: 1025 passed, 0 failed (VST3 tests the_vst3_parameters_are_continuous, the_vst3_factory_has_the_identity, a_note_at_frame_0_..._vst3_block_sounds all pass unchanged). ORB_REAPER=/Applications/REAPER.app/Contents/MacOS/REAPER cargo test -p orb-plugin --test reaper: ok, all 11 VST3 cases equal to the in-process host (REAPER started with -newinst by the test). clippy -D warnings and nightly fmt clean. cargo xtask bundle orb-plugin --release makes only OpenRB-338.vst3 (nm: GetPluginFactory, no clap_entry). cargo tree -d: no nice-plug duplicate; Cargo.lock nice-plug 0.4.2/core 0.4.2/derive 0.1.2 from crates.io. Packaging checks: scripts/test-pkg-scripts.sh passes (scratch volume, stand-in pkgutil, neighbors kept); package-pkg.sh run into a scratch DIST and pkgutil --expand shows one vst3.pkg with the preinstall script; makensis -WX builds openrb.nsi into an exe (no installer run). Not done: no installer was run on this Mac; the NSIS removal logic is checked by build only (the user runs installs). The site notice (user request) is in site/index.html in the download section. Left as history: CLAP mentions in docs/plugin.md sections 3-10 and editor.rs comments about host behavior found with the CLAP bundle.

Review round 1 (sent back, fixed): docs/midi.md, standard_midi_mapping.md, ui_spec.md and plugin.md 227 say VST3 only; cases.rs doc comment rewrapped; stale CLAP comments in editor.rs, tests/editor.rs, tests/state.rs reworded; bare blocks in tests/reaper.rs are plain lets; ci.yml blank line, release.yml, package-installer.sh and README.md long lines wrapped (both workflows parse with ruby YAML); commit subject now names areas. Re-run: cargo test -p orb-plugin ok, clippy -D warnings and nightly fmt clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
CLAP dropped; the plugin is VST3 only and nice-plug 0.4.2 comes from crates.io again (the three patch entries and docs/plugin.md "The forked nice-plug" are gone). Code: ClapPlugin, CLAP_ID and the export, clap-sys, tests/clap_gui.rs, the .clap fixtures, the CLAP cases in exports.rs, reaper.rs, cases.rs, the CLAP logo (part, svg, plugin_logos.rs) and PluginFormat::Clap are removed; the Plugin Usage vst3-only markers are gone. Packaging: Windows openrb.nsi has no CLAP component; RemoveOldClap (install, always) and un.RemoveOldClap delete only OpenRB-338.clap in COMMONFILES64\CLAP and the recorded ClapDir. macOS pkg: one vst3 component; scripts/pkg/preinstall removes the old VST3 and only /Library/Audio/Plug-Ins/CLAP/OpenRB-338.clap and runs pkgutil --forget of the old clap receipt id; scripts/test-pkg-scripts.sh checks it (also in ci.yml). dmg README, archive README.txt, build.sh, package.sh, validate-plugin.sh (clap-validator dropped), release.yml and ci.yml updated. Docs: plugin.md (intro, section 7.2), building.md, README, AGENTS.md, assets/README.md, the site (a visible notice in the download section, the user request), release note in the scripts/changelog.sh intro. Tests: cargo test --workspace 1025 pass, REAPER test with ORB_REAPER passes, clippy and nightly fmt clean. Branch worktree-agent-aa2c15431e5a63907, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
