---
id: TASK-276
title: 'Decide: vendor nice-plug to keep the CLAP range of the continuous knobs'
status: Done
assignee:
  - '@pingw33n'
created_date: '2026-11-22T07:00:00Z'
updated_date: '2026-11-22T08:09:00Z'
labels:
  - decision
  - player
milestone: m-7
dependencies: []
references:
  - docs/plugin.md
  - https://github.com/pingw33n/nice-plug/tree/OpenRB-338
priority: high
type: spike
ordinal: 260700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**As it stood on 2026-11-22 (the options as proposed; option 1 then meant vendoring nice-plug into the repository). The outcome was a GitHub fork instead: see the final summary.**

Context (task-275, GitHub issue 21): the 86 parameters over 0..127 must become continuous for the host (Studio One draws a stepped parameter's automation as steps), and projects saved with the released plugin v2026.1118.0919 must keep their automation and state.

Stock nice-plug 0.4.2 gives a continuous FloatParam the CLAP range 0..1 and CLAP values normalized (src/wrapper/clap/wrapper.rs: step_count().unwrap_or(1) as the factor in 11 places). The released build's stepped IntParams had the CLAP range 0..127. REAPER 7.86 saves CLAP envelope points in the CLAP value ('<PARMENV 27:3029309 0 127 63.5', 'PT 3.5 127'): with stock nice-plug a CLAP project saved with the released build rendered differently in 383586 of 384000 samples (the envelope's points above 1 clamped to the top). VST3 is not affected (normalized values).

Options:
1. Vendor nice-plug 0.4.2 under vendor/nice-plug ([patch.crates-io], as raw-window-metal) with a small patch: clap_scale, the CLAP factor of a FloatParam over a linear 0..=max (max whole, >= 1) is max, so its CLAP range stays 0..127 and its values the released build's, while it is no longer stepped. Cost: a vendored crate (800 KB) to re-patch on each nice-plug update. CLAP projects of the released build render bit for bit the same (checked in REAPER).
2. Stock nice-plug: CLAP continuous with the range 0..1. CLAP host automation saved with the released build breaks in REAPER (and in any CLAP host that stores CLAP values); VST3 and the plugin state are fine.
3. Keep the knobs stepped for CLAP only: not possible without patching nice-plug either (one Params object serves both formats).
4. Upstream the patch to nice-plug (an option for later, together with 1).

Recommendation (then): 1, implemented on task-275's branch at the time, and offer the patch upstream later. The patch now lives in the fork and is documented in docs/plugin.md 'The forked nice-plug'.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user chose an option; the outcome and the date are in the final summary
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user on 2026-11-22: a GitHub fork of nice-plug, neither the vendored copy (option 1) nor stock nice-plug (option 2). The fork is github.com/pingw33n/nice-plug, branch OpenRB-338, commit 250daefd57cdc58d41ddb25f053f1a0589f44cad: upstream 91ba75d (the commit nice-plug 0.4.2, nice-plug-core 0.4.2 and nice-plug-derive 0.1.2 were published from) plus task-275's clap_scale patch, which changes only crates/nice-plug/src/wrapper/clap/wrapper.rs and context.rs. The workspace Cargo.toml [patch.crates-io] takes nice-plug, nice-plug-core and nice-plug-derive from that rev. A new upstream: rebase the branch, push, bump the rev (docs/plugin.md 'The forked nice-plug'). Applied in task-275's commit.
<!-- SECTION:FINAL_SUMMARY:END -->
