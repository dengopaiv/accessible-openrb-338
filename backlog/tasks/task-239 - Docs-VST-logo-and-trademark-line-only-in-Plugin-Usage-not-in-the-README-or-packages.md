---
id: TASK-239
title: >-
  Docs: VST logo and trademark line only in Plugin Usage, not in the README or
  packages
status: To Do
assignee:
  - '@agent-chore'
created_date: '2026-11-17T17:13:00Z'
labels:
  - docs
  - tools
milestone: m-7
dependencies:
  - TASK-237
references:
  - README.md
  - docs/plugin_usage.md
  - scripts/package.sh
  - scripts/package-dmg.sh
  - scripts/package-pkg.sh
  - scripts/changelog.sh
  - task-233
priority: medium
type: docs
ordinal: 400
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The user's decision (2026-11-17), revising task-233: the Steinberg VST Compatible logo and the sentence "VST is a registered trademark of Steinberg Media Technologies GmbH." stay only in docs/plugin_usage.md (its VST section between the vst3-only markers, which Help > Plugin Usage shows in the VST3 build; crates/orb-ui/src/plugin_usage.rs). Remove them everywhere else that users read:
- README.md: the logo <picture> (around lines 66-71), the trademark sentence after it (line 73), the License-section sentences (lines 169-170), and the registered sign after VST (lines 51, 61: "VST 3" without the sign).
- The package texts: scripts/package.sh (the .zip/.tar.gz README.txt), scripts/package-dmg.sh (the DMG README.txt), scripts/package-pkg.sh (the pkg welcome page), scripts/installer/ and scripts/package-installer.sh (the Windows installer readme) if they carry it, and scripts/changelog.sh (the release notes): drop the sentence and the registered sign.
Keep: assets/steinberg/*.svg (plugin_usage.md uses them) and assets/README.md's source/terms note for them; docs/plugin_usage.md unchanged. Update developer docs that state where the attribution must appear (docs/building.md, docs/plugin.md, assets/README.md) to say it is in Plugin Usage only. Don't touch tests or Rust code unless a test checks the README/package text (grep crates/ for the sentence; the hits in crates/orb-plugin/tests and orb-ui-art are about the panel/test text, check whether they assert on the removed places).
Base on main after task-237 merges (it edits scripts/package.sh, the installer and docs/building.md).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 grep -rn -E 'Steinberg|VST®|registered trademark' README.md scripts/ finds nothing; docs/plugin_usage.md still has the logo and the sentence.
- [ ] #2 Developer docs (docs/building.md, docs/plugin.md, assets/README.md) say the attribution is in Plugin Usage only.
- [ ] #3 cargo test passes; bash -n on changed scripts passes; scripts/changelog.sh runs.
<!-- AC:END -->
