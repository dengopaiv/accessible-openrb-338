---
id: TASK-227
title: App-level tests of the numpad hook queue (gate, * Record, Enter)
status: Done
assignee: []
created_date: '2026-11-10T20:38:00Z'
updated_date: '2026-11-28T13:10:00Z'
labels:
  - ui
  - tests
milestone: m-7
dependencies:
  - TASK-225
references:
  - crates/orb-ui/src/numpad.rs
priority: low
type: task
ordinal: 245000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: task-225's review (2026-11-10) found no app-level test of the numpad hook's queue (crates/orb-ui/src/numpad.rs): that App::show clears unclaimed entries every frame even when App::keyboard is gated (browser, prompt, menu bar, text field), that numpad * fires Record press/release through Insert's path, and that numpad Enter plays with a 303's section focused in screen reader mode. The queue is a process-global static and the tests of one integration binary run in parallel threads, so such tests race with other tests' end_frame. Put them in their own integration test file (its own process) or serialize them with a mutex, feeding synthetic hook entries the way numpad.rs's unit tests do.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A test shows that a numpad * pressed while a dialog or the menu bar has the keys does not fire Record when it closes
- [x] #2 A test shows numpad * presses and releases Record, and numpad Enter plays with a 303 section focused in screen reader mode
- [x] #3 The tests cannot race with other tests (own process or a lock), and cargo test --release -p orb-ui passes
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. New integration test file crates/orb-ui/tests/numpad_queue.rs (own process, static Mutex serializes), feeding the hook's entries through the public numpad::note_message (no seam needed). 2. Tests: * is Record as Insert (twin app); * pressed under the file browser / the menu bar does not fire Record after close; a left entry does not tag a later key; numpad Enter plays with a 303 section focused in SR mode while main Enter is the 303 Step. 3. cargo test --release --workspace, clippy, fmt.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a8d619c1a53767618

Tests: crates/orb-ui/tests/numpad_queue.rs, own process plus a static Mutex (lock() also clears the queue). No seam needed: numpad::note_message is already pub, so the tests feed the hook's entries as the hook does. 5 tests: star_is_record_as_insert_is (twin app with Insert gives the same events; release only lets go; next tap toggles off); star_pressed_while_a_dialog_has_the_keys_does_not_fire_later (in-app browser open, * press/release, Escape, no Record after close, a new press records); same for the menu bar (F10); an_entry_left_in_a_gated_frame_does_not_tag_a_later_key (numpad - entry gone after a gated frame); numpad_enter_plays_with_a_303_section_focused_in_screen_reader_mode (main Enter = 303 Step, hooked numpad Enter = Play). No bug found; all pass first run.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added crates/orb-ui/tests/numpad_queue.rs: 5 app-level tests of the numpad hook queue (gate for browser and menu bar, numpad * as Record via Insert's path, numpad Enter as Play with a 303 section focused in screen reader mode), in their own test binary with a mutex. No code change, no bug found. cargo test --release --workspace: 1248 passed, 0 failed, 9 ignored; clippy -D warnings and nightly fmt --check clean.
<!-- SECTION:FINAL_SUMMARY:END -->
