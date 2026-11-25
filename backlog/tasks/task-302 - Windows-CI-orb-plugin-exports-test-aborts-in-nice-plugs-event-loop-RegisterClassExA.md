---
id: TASK-302
title: >-
  Windows CI: orb-plugin exports test aborts in nice-plug's event loop
  (RegisterClassExA)
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-25T10:54:00Z'
labels:
  - tests
  - player
milestone: m-5
dependencies: []
references:
  - crates/orb-plugin/tests/exports.rs
  - .github/workflows/ci.yml
priority: high
type: bug
ordinal: 100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** main 4201d33 fails CI on windows-latest twice in a row: run 45596405986, attempts 1 and 2. The previous run, 559e6ab (run 45586804316), passed, and so did every run before it back to 31affca. The range 559e6ab..4201d33 is task-293 (d810923, the 303-1/303-2 rename: name strings, comments and tests) plus two backlog commits. Its crates/orb-plugin/tests/exports.rs change is a comment only.

**The failure:** in `cargo test --workspace --locked --no-fail-fast`, test binary tests/exports.rs (3 tests). After "the_vst3_factory_has_the_identity ... ok", a spawned thread panics:

    nice-plug-0.4.2/src/event_loop/windows.rs:94:9: assertion left != right failed (left: 0, right: 0)
    ... panic in a function that cannot unwind

That is `assert_ne!(RegisterClassExA(&class), 0)`. The class name is `nice-event-loop-{QueryPerformanceCounter ticks}` (windows.rs:78-81). The panic is inside an extern "C" entry, so the process aborts. The exports test builds the plugin (dev profile) and loads it; read the test for exactly how.

**Hypotheses to check, not conclusions:**
1. Two instances or event loops created at the same QPC tick in parallel test threads, so the same class name is registered twice.
2. A class not unregistered, or an instance not dropped between tests in the process. Drop does UnregisterClassA at windows.rs:187-193.
3. Something in d810923 that changes how many instances the exports tests create, or when.

Get GetLastError for the failing RegisterClassExA, for example by reproducing on the Windows test machine (doc-001: rcs_client.py and the permits). Find the real cause before fixing anything.

**Constraints:**
- No upstream pull request to nice-plug. Patch our side, or vendor a fork only with the supervisor's agreement.
- Don't just serialize or skip the test without understanding why it started now. A real host can hit the same abort if it is a QPC collision.
- Agents never push. The supervisor verifies with a CI run.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The root cause is found with evidence (GetLastError, a reproduction or the code path) and recorded in the notes, including why it started at 4201d33
- [ ] #2 The fix makes the exports test pass on Windows reliably (repeated runs on the test machine), without skipping or weakening it
- [ ] #3 If the cause can hit a real host, the notes say so and the fix or a follow-up task covers the plugin itself
- [ ] #4 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass; the supervisor's CI run on the pushed fix is green
<!-- AC:END -->
