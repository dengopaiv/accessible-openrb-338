---
id: TASK-204
title: 'Plugin packaging: VST3/CLAP bundles in the packages and CI, with validators'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-16T07:47:00Z'
updated_date: '2026-11-16T11:54:00Z'
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
- [ ] #1 scripts/package.sh produces .vst3 and .clap bundles in the package for this machine's target; the other targets are wired the same way
- [ ] #2 The macOS DMG contains the two plugin bundles with links to /Library/Audio/Plug-Ins/VST3 and /Library/Audio/Plug-Ins/CLAP (a check of the built DMG's contents)
- [ ] #3 CI runs clap-validator and a VST3 validator on the bundles and fails on errors
- [ ] #4 docs/building.md describes building, installing (incl. the DMG and the quarantine fix) and validating the plugin
<!-- AC:END -->
