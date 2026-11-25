---
id: TASK-302
title: >-
  Windows CI: orb-plugin exports test aborts in nice-plug's event loop
  (RegisterClassExA)
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-25T10:54:00Z'
updated_date: '2026-11-25T11:46:00Z'
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
- [x] #1 The root cause is found with evidence (GetLastError, a reproduction or the code path) and recorded in the notes, including why it started at 4201d33
- [x] #2 The fix makes the exports test pass on Windows reliably (repeated runs on the test machine), without skipping or weakening it
- [x] #3 If the cause can hit a real host, the notes say so and the fix or a follow-up task covers the plugin itself
- [x] #4 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass; the supervisor's CI run on the pushed fix is green
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Reproduce on the test machine with a diagnostic nice-plug copy (scratch only, never committed: [patch.crates-io] via --config) that logs the class name, atom and GetLastError of RegisterClassExA and the results of DestroyWindow/UnregisterClassA; cross-build the cdylib and the exports test with cargo xwin, run the test there repeatedly with a sleep inside library()'s OnceLock to mimic CI's in-test cargo build.
2. Compare the collision rate of 559e6ab's sources and 4201d33's to answer why it showed at 4201d33.
3. Fix on our side without touching nice-plug: the exports test's host creates VST3 instances one at a time (a process-wide lock around createInstance, the host's single main thread), so two test threads cannot register the event-loop class in the same QueryPerformanceCounter tick. The tests keep running in parallel and check the same things.
4. Verify: repeated runs of the fixed test on the test machine (with and without the sync sleep), cargo test --workspace, clippy macOS and xwin Windows, nightly fmt.
5. Real hosts: record whether a host can hit it; a Decide task for any nice-plug patch/vendor (the supervisor decides with the user).
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a27b95e841b2ef8e7

**CI evidence (run 45596405986, attempts 1 and 2, image windows-2025-vs2026 20261116.315.1, the same image as the passing 45586804316).** The in-test `cargo build -p orb-plugin --lib` finished at 10:41:02.54; identity test ok at 02.65; the panic came at 02.70, in `createInstance` (frame 17 `vst3::...make_vtbl::createInstance<orb_plugin::vst3::Factory>`) of the_vst3_parameters_are_continuous (attempt 1) and of a_note_at_frame_0..., via vst3_first_note (attempt 2): the first instance each test creates, right after the three test threads leave `library()`'s OnceLock together. Exit 0xC0000409. Thread name `<unnamed>`: the panic runs in the cdylib's own std, which does not know the test's thread name.

**Reproduction (test machine, 1 permit of redc-win-run; files under C:\re\rb\task302, removed afterwards).** A copy of nice-plug 0.4.2 in the scratchpad with logging only (class name, hInstance, atom, GetLastError, QueryPerformanceFrequency after RegisterClassExA; DestroyWindow/UnregisterClassA results in Drop), wired in with `cargo xwin ... --config 'patch.crates-io.nice-plug.path=...'` (never committed). The cdylib (dev) and the exports test (`--no-run`, test profile) cross-built with cargo xwin; a temporary env hook in `library()` (ORB_DIAG_LIB, plus ORB_DIAG_SLEEP=2000 ms inside the OnceLock to stand in for CI's in-test build) let it run without cargo; the hook is not in the commit. Scripts: scratchpad wt302/{diag.py,xbuild.sh,remote.sh,batch.sh,gaps.py}.
- QueryPerformanceFrequency = 10,000,000 (100 ns ticks).
- The failing case, caught in run 6 of 30: `register thread=ThreadId(1) class="nice-event-loop-12511349381547" atom=49731` and `register thread=ThreadId(2) class="nice-event-loop-12511349381547" atom=0 err=WIN32_ERROR(1410)` (ERROR_CLASS_ALREADY_EXISTS), then the same `assert_ne!` panic, "panic in a function that cannot unwind", exit 3221226505 = 0xC0000409, as on CI.
- Every Drop on the creating thread: DestroyWindow ok, UnregisterClassA ok (err 0). Hypothesis 2 (classes not unregistered) refuted. Hypothesis 1 (two creations in the same QPC tick) confirmed.
- The two first registrations come within microseconds: the tick gaps are 0..326 (median about 60). The threads are released together by the OnceLock (CI: by the cargo build inside it), then run near-identical paths (load, factory, createInstance) to the QPC read.

**Why it showed at 4201d33 (hypothesis 3).** The race exists since 6fd99b8 (task-275, 2026-11-22) added the_vst3_parameters_are_continuous, the second test that creates an instance in parallel with a_note_at_frame_0...; the 17 Windows CI runs from 6fd99b8 to 559e6ab passed. d810923 changes nothing in the path's logic (the parameter and group names "ORB-303 1" etc. become "303-1"; nice-plug hashes the ids and builds the units from the group names in `WrapperInner::new` before the event loop), but it shifts the timing. Measured, 100 runs each with the 2 s sync and the diagnostic nice-plug, src of 559e6ab (crates/orb-player, orb-plugin, orb-ui src checked out from 559e6ab for the build, then restored) vs 4201d33:
- 559e6ab: 100/100 pass, 0 collisions; tick gaps <= 2: 1, <= 5: 4, median 79.
- 4201d33: 94/100 pass, 6 collisions (all 1410); tick gaps <= 2: 11, <= 5: 16, median 56.
So d810923 made the same race several times likelier on this machine (the timing, not a logic change); CI's two failures in a row at 4201d33 fit that. The race could have hit any earlier run.

**Fix (crates/orb-plugin/tests/exports.rs).** `create_instance()` creates each VST3 instance under one process-wide `Mutex`, as a host creates instances on its main thread; both createInstance call sites use it. The tests still run in parallel; only the creation itself is serialized; nothing is skipped or weakened. After the fix (same diagnostic build, the fixed test): 100/100 pass with the 2 s sync, 30/30 without it, 0 collisions; the first two registrations are 11.3 ms apart at least (gaps 112749..).

**Real hosts (AC 3).** A host can hit it only by creating nice-plug instances (ours or any nice-plug plugin's: same prefix, same exe module handle from GetModuleHandleA(NULL)) on two threads within the same 100 ns. VST3 hosts create instances on the main thread, so REAPER etc. do not; the plugin cannot guard against it from our side (the registration happens inside nice-plug after `Plugin::default()` returns). Any nice-plug patch or vendoring is the user's decision: task-303 (Decide, low, m-5) with the options and the recommendation (accept and document now). docs/plugin.md section 7 records the fact. Not a dependency of this task: the CI fix does not need it.

**Checks.** cargo test --workspace: 1152 passed, 0 failed, 5 ignored. cargo clippy --all-targets -D warnings (macOS) clean; cargo xwin clippy --target x86_64-pc-windows-msvc --all-targets -D warnings clean; cargo +nightly fmt --all --check clean.

Review round 1 (text only; code and criteria accepted): (1) exports.rs create_instance comment: the garbled 'aborted so' is now 'On the test machine, 6 runs in 100 aborted this way.', and the first line no longer states what hosts do. (2) docs/plugin.md section 7 no longer claims hosts are safe: it says the code path and the test machine show the abort needs two threads within the same 100 ns, that no host's behavior was checked, and that task-303 holds the open question. task-303's description now marks 'VST3 hosts create instances on their main thread, so REAPER, Bitwig, Live and the validators do not' as an unverified belief, with checking it part of the decision. The same claim in these notes above ('VST3 hosts create instances on the main thread, so REAPER etc. do not') is corrected here: unverified, see task-303. Rechecked: cargo test -p orb-plugin --test exports, cargo +nightly fmt --all --check.

Review round 1 check: cargo test -p orb-plugin --test exports passes 3/3 on the amended commit. Run before the amend, with the edits uncommitted, the_vst3_factory_has_the_identity failed: its version check compared the test binary's 'dev-8b23745-dirty' with the cdylib's 'dev-8b23745' (a stale cdylib build next to a dirty tree; the existing task-291 check, not this change). On a clean tree it passes.

2026-11-25, supervisor: merged as a74c22c and pushed; CI run 45605823388 on a74c22c passed on Linux, Windows and macOS with no rerun. Criterion 4 checked.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Root cause: nice-plug 0.4.2's Windows event loop, made in every VST3 createInstance, registers a window class named nice-event-loop-{QueryPerformanceCounter ticks} (10 MHz) and asserts RegisterClassExA succeeded. The exports tests create instances on two parallel test threads, released together by library()'s OnceLock (the in-test cargo build); when both read the same 100 ns tick, the second registration fails with ERROR_CLASS_ALREADY_EXISTS (1410), and the assert inside the extern entry aborts (0xC0000409). Reproduced on the test machine with a logging copy of nice-plug (scratch only). The race exists since 6fd99b8 (task-275); d810923 shifted the timing: 6/100 aborts at 4201d33 vs 0/100 with 559e6ab's sources.

Fix: crates/orb-plugin/tests/exports.rs creates instances one at a time under a process-wide lock (create_instance), as a host's main thread does; the tests still run in parallel and check the same things. Test machine after the fix: 130/130 pass (100 with the CI-like sync, 30 without), 0 collisions. docs/plugin.md section 7 records the nice-plug fact. A real host hits it only by creating nice-plug instances on two threads at once; the plugin-side choice (accept, patch or vendor nice-plug, upstream report) is task-303 for the user.

Branch worktree-agent-a27b95e841b2ef8e7, one commit. cargo test --workspace 1152 passed (5 ignored); clippy -D warnings clean on macOS and x86_64-pc-windows-msvc (cargo xwin); nightly fmt --check clean. AC 4's CI run on the pushed fix is the supervisor's, after the merge: left unchecked.
<!-- SECTION:FINAL_SUMMARY:END -->
