---
id: TASK-240
title: >-
  Windows installer: report a plugin a host holds at uninstall, and check the
  CLAP copy
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-17T17:15:00Z'
updated_date: '2026-11-17T17:19:00Z'
labels:
  - tools
milestone: m-7
dependencies: []
references:
  - scripts/installer/openrb.nsi
priority: low
type: bug
ordinal: 241000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Found in the review of task-237 (scripts/installer/openrb.nsi, as built in d4cc0ef): (1) the uninstaller reports a plugin bundle that a host still holds only with DetailPrint (openrb.nsi:314 and :321), then deletes the HKLM record (DeleteRegKey, :342-343), so the bundle is orphaned silently and a later uninstall no longer knows its folder. Use a MessageBox like the install side, or keep the HKLM record (at least Vst3Dir/ClapDir) until the bundle is gone, so the uninstall can be repeated as it is for the program files (exit code 6). (2) The CLAP install section has no post-copy check like the VST3 section has (:206-209, the old bundle could not be removed: message and Abort). Add an equivalent for the file. Needs a check on Windows by the user (a host holding the plugin open, then install, upgrade and uninstall) and docs/building.md ("The Windows installer", the plugins paragraph) updated.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The uninstaller does not delete the HKLM record while a plugin bundle it recorded is still there, or tells the user with a message (silent: exit code), so the uninstall can be repeated
- [ ] #2 The CLAP section reports a failed overwrite like the VST3 section does
- [ ] #3 The user checks it on Windows with a host holding the plugin
<!-- AC:END -->
