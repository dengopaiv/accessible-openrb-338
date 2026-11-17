---
id: TASK-238
title: 'macOS DMG: a .pkg installer for the VST3 and CLAP plugins'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-17T15:50:00Z'
updated_date: '2026-11-17T16:46:00Z'
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
The user found (2026-11-17, release run 44111505544's DMG): the DMG's plugin links (VST3 -> /Library/Audio/Plug-Ins/VST3, CLAP -> /Library/Audio/Plug-Ins/CLAP) don't work: macOS creates neither folder (only plugin installers do), so the CLAP link shows as a blank document, and dragging the bundles onto the links doesn't work even when the folder exists.

Final design (the user's decisions, 2026-11-17): the drag-and-drop plugin install is replaced by a .pkg installer, and the pkg is NOT in the DMG but its own release asset, OpenRB-338-Plugins-<YYYY.MMDD.HHMM>-<target>.pkg (one per macOS target, written to dist/<target>/ by scripts/package-pkg.sh). The DMG holds only OpenRB-338.app, the Applications link, README.txt and LICENSE; its README names the pkg. The pkg (pkgbuild + productbuild) has two choices, VST3 plugin and CLAP plugin, both selected by default, installs machine-wide into /Library/Audio/Plug-Ins/VST3 and /CLAP (folders created), a preinstall script removes the old bundle for clean upgrades, and a welcome page carries the unsigned-open steps, the quarantine note and the removal note. release.yml's dmg job builds and uploads pkg-<target>; the draft release attaches both pkgs. Unsigned like the app. No uninstaller: delete the two bundles. The README.md install section follows.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user's check on macOS: the DMG opens with the new layout, the pkg installs both plugins on a Mac where the folders didn't exist, REAPER finds them.
- [x] #2 scripts/package-dmg.sh builds the DMG with only OpenRB-338.app, the Applications link, README.txt and LICENSE (no pkg, no plugins; the README names the separate pkg), for both macOS targets
- [x] #3 scripts/package-pkg.sh TARGET builds dist/<target>/OpenRB-338-Plugins-<version>-<target>.pkg: VST3 and CLAP as two choices selected by default, installed to /Library/Audio/Plug-Ins/VST3 and /CLAP (folders created), preinstall removing the old bundle, a welcome page with the unsigned-open instructions, quarantine and removal
- [x] #4 release.yml's dmg job builds and uploads pkg-<target>, the draft release attaches both pkgs (release-assets.sh lists 14 files)
- [x] #5 README.txt, docs/building.md and task-215 describe the layout (pkg as its own download, quarantine, removal)
- [x] #6 bash -n passes on the changed scripts
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. scripts/package-dmg.sh: drop VST3/CLAP links and loose bundles; build two component pkgs (pkgbuild --root, --install-location /Library/Audio/Plug-Ins/{VST3,CLAP}, --component-plist with BundleIsRelocatable false, --scripts with a preinstall that removes the old bundle for clean upgrades, --ownership recommended) and a productbuild --distribution with two choices start_selected, into Install OpenRB-338 Plugins.pkg; verify bundles' signature and arch as before. 2. README text: how to open unsigned pkg, what it installs, quarantine, removal; keep Steinberg sentence. 3. docs/building.md DMG section and plugin install table. 4. task-215 note: AC 5 superseded (no icon layout exists; none added). 5. release.yml: no plugin-in-DMG check exists, so comment only. 6. Build aarch64 DMG locally; pkgutil --payload-files / --expand, mount DMG, inspect. Not run: the installer (AC 4 is the user's).
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a2170db45770fbfdd

Built aarch64 locally: scripts/build.sh aarch64-apple-darwin, then SKIP_BUILD=1 scripts/package-dmg.sh (35.9 MB DMG, mounted read-only: Applications link, Install OpenRB-338 Plugins.pkg 21.2 MB, LICENSE, OpenRB-338.app, README.txt; no VST3/CLAP links, no loose bundles). pkgutil --expand-full: Distribution (hostArchitectures arm64, customize allow, choices vst3 and clap both start_selected/start_enabled), vst3.pkg and clap.pkg: install-location /Library/Audio/Plug-Ins/VST3 and /CLAP, relocatable=false, auth=root, payload 17 files each, owner root:wheel, preinstall script removing the old bundle; the payload bundles pass codesign --verify --strict. Failed/changed: first version used a component plist with BundleIsRelocatable false and BundleOverwriteAction upgrade; pkgbuild then wrote <upgrade-bundle> keyed by the bundle id, and both plugins share com.nice-plug.orb-plugin, so it was replaced by an empty component plist (no bundle entries, plain files at the fixed path) plus the preinstall script. Not verified: the Installer was not run (AC 4, the user's); the x86_64 image was not built here (same code path, arch check and hostArchitectures differ); quarantine behavior of installed files is from Apple's documented behavior, not tested (README keeps a sudo xattr fallback). The pkg Bom has ._ entries here only because this sandbox adds com.apple.provenance to every file (xattr -c cannot remove it, COPYFILE_DISABLE does not help); CI has none. No headless icon layout exists or was added (noted in task-215). release.yml needs no change (it checks nothing about the plugins in the DMG). No cargo test covers these scripts; bash -n passes.

DECISION (user, 2026-11-17): the plugin pkg leaves the DMG and is its own download. Layout now: the DMG holds OpenRB-338.app, the Applications link, LICENSE, README.txt (which names OpenRB-338-Plugins-<version>-<target>.pkg); scripts/package-pkg.sh TARGET (new; package-dmg.sh no longer builds a pkg) writes dist/<target>/OpenRB-338-Plugins-<version>-<target>.pkg with the choices, preinstall and fixed install paths of the first round, plus a productbuild --resources welcome.txt (unsigned-open steps, what installs where, quarantine, removal). release.yml: the dmg job runs package-pkg.sh and uploads pkg-<target>; the draft-release job downloads pkg-*; scripts/release-assets.sh expects 14 files (15 assets with SHA256SUMS.txt); changelog.sh lists no downloads, so no change. Rebuilt aarch64: DMG 13,872,094 bytes (mounted: Applications, LICENSE, OpenRB-338.app, README.txt), pkg 21,196,934 bytes (pkgutil --expand-full: Distribution with welcome, hostArchitectures arm64, two choices; Resources/welcome.txt; vst3.pkg and clap.pkg with Payload, Scripts/preinstall, Bom; both bundles pass codesign --verify --strict). The earlier AC list described the pkg inside the DMG; it was replaced by the current list.

Review round 1 (send back), fixed: README.md macOS install now names the separate pkg (no links, no xattr line); no other stale mentions of the DMG plugin links in docs/, scripts/, crates/; welcome text has VST® at first use and a one-line xattr fallback; docs/building.md quarantine wording is now verified and the lines at 397 and 531 are wrapped; the description is rewritten to the final design. User check (2026-11-17, macOS 15, aarch64 DMG and pkg): 'look fine'; machine-wide only as built, the greyed-out 'for me only' / 'specific disk' on the Destination page is accepted. Supervisor's verification after the user's install: xattr -lr on /Library/Audio/Plug-Ins/VST3/OpenRB-338.vst3 and /CLAP/OpenRB-338.clap shows only com.apple.provenance, no com.apple.quarantine; find -name '._*' in both finds nothing; pkgutil --pkgs lists net.emphased.openrb338.plugin.vst3 and .clap.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The plugin installer is its own download. scripts/package-pkg.sh builds OpenRB-338-Plugins-<version>-<target>.pkg (two choices selected by default, /Library/Audio/Plug-Ins/VST3 and /CLAP, preinstall for clean upgrades, welcome page with the unsigned-open, quarantine and removal notes); package-dmg.sh makes the DMG with only the app, the Applications link, LICENSE and README.txt (it names the pkg); release.yml uploads pkg-<target>, release-assets.sh requires 14 files; README.md and docs/building.md follow. Built aarch64 only. The user installed the pkg on macOS 15 and it looked fine; quarantine verified absent on the installed bundles, no stray ._ files. Tests: bash -n; no cargo test covers these scripts or docs. Branch worktree-agent-a2170db45770fbfdd, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
