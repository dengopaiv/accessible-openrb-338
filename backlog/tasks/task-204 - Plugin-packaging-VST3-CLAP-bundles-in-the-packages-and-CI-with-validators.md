---
id: TASK-204
title: 'Plugin packaging: VST3/CLAP bundles in the packages and CI, with validators'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-16T07:47:00Z'
updated_date: '2026-11-16T12:14:00Z'
labels:
  - tools
  - tests
milestone: m-7
dependencies:
  - TASK-185
  - TASK-197
references:
  - backlog task-185 (final summary)
  - scripts/package.sh
  - scripts/build-all.sh
  - .github/workflows/ci.yml
  - .github/workflows/release.yml
  - docs/building.md
  - scripts/package-dmg.sh
priority: medium
type: task
ordinal: 217000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The plugin crate (task-197) is shipped as .vst3 and .clap bundles with the existing packages (task-185's final summary), and checked by the plugin validators.

- scripts/package.sh TARGET bundles the plugin (the repository's xtask: `cargo xtask bundle orb-plugin --release`, which starts from its own folder) for the six targets and adds the bundles to the package archive; scripts/build-all.sh and release.yml follow. macOS bundles are ad-hoc signed like the app (no notarization yet).
- **macOS DMG (the user, 2026-11-16):** scripts/package-dmg.sh also puts `OpenRB-338.vst3` and `OpenRB-338.clap` into the DMG next to the app, with folder links to `/Library/Audio/Plug-Ins/VST3` and `/Library/Audio/Plug-Ins/CLAP` so the user drags each bundle onto its link (a DMG can only link absolute paths, so these are the all-users folders; macOS asks for an admin password). The archive keeps the bundles too. The DMG window's layout shows the app with its Applications link and the two plugin bundles with their links.
- **Quarantine:** the builds are only ad-hoc signed, not notarized, so a downloaded plugin carries the quarantine flag and a host may refuse it or report it as damaged. docs/building.md (and the README's install notes) give the fix (`xattr -dr com.apple.quarantine` on the installed bundles, and where to find them: `~/Library/Audio/Plug-Ins/...` or `/Library/Audio/Plug-Ins/...`). Notarization or a .pkg installer comes later with signing.
- CI runs clap-validator on the CLAP bundle and the VST3 SDK's validator (or pluginval) on the VST3 bundle, at least on one OS; document what runs where.
- docs/building.md: how to build and install the bundles locally (the plugin folders per OS), the DMG, the quarantine step, and the validators.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 scripts/package.sh produces .vst3 and .clap bundles in the package for aarch64-apple-darwin (verified locally: archive contents); the other targets use the same scripts (their first run is task-215)
- [x] #2 The macOS DMG contains the two plugin bundles with links to /Library/Audio/Plug-Ins/VST3 and /Library/Audio/Plug-Ins/CLAP (a check of the built DMG's contents)
- [x] #3 scripts/validate-plugin.sh runs clap-validator on the CLAP bundle and pluginval on the VST3 bundle and exits 1 on an error (verified locally on macOS: both pass on the built bundles; empty and broken bundles exit 1); ci.yml runs it on ubuntu-22.04 and macos-latest (the CI run itself is task-215)
- [x] #4 docs/building.md and README.md describe building, installing (incl. the DMG and the quarantine fix) and validating the plugin
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. xtask --no-build flag. 2. scripts/build.sh builds -p orb-plugin and bundles with the xtask into dist/<target>/plugins/. 3. package.sh: plugins/ in every archive; package-dmg.sh: signed bundles plus VST3/CLAP links; release.yml: plugins-<target> artifact for the dmg jobs. 4. scripts/validate-plugin.sh (pinned clap-validator and pluginval) and a ci.yml job on ubuntu and macOS. 5. docs/building.md 'The plugin'. 6. Local run on aarch64-apple-darwin, cargo test/clippy/fmt.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-acd4b86e183e7e4de

Local run (aarch64-apple-darwin, DIST in scratchpad): scripts/package.sh and package-dmg.sh built. tar.gz holds plugins/OpenRB-338.vst3 and .clap (signed, _CodeSignature). Mounted DMG: OpenRB-338.app, OpenRB-338.vst3, OpenRB-338.clap, Applications, VST3 -> /Library/Audio/Plug-Ins/VST3, CLAP -> /Library/Audio/Plug-Ins/CLAP, README with the install and xattr text; codesign --verify --strict ok, Signature=adhoc, arm64 only. scripts/validate-plugin.sh on the bundles: clap-validator 44 tests run, 36 passed, 0 failed, 8 skipped; pluginval 1.0.4 strictness 5 SUCCESS (pluginval used instead of Steinberg's validator, which needs a CMake build of the SDK). Assets pinned by version and SHA-256 in the script. actionlint and shellcheck clean; clippy -D warnings clean; nightly fmt clean; cargo test 777 pass. Not run here: the other five targets (same scripts; cargo-xwin and Docker paths of build.sh untested), the ubuntu run of validate-plugin.sh (xvfb), the CI job, the release.yml artifact hand-over to the dmg job.

Review round 1 (all six points): (1) CI/release-only claims moved out of the ACs into new task-215 (task-134 is Done and the user's, so not edited); exit-1 path run locally: validate-plugin.sh on an empty dir exits 1 ('no bundles'), on empty .clap/.vst3 folders exits 1 (pluginval FAILED). (2) build.sh now removes target/bundled before bundling; checked by building aarch64-apple-darwin twice with a plain file left at bundled/OpenRB-338.clap (a Windows-style bundle): both exit 0, signature ok. (3) README.md install step has the plugin and quarantine note. (4) sudo xattr for /Library, plain for ~/Library (package-dmg.sh README, docs/building.md, README.md, package.sh README). (5) comment in build.sh reworded. (6) DMG window layout dropped: icon positions need a .DS_Store, which needs a Finder session or a new tool (create-dmg, Python ds_store), none a dependency; recorded in task-215.

Review round 2: task-215 got five acceptance criteria (CI job passes, fails on error, five targets, artifact hand-over, DMG layout), the dependency on task-204, the refs, and a description that states the DMG layout as a requirement (tool open: create-dmg, ds_store, other headless way), not a decision.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Plugin bundles ship in every package archive (plugins/) and in the macOS DMG (bundles plus VST3/CLAP folder links) via scripts/build.sh (-p orb-plugin plus the xtask, new --no-build flag; target/bundled cleared before each bundling) and release.yml (plugins-<target> artifact for the dmg jobs). New scripts/validate-plugin.sh (pinned clap-validator and pluginval, SHA-256 checked) and CI job 'plugin validators' (ubuntu-22.04, macos-latest). docs/building.md 'The plugin' and README.md cover build, install, quarantine (sudo for /Library), validation. Verified locally on aarch64-apple-darwin: archive, DMG contents and links, ad hoc signature, clap-validator 36/44 passed 0 failed, pluginval SUCCESS, exit 1 on empty/broken bundles, two builds in a row; cargo test 777 pass. Not verified, in task-215: the CI job on both OSes, the five other targets (cargo-xwin and Docker paths), the plugins artifact hand-over to the dmg jobs. Dropped: the DMG window layout (icon positions, needs a .DS_Store; task-215).
<!-- SECTION:FINAL_SUMMARY:END -->
