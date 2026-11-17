---
id: TASK-238
title: 'macOS DMG: a .pkg installer for the VST3 and CLAP plugins'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-17T15:50:00Z'
updated_date: '2026-11-17T15:53:00Z'
labels:
  - tools
  - docs
milestone: m-7
dependencies: []
references:
  - scripts/package-dmg.sh
  - .github/workflows/release.yml
  - docs/building.md
  - task-215
  - task-204
priority: high
type: feature
ordinal: 300
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The user found (2026-11-17, release run 44111505544's DMG): the DMG's plugin links (scripts/package-dmg.sh lines 55-56: VST3 -> /Library/Audio/Plug-Ins/VST3, CLAP -> /Library/Audio/Plug-Ins/CLAP) don't work: macOS creates neither folder (only plugin installers do), so the CLAP link shows as a blank document ("the original item for "CLAP" can't be found"), and dragging the bundles onto the links doesn't work even when the folder exists. The user's decision: replace the drag-and-drop plugin install with a .pkg installer inside the DMG.

Build `Install OpenRB-338 Plugins.pkg` with pkgbuild + productbuild in scripts/package-dmg.sh (or a helper script it calls), for each macOS target (x86_64, aarch64) from the plugin bundles it already gets (dist/<target>/plugins):
- Two choices in the installer's Customize pane, both selected by default: "VST3 plugin" installs OpenRB-338.vst3 into /Library/Audio/Plug-Ins/VST3, "CLAP plugin" installs OpenRB-338.clap into /Library/Audio/Plug-Ins/CLAP; the folders are created if missing (pkgbuild creates the install path). Machine-wide (the Installer asks for the admin password once); no per-user option needed.
- An upgrade replaces the bundles cleanly (no stale files inside the bundle: consider a preinstall script that removes the old bundle, or BundleIsRelocatable false and the right component plist so macOS doesn't relocate a bundle it finds elsewhere; document the choice).
- Unsigned for now (no Developer ID), like the app; the README says how to open it (right-click > Open, or System Settings > Privacy & Security > Open Anyway) and that the installed bundles need no xattr command (Installer-installed files aren't quarantined; verify and say so, or keep the xattr note if they are).
- The DMG no longer has the VST3 and CLAP links or the loose plugin bundles: it holds the app, the Applications link, the .pkg and README.txt. Update the DMG window layout (icon positions) accordingly and the README text; keep the Steinberg attribution sentence and registered sign in the README.
- release.yml's dmg jobs: anything they check (e.g. the plugins in the DMG) follows; the pkg needs no extra tool on macos runners (pkgbuild/productbuild ship with Xcode CLT).
- docs/building.md: the DMG section and the plugin install table. task-215's AC 5 (the DMG window with plugin links) is superseded by this: add a note to task-215 (backlog CLI) saying so.
Uninstalling: a pkg has no uninstaller; the README says to delete the two bundles from /Library/Audio/Plug-Ins/VST3 and CLAP.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 scripts/package-dmg.sh builds the DMG with OpenRB-338.app, the Applications link, Install OpenRB-338 Plugins.pkg and README.txt (no plugin links, no loose bundles), for both macOS targets.
- [ ] #2 The .pkg (pkgutil --expand / --payload-files listing in the notes) installs OpenRB-338.vst3 to /Library/Audio/Plug-Ins/VST3 and OpenRB-338.clap to /Library/Audio/Plug-Ins/CLAP, as two choices selected by default, creating the folders if missing; an upgrade replaces the bundles cleanly.
- [ ] #3 README.txt and docs/building.md describe opening the unsigned pkg, what it installs, quarantine (verified), and removal; task-215 notes that this supersedes its DMG plugin-link layout.
- [ ] #4 The user's check on macOS: the DMG opens with the new layout, the pkg installs both plugins on a Mac where the folders didn't exist, REAPER finds them.
- [ ] #5 bash -n on the changed scripts passes; cargo test passes if any test covers these scripts or docs.
<!-- AC:END -->
