---
id: TASK-237
title: 'Windows installer: optional VST3 and CLAP plugin components'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-17T15:37:00Z'
updated_date: '2026-11-17T17:18:00Z'
labels:
  - tools
  - player
milestone: m-7
dependencies: []
references:
  - scripts/installer/openrb.nsi
  - scripts/package-installer.sh
  - .github/workflows/release.yml
  - scripts/package.sh
  - docs/building.md
  - task-111
  - task-204
priority: high
type: feature
ordinal: 200
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The user's decisions (2026-11-17): the Windows NSIS installer (scripts/installer/openrb.nsi, scripts/package-installer.sh, release.yml job `installer`, task-111) installs the plugins too, and it is ALL USERS ONLY (final decision, replacing an earlier per-user-only and a per-user-or-all-users choice): RequestExecutionLevel admin (one UAC prompt at start), the app in $PROGRAMFILES64\OpenRB-338, HKLM (64-bit view), common Start menu/desktop shortcuts, the machine PATH (path.ps1 machine), and two optional components, checked by default: "VST3 plugin" installs the OpenRB-338.vst3 bundle folder to $COMMONFILES64\VST3 and "CLAP plugin" installs OpenRB-338.clap to $COMMONFILES64\CLAP (x86_64 and aarch64 alike). Silent options /VST3=, /CLAP=, /VST3DIR=, /CLAPDIR=, /D=. The uninstaller removes what the install recorded in HKLM (the two bundles, nothing else in the shared folders); reinstall/upgrade replaces the bundles. An earlier per-user install (previous versions, HKCU) is detected and the user is told how to remove it (Settings > Apps); it is not touched. The installer job gets the bundles from the plugins-<target> artifacts for both architectures. The .zip README.txt (package.sh), docs/building.md and the installer readme describe it.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The installer offers 'VST3 plugin' and 'CLAP plugin' components, checked by default, installing to the documented folders for x86_64 and aarch64.
- [x] #2 The uninstaller removes the installed plugin bundles and nothing else in the shared folders; reinstalling replaces them.
- [x] #3 release.yml's installer job is wired to get the plugin bundles for both architectures, and makensis run locally (nsis from Homebrew or apt in Docker) on a staged folder with real or stand-in bundles produces an installer whose listing (7z l) holds them.
- [x] #4 The user's check on Windows: install, the plugins show in a host (REAPER), uninstall removes them.
- [x] #5 docs/building.md and the package README.txt describe it; makensis builds without warnings that MAKENSIS_FLAGS treats as errors.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. openrb.nsi: define PLUGINDIR; optional sections "VST3 plugin" and "CLAP plugin" (checked by default), recorded in HKLM (Vst3Dir, ClapDir); reinstall removes the old bundle first; uninstall removes only OpenRB-338.vst3 / OpenRB-338.clap and never the shared folders; silent options /VST3=0|1, /CLAP=0|1, /VST3DIR=, /CLAPDIR=. SUPERSEDED design of this item (per-user folders, RequestExecutionLevel user, HKCU): replaced by the user's final decision of 2026-11-17, ALL USERS ONLY: RequestExecutionLevel admin, $PROGRAMFILES64, HKLM, machine PATH, plugins in $COMMONFILES64\VST3 and \CLAP (see the notes).
2. package-installer.sh: stage dist/<target>/plugins into the stage (build-time mtime), pass -DPLUGINDIR.
3. release.yml: installer job also downloads plugins-<target>.
4. package.sh README.txt, docs/building.md.
5. Local makensis -WX build on stand-in bundles for both targets; 7zz l listing. AC 4 is the user check on Windows.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
The supervisor verifies the real release run after the merge (installer artifact listing for x86_64 and aarch64); not an AC since agents don't push.

Branch: worktree-agent-aecb956711af86f3a (one commit, rebased onto main 3429c99 in review round 2).

SUPERSEDED (kept as history; the design now is the FINAL DECISION below): the per-user-only decision made inside the task. The plugin components were to install per user to %LOCALAPPDATA%\Programs\Common\VST3 and \CLAP, the installer to stay RequestExecutionLevel user (no UAC). Reason then: NSIS cannot elevate for part of a run; elevating the whole installer would make it per machine and add the admin prompt to an unsigned installer. Risk noted then: a host scanning only %COMMONPROGRAMFILES% will not see them.
Evidence (of the first build, still valid for the plugin staging): SOURCE_DATE_EPOCH=1700000000 DIST=<scratch>/dist SKIP_BUILD=1 scripts/package-installer.sh run for both targets with stand-in bundles (vst3 folder with Contents/<arch>-win/OpenRB-338.vst3, a clap file), default -WX, no warnings. The Windows bundle layout (clap a file, vst3 folder with Contents/<arch>-win) is assumed from nih-plug and build.sh comments; package-installer.sh fails when the vst3 is not a folder or the clap not a file. No cargo test: no Rust code or test-covered file changed.

SUPERSEDED (kept as history; replaced by the FINAL DECISION below): the user's decision of 2026-11-17 for All users (default) and Just me via MultiUser.nsh (MULTIUSER_EXECUTIONLEVEL Highest, install-mode page, /ALLUSERS and /CURRENTUSER, per-mode registry hive and folders, the uninstaller picking the mode by its own folder). Its known deviation was that an administrator account got UAC at start (highestAvailable) whatever the mode. Built with makensis 3.13 -WX (x64 149596 bytes, arm64 149418), never run.

FINAL DECISION (user, 2026-11-17), supersedes the two notes above (per-user only; per-user or all users with MultiUser.nsh): ALL USERS ONLY. MultiUser.nsh and its page are removed. openrb.nsi: RequestExecutionLevel admin (built exes carry requestedExecutionLevel level="requireAdministrator", checked with strings), InstallDir $PROGRAMFILES64\OpenRB-338 with InstallDirRegKey HKLM, SetRegView 64, SetShellVarContext all, all registry in HKLM, path.ps1 called with scope machine, plugins to $COMMONFILES64\VST3 and \CLAP (HKLM Vst3Dir/ClapDir recorded; uninstaller removes only the two bundle names). /ALLUSERS and /CURRENTUSER do not exist. Earlier per-user install: the installer reads HKCU\Software\OpenRB-338 InstallDir of the user running it, and if set shows a message (log only under /S) that it stays and how to remove it (Settings > Apps); not migrated or uninstalled (simple; an elevated run as another account has another HKCU, so the note can be missing). Documented in docs/building.md, installer readme, zip README.txt. Rebuilt with makensis 3.13 -WX, stand-in bundles: x64 145770 bytes, arm64 145769; 7zz l shows OpenRB-338.exe, orb.exe, LICENSE.txt, README.txt, path.ps1, $_0_/OpenRB-338.vst3/Contents/<x86_64|arm64>-win/OpenRB-338.vst3, $_1_/OpenRB-338.clap, Uninstall.exe (13 files). apt NSIS 3.08 not tried locally.

User check on Windows passed (2026-11-17, "installer is good"): the installer from release run 44130121208, built from commit d4cc0ef (branch task-237). Its installer jobs built both architectures from the plugins-<target> artifacts.

Review round 1 (send back, docs and comments only; installer code unchanged from d4cc0ef): rebased onto main e1a4f89 (building.md conflict kept main's DMG/pkg sentences plus the Windows installer part); "Checked" paragraph now records the user's passed check (run 44130121208, from d4cc0ef); openrb.nsi comment on the plugin sections matches the code (VST3 reports an old folder it cannot remove, CLAP has no own check); MultiUser mention cut to nothing; per-user plugin folders marked manual-install-only. Follow-up: task-240 (uninstaller reports a held bundle only in the log and drops the HKLM record; CLAP copy has no post-copy check).

Review round 2 (send back, text only; installer code unchanged): rebased onto main 3429c99; subject kept as "tools: ..." (AGENTS.md: a subject names the change's own areas); docs/building.md says the held-open message is VST3 only; this plan and these notes marked superseded where they describe the per-user design; task-240 line numbers corrected; release.yml installer comment rewrapped.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The Windows installer is all users only: admin (requireAdministrator), Program Files, HKLM, common shortcuts, machine PATH, and optional default-on VST3 and CLAP plugin components in $COMMONFILES64\VST3 and \CLAP. The uninstaller removes what HKLM records (the two bundles only); reinstall replaces them; an earlier per-user install is noted, not touched. Silent /VST3 /CLAP /VST3DIR /CLAPDIR /D. release.yml installer job downloads plugins-<target>. Docs and both readmes updated. Verified: local makensis 3.13 -WX builds on stand-in bundles with 7zz l listings; release run 44130121208 built both installers (x64, arm64) from the plugins-<target> artifacts; the user checked the installer on Windows on 2026-11-17 and it passed. Branch task-237 (worktree-agent-aecb956711af86f3a), one commit. No cargo test (no Rust or test-covered file changed).
<!-- SECTION:FINAL_SUMMARY:END -->
