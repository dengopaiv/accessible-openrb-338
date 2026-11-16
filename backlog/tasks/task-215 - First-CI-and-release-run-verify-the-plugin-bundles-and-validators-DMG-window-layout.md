---
id: TASK-215
title: >-
  First CI and release run: verify the plugin bundles and validators; DMG window
  layout
status: To Do
assignee: []
created_date: '2026-11-16T12:13:00Z'
updated_date: '2026-11-16T12:14:00Z'
labels:
  - tools
  - tests
milestone: m-7
dependencies:
  - TASK-204
references:
  - task-204
  - scripts/build.sh
  - scripts/package-dmg.sh
  - .github/workflows/ci.yml
  - .github/workflows/release.yml
  - scripts/validate-plugin.sh
priority: low
ordinal: 222000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Follow-up of task-204 (task-134, the first release run, is Done and the user's, so the plugin checks are here). Verify on the first push and release run: (1) ci.yml job 'plugin validators' passes on ubuntu-22.04 and macos-latest, including the Linux assets (SHA-256 pins in scripts/validate-plugin.sh) and pluginval under xvfb; (2) the job fails on a validator error (locally verified: an empty bundle dir exits 1 with a message, empty .clap/.vst3 folders exit 1); (3) the plugin bundles build for the five other targets, including the cargo-xwin (Windows on a Mac) and Docker (Linux) paths of scripts/build.sh, and every package archive holds plugins/; (4) the plugins-<target> artifact reaches the dmg jobs and scripts/package-dmg.sh accepts it (chmod, codesign --verify). Also required (the user asked for it on 2026-11-16, task-204's description; not an open decision): the DMG window layout, the app with its Applications link and the two bundles with their VST3 and CLAP links, with icon positions. It needs a .DS_Store written without a GUI session; the tool is the implementer's choice: create-dmg, the Python ds_store module, or another way that works headless in CI (the macOS dmg jobs).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The ci.yml job 'plugin validators' passes on ubuntu-22.04 and macos-latest
- [ ] #2 The job fails on a validator error
- [ ] #3 The five other targets build the plugin and every package archive holds plugins/, including the cargo-xwin and Docker paths of scripts/build.sh
- [ ] #4 The plugins-<target> artifact reaches the dmg jobs and scripts/package-dmg.sh accepts it
- [ ] #5 The DMG window shows the app with its Applications link and the two plugin bundles with their links (VST3, CLAP)
<!-- AC:END -->
