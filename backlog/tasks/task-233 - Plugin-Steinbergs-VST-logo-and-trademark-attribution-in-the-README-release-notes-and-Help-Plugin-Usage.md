---
id: TASK-233
title: >-
  Plugin: Steinberg's VST logo and trademark attribution in the README, release
  notes and Help > Plugin Usage
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-17T09:25:00Z'
updated_date: '2026-11-17T10:07:00Z'
labels:
  - docs
  - tools
milestone: m-7
dependencies:
  - TASK-231
references:
  - assets/README.md
  - docs/plugin.md
  - task-221
  - task-232
priority: medium
type: task
ordinal: 234000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: task-232 (decided by the user 2026-11-17) chose the plain text 'VST3' on the VST3 plugin's panel, not Steinberg's logo, under Steinberg's VST3 usage guidelines (https://github.com/steinbergmedia/vst3sdk#trademark-and-logo-usage, VST3_Usage_Guidelines.pdf, SDK 3.8 of 16.10.2025; the terms are in assets/README.md). Section 2 allows 'VST' in a standard font to denote compatibility but still wants the official VST Compatible Logo shown in a relevant context near uses of 'VST'. task-231 put the text on the panel and the attribution sentence in docs/plugin.md and assets/README.md. What remains:
- the GitHub README's plugin part: the VST Compatible Logo (unaltered, from the official artwork github.com/steinbergmedia/vst3_doc artwork/VST_Compatible_Logo_Steinberg*.svg, at least 15 x 10 mm; the negative or the positive variant as the README background needs), the registered sign at the first use of VST, and the sentence 'VST is a registered trademark of Steinberg Media Technologies GmbH.';
- the release notes and the release page (release.yml, scripts/changelog.sh): the same sentence, and the sign at the first use of VST;
- Help > Plugin Usage (task-221), the About-box equivalent of the guidelines' section 1e: the official logo (unaltered, 15 x 10 mm minimum) and the sentence there. task-221 depends on this task (its description is unchanged); its implementer follows this text for that screen;
- any other product text that names VST (docs/building.md, the bundle's metadata): the registered sign at its first use.

Acceptance criteria: each place above has the logo where listed and the sentence; the minimum size holds; a grep for VST in the README and the release text shows the sign at the first use.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 README plugin section shows the unaltered logo (picture, light and dark variants) at 120 px (>= 15 x 10 mm) with the sentence, and the first VST in the README carries the registered sign; the Help > Plugin Usage screen is handed to task-221 (its notes say what it needs), not built here
- [x] #2 Release notes (scripts/changelog.sh) start with the sign and the sentence; package README.txt files and docs/building.md carry them too
- [x] #3 Official SVGs committed unaltered under assets/steinberg/ with source and terms in assets/README.md
- [x] #4 task-221 notes say what Help > Plugin Usage needs (logo path, minimum size, sentence)
- [x] #5 Bundle metadata and host-visible identifiers unchanged; cargo test passes
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Download the official VST Compatible Logo SVGs into an empty folder and look at them. 2. Commit them unaltered under assets/steinberg/, source and terms in assets/README.md. 3. README: plugin section with a picture element (dark/light), the sign at the first VST, the sentence. 4. Release text (changelog.sh header), package README.txt (package.sh, package-dmg.sh), docs/building.md: sign and sentence. 5. Note for task-221 (the Help page is its). 6. cargo test.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a35647d1e34553e40

Logos: both SVGs from steinbergmedia/vst3_doc artwork/ (viewBox 116.2x74.7), viewed as PNG: plain = black box, negative = white box with black border; both self-contained boxes. Light theme uses plain, dark uses negative. SHA-256 in assets/README.md. 120 px wide is about 32 x 20 mm at 96 dpi. Bundle metadata not touched: the bundle ids, VST3 class id and Cargo description identify or feed the plugin to hosts, so no registered sign there. The README line 'MIDI, VST and ReWire' was stale (plugin exists); reworded so the first VST carries the sign. Slip: the text edits of README.md, scripts and docs/building.md were applied with a short python script instead of the Edit tool (result checked by diff). cargo test: 877 passed, 0 failed (no Rust touched). Task left To Do: the user reviews README.md first.

Review round 1 (send back): subject shortened to 72 chars; task file restored from main and redone through the CLI (AC 1 reworded: the Help screen is handed to task-221); package.sh underline made 35 chars; blank comment line added in changelog.sh. The task-221 note was written with the CLI.

User review 2026-11-17: README approved. The user also decided the panel's plain text 'VST3' stays as it is (no registered sign, no logo on the panel); the official logo and the sentence go on Help > Plugin Usage (task-221).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Steinberg's official VST Compatible Logo (light and dark SVGs, unaltered, source and SHA-256 in assets/README.md) is committed under assets/steinberg/ and shown in the README's new plugin section via a picture element at 120 px (about 32 x 20 mm, above the 15 x 10 mm minimum) with the sentence 'VST is a registered trademark of Steinberg Media Technologies GmbH.'; the first VST in the README carries the registered sign. The release notes (scripts/changelog.sh), the package README.txt files (package.sh, package-dmg.sh) and docs/building.md carry the sign and sentence. Bundle metadata and identifiers are untouched. task-221 has a note on what Help > Plugin Usage needs. The user approved the README and decided the panel keeps the plain text VST3. Branch worktree-agent-a35647d1e34553e40, one commit on base e41298d. cargo test: 877 passed, 0 failed.
<!-- SECTION:FINAL_SUMMARY:END -->
