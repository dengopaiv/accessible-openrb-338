---
id: TASK-232
title: 'Decide: the VST3 logo on the plugin panel (Steinberg''s trademark terms)'
status: Done
assignee:
  - '@pingw33n'
created_date: '2026-11-17T09:12:00Z'
updated_date: '2026-11-17T09:27:00Z'
labels:
  - decision
  - ui
  - docs
milestone: m-7
dependencies: []
references:
  - assets/README.md
  - crates/orb-ui-art/src/plugin_logos.rs
  - task-231
priority: high
type: spike
ordinal: 233000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Context: task-231 puts the plugin format's logo on the plugin's transport (the VST3 logo in the VST3 build, the CLAP logo in the CLAP build). The CLAP logo is MIT licensed (Alexandre Bique; Wikimedia Commons). The VST3 logo is a vector redraw by Olivier Clero (https://www.olivierclero.com/art/vst3-logo/), whose author allows its use in a VST plugin without credit; but the VST logo and name are a trademark of Steinberg Media Technologies GmbH, with their own logo usage guidelines (which the agent could not confirm allow this use: they may require a license agreement or a trademark notice). The OpenRB-338 repository is public.

Options:
1. Keep the logo and add Steinberg's required trademark notice (about box, README), after checking their current VST3 logo usage guidelines.
2. Replace it by plain engraved text "VST3" (no logo, no trademark artwork; the same engraving style).
3. Drop the VST3 logo (the CLAP build keeps its logo; the VST3 build shows nothing there).

Recommendation: 2 unless the guidelines clearly allow the logo (then 1). Until decided the logo stays in the code as the user approved it in their mockup; task-231 depends on this decision before merge.
<!-- SECTION:DESCRIPTION:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user 2026-11-17, in two steps. First: use Steinberg's official VST Compatible Logo, unaltered. Then, after seeing it on the panel (it does not fit), the final decision: the VST3 plugin's panel shows the plain text 'VST3' in the panel's label font, engraved like the CLAP logo (guidelines section 2 allow 'VST' in a standard font to denote compatibility); no VST logo on the panel and no Olivier Clero redraw. Sources: https://github.com/steinbergmedia/vst3sdk#trademark-and-logo-usage, VST3_Usage_Guidelines.pdf (SDK 3.8 of 16.10.2025): logo unaltered in the About box or equivalent (1e), 15 x 10 mm minimum (14), registered sign at the first use of VST and the sentence 'VST is a registered trademark of Steinberg Media Technologies GmbH.' in credits and documentation (15); artwork github.com/steinbergmedia/vst3_doc artwork/. Applied in task-231 (docs/plugin.md, assets/README.md). The remaining obligations (the official logo near uses of VST in the README and Help > Plugin Usage, the attribution in release notes) are task-233.
<!-- SECTION:FINAL_SUMMARY:END -->
