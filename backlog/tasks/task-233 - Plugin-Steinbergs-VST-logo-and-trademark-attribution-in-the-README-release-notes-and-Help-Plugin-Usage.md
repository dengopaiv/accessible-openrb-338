---
id: TASK-233
title: >-
  Plugin: Steinberg's VST logo and trademark attribution in the README, release
  notes and Help > Plugin Usage
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-17T09:25:00Z'
updated_date: '2026-11-17T09:27:00Z'
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
