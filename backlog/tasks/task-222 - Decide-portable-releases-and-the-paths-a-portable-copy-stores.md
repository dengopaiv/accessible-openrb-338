---
id: TASK-222
title: 'Decide: portable releases, and the paths a portable copy stores'
status: Done
assignee:
  - '@dengopaiv'
created_date: '2026-11-09T18:13:00Z'
updated_date: '2026-11-09T18:14:00Z'
labels:
  - decision
  - tools
milestone: m-7
dependencies: []
priority: high
type: spike
ordinal: 240000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Context: the fork's first release draft (v2026.1109.1802, 2026-11-09) had upstream's two Windows downloads: the NSIS installer (per user, Start menu, uninstaller) and a zip. The user asked: "Installer that is to install the entire thing to the computer? Can we have a portable release. Can we have our releases be portable?" The zip ran from any folder but still wrote the settings to %APPDATA%\OpenRB-338\ui.toml and the logs to %LOCALAPPDATA%\OpenRB-338\logs; nothing else is written outside the program's folder (songs go where the user saves them).

Questions and options put to the user:
1. Downloads: (a) the portable zip only, the installer script kept in the repository but not used for releases (recommended); (b) the portable zip and the installer.
2. On a USB stick the drive letter changes: (a) folders inside the portable folder are stored relative to it (recommended); (b) full paths as now.
<!-- SECTION:DESCRIPTION:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The user decided on 2026-11-09: the fork's releases are the portable zip only (no installer), and a portable copy stores the folders that lie inside its own folder as paths relative to it. Implemented by TASK-223.
<!-- SECTION:FINAL_SUMMARY:END -->
