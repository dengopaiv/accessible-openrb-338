---
id: TASK-214
title: >-
  Plugin: a parameter change before the first engine block after an activation
  is lost
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-16T11:41:00Z'
updated_date: '2026-11-17T08:07:00Z'
labels:
  - engine
  - player
milestone: m-7
dependencies: []
references:
  - crates/orb-plugin/src/core.rs
  - crates/orb-plugin/src/instance.rs
  - crates/orb-plugin/tests/editor.rs
priority: low
type: bug
ordinal: 223000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Found in task-201 (tests/editor.rs, a_knob_sets_its_parameter_through_the_host). An activation (instance::core -> OrbParams::activate) queues Command::Load of the engine built with the parameter values of that moment. Core::process_with applies a changed parameter right after the call's first frame through Audio::apply_now, but the first engine block is rendered only when its first frame is due (64 frames later, docs/plugin.md section 4); its Audio::drain then applies the queued Load, which replaces the engine with the activation's values. Core::apply has already recorded the new value in its last array, so it is never sent again: the engine plays the old value while the host parameter holds the new one, until the parameter changes again or the next activation.

Trigger: a host or editor parameter change between an activation and the end of the first 64 host frames. Rare in practice, but an automation lane starting at the transport start of an offline bounce could hit it.

Fix to evaluate: drain the activation's Load before the first call (in Core::new, on the main thread, since nothing else can be queued yet), or re-send every parameter whose value the engine does not hold after a Load is drained. Check tests/timing.rs and the bit-exact tests stay as they are.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A parameter changed in the first host buffer after an activation is played by the engine (a test in crates/orb-plugin/tests)
- [x] #2 cargo test -p orb-plugin passes; the timing and sync tests are unchanged
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Verify task-226's fix (core::settle in Core::new drains the activation's Load on the main thread) covers task-214: a new test crates/orb-plugin/tests/first_change.rs activates an instance, changes the 303 #1 cutoff (d1c3) at frames 0, 10 and 63 of the first host buffer, at 44.1 and 48 kHz, and checks the engine holds the new value after the first call and later, and that the output equals an instance activated with the new value (where bit-exact applies), and differs from the old value.
2. Run the test on core.rs of af012b8 (before task-226) to show it fails there, and on the current code to show it passes.
3. If it fails now, find and fix the cause in core.rs narrowly (task-225 works in core.rs at the same time).
4. cargo test, clippy -D warnings, nightly fmt --check; if plugin code changes, bundles + scripts/validate-plugin.sh.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a5709292acb232c76 (worktree .claude/worktrees/agent-a5709292acb232c76), based on 13b61ee, rebased onto main 0c6ba11 before the commit.

Verification (task-226's fix, core::settle in Core::new at crates/orb-plugin/src/core.rs, drains the activation's Command::Load and renders 256 silent blocks on the main thread):
New test crates/orb-plugin/tests/first_change.rs a_change_in_the_first_buffer_after_an_activation_is_played: song 303_accent0, 303 #1 cutoff d1c3 50 -> 100 set at host frame 0 (before the first call), 10 and 63 (the first call split there, as nice-plug's sample-accurate automation does), at 44.1 and 48 kHz, buffers of 512. Checks: the engine's cutoff byte (params::engine_byte) is 100 after the first call and after 32768 frames; the output from frame 24576 on equals bit for bit an instance activated with 100 (and an instance left at 50 differs there; reference peak > 0.1).
Not bit-exact from the start, by design: the changed instance's 303 cutoff smoother (0x1012470C, docs/initial_investigation.md 'cutoff smoother') settled at the old value's fixed point and moves to the new one as any cutoff change while playing does. Measured (scratch diag, removed): difference to the reference per 4096-frame window 0.092, 0.0070, 0.0016, 0.0013, 3.1e-5, then 0; the last differing frame 19016 at 44.1 kHz and 20767 at 48 kHz. Frames 10 and 63 give the same output (both reach engine block 1, the first block at or after them, task-227's rule).
Before task-226 (the same test in a detached worktree of af012b8): FAILS, '44100 at 0: the engine holds it after the call', left 50 right 100: the change went to the placeholder song and the activation's Load replaced it. On main 0c6ba11 (13b61ee's code): PASSES. So task-226 fixed the cause; no plugin source change here (no bundle rebuild needed), tests/timing.rs and the sync tests untouched.

Checks after the rebase onto main 0c6ba11: cargo test 858 passed 0 failed (856 + ... incl. the new test); cargo clippy --all-targets -D warnings clean; cargo +nightly fmt --all --check clean (after formatting the new file). No plugin source changed, so the bundles were not rebuilt and validate-plugin.sh was not needed. Reproduce: cargo test -p orb-plugin --test first_change.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Verified: task-226's fix (core::settle in Core::new, crates/orb-plugin/src/core.rs: the activation's Command::Load is drained on the main thread before the first call) removes task-214's cause. A parameter change in the first host buffer after an activation now reaches the engine that plays.
Test: crates/orb-plugin/tests/first_change.rs a_change_in_the_first_buffer_after_an_activation_is_played. The 303 #1 cutoff (d1c3) goes from 50 to 100 at host frames 0, 10 and 63 of the first call, at 44.1 and 48 kHz. The engine's byte is 100 after the first call and at the end. From frame 24576 on the output equals, bit for bit, an instance activated with 100. Before that point the 303 cutoff smoother (0x1012470C) is still moving from the old value's fixed point; the last differing frame is 19016 at 44.1 kHz and 20767 at 48 kHz.
On af012b8 (before task-226) the test fails: the engine keeps 50 after the first call. On main 0c6ba11 it passes. No plugin source change; tests/timing.rs and the sync tests are untouched.
Checks: cargo test 858 passed 0 failed, clippy -D warnings clean, nightly fmt clean. Branch worktree-agent-a5709292acb232c76, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
