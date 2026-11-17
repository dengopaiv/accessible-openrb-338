---
id: TASK-238
title: 'DMG README: the VST3 and CLAP links can be broken until the folders exist'
status: To Do
assignee:
  - '@agent-chore'
created_date: '2026-11-17T15:50:00Z'
labels:
  - tools
  - docs
milestone: m-7
dependencies: []
references:
  - scripts/package-dmg.sh
  - docs/building.md
  - task-215
priority: medium
type: docs
ordinal: 300
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The user found (2026-11-17, release run 44111505544's DMG): the DMG's CLAP link shows as a blank document and Finder says "The operation can't be completed because the original item for "CLAP" can't be found." scripts/package-dmg.sh links VST3 -> /Library/Audio/Plug-Ins/VST3 and CLAP -> /Library/Audio/Plug-Ins/CLAP (lines 55-56); macOS creates neither folder, plugin installers do, so either link can be broken. The user's decision: keep the links and add a README note for now.

Add to the DMG README.txt (the heredoc in scripts/package-dmg.sh, in the plugin paragraph after "drag OpenRB-338.vst3 onto the VST3 link ...") this text, wrapped like the surrounding lines:

"If the VST3 or CLAP link shows as a blank document (the folder does not exist yet: macOS does not create it, plugin installers do), create both folders first, then drag again:
    sudo mkdir -p /Library/Audio/Plug-Ins/VST3 /Library/Audio/Plug-Ins/CLAP
or install for you only into ~/Library/Audio/Plug-Ins/VST3 and ~/Library/Audio/Plug-Ins/CLAP (mkdir -p them the same way, without sudo)."

Add one sentence with the same fact to docs/building.md where it describes the DMG's VST3 and CLAP links.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The DMG README.txt (scripts/package-dmg.sh) has the note with the sudo mkdir -p command and the per-user alternative.
- [ ] #2 docs/building.md's DMG description says the links can be broken until the folders exist.
- [ ] #3 Any test that checks package-dmg.sh or docs still passes (cargo test); bash -n scripts/package-dmg.sh passes.
<!-- AC:END -->
