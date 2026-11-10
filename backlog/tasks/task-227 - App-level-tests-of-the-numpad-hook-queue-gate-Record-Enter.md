---
id: TASK-227
title: App-level tests of the numpad hook queue (gate, * Record, Enter)
status: To Do
assignee: []
created_date: '2026-11-10T20:38:00Z'
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
- [ ] #1 A test shows that a numpad * pressed while a dialog or the menu bar has the keys does not fire Record when it closes
- [ ] #2 A test shows numpad * presses and releases Record, and numpad Enter plays with a 303 section focused in screen reader mode
- [ ] #3 The tests cannot race with other tests (own process or a lock), and cargo test --release -p orb-ui passes
<!-- AC:END -->
