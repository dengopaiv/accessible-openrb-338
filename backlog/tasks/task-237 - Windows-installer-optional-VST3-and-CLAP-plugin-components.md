---
id: TASK-237
title: 'Windows installer: optional VST3 and CLAP plugin components'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-17T15:37:00Z'
updated_date: '2026-11-17T15:37:00Z'
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
The user's decision (2026-11-17): the Windows NSIS installer (scripts/installer/openrb.nsi, scripts/package-installer.sh, release.yml job `installer`, task-111) installs only the app and the CLI (with the PATH entry); the plugin bundles exist only in the .zip's plugins\ folder (scripts/package.sh, task-204), to be copied by hand. Windows users expect the installer to set the plugins up.

Add two optional components, both checked by default: "VST3 plugin" installs the OpenRB-338.vst3 bundle folder to $COMMONFILES64\VST3 (the standard %COMMONPROGRAMFILES%\VST3; on arm64 the same folder), "CLAP plugin" installs OpenRB-338.clap to $COMMONFILES64\CLAP. Both need admin rights: check how the installer elevates today (per-user vs per-machine install) and make the plugin components work in that mode (if the installer is per-user, either require elevation for the plugin components or install to the per-user folders %LOCALAPPDATA%\Programs\Common\VST3 and \CLAP; pick one, document why). The uninstaller removes exactly what it installed (and nothing else in those shared folders). Reinstall/upgrade replaces the bundles. The installer's inputs come from the package job's artifacts: make sure the plugins reach the installer job (release.yml: bin-<target> artifacts and/or the plugins-<target> artifact) for both x86_64 and aarch64.

Update the .zip's README.txt (package.sh) to say the installer can install the plugins, docs/building.md (the installer section, the plugin install table), and the release notes text if it describes installation.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The installer offers 'VST3 plugin' and 'CLAP plugin' components, checked by default, installing to the documented folders for x86_64 and aarch64.
- [ ] #2 The uninstaller removes the installed plugin bundles and nothing else in the shared folders; reinstalling replaces them.
- [ ] #3 release.yml's installer job is wired to get the plugin bundles for both architectures, and makensis run locally (nsis from Homebrew or apt in Docker) on a staged folder with real or stand-in bundles produces an installer whose listing (7z l) holds them.
- [ ] #4 The user's check on Windows: install, the plugins show in a host (REAPER), uninstall removes them.
- [ ] #5 docs/building.md and the package README.txt describe it; makensis builds without warnings that MAKENSIS_FLAGS treats as errors.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
The supervisor verifies the real release run after the merge (installer artifact listing for x86_64 and aarch64); not an AC since agents don't push.
<!-- SECTION:NOTES:END -->
