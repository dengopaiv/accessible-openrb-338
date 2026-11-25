---
id: TASK-303
title: >-
  Decide: patch nice-plug's Windows event-loop class name against concurrent
  instance creation
status: Done
assignee:
  - '@pingw33n'
created_date: '2026-11-25T11:19:00Z'
updated_date: '2026-11-25T11:51:00Z'
labels:
  - decision
  - player
milestone: m-5
dependencies: []
references:
  - docs/plugin.md
  - crates/orb-plugin/tests/exports.rs
priority: low
type: spike
ordinal: 274700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** task-302 found the cause of the Windows CI abort of `crates/orb-plugin/tests/exports.rs`: nice-plug 0.4.2's Windows event loop, made by every VST3 `createInstance` (`src/wrapper/vst3/inner.rs:388`), registers a hidden window class named `nice-event-loop-{QueryPerformanceCounter ticks}` under the host executable's module handle (`GetModuleHandleA(NULL)`, `src/event_loop/windows.rs:78-94`) and does `assert_ne!(RegisterClassExA(..), 0)`. QueryPerformanceCounter runs at 10 MHz (measured on the test machine and the usual value on Windows 10+). Two threads of one process that create an instance of any nice-plug plugin within the same 100 ns register the same name; the second gets ERROR_CLASS_ALREADY_EXISTS (1410, measured with a diagnostic nice-plug on the test machine) and the panic inside the `extern "system"` entry aborts the host (0xC0000409). task-302 fixed the test (its host creates instances one at a time, under one lock); the plugin itself is unchanged.

**Who can hit it:** only a host that creates nice-plug instances (ours or another nice-plug plugin's, since every one uses the same name prefix and the exe's module handle) on two threads at once. Which hosts create instances on more than one thread was not checked (task-302 checked no host): the belief that VST3 hosts create instances on their main (UI) thread, so REAPER, Bitwig, Live and the validators would not hit it, is unverified; checking it (host documentation or a test in each host) is part of this decision. A host that scans or creates plugins on several threads in one process could, rarely (both calls inside one 100 ns window). No user report exists.

**Options:**
1. Accept: document it (docs/plugin.md section 7, done in task-302), act only on a report.
2. Patch nice-plug through `[patch.crates-io]` with a vendored copy (or a git fork) of nice-plug 0.4.2: make the class name unique per process (a process-wide atomic counter, or the module address plus the counter, in addition to the ticks), and retry or return an error instead of the assert. A small diff (about 10 lines in `src/event_loop/windows.rs`), but a vendored crate to keep in step with each nice-plug update.
3. An upstream report (no pull request, per the user's rule): would fix it at the source with no vendoring.

**Recommendation:** 1 now, unless the host check finds a host that creates instances on several threads, with 3 if the user agrees to filing an issue; 2 only if a host is seen to abort. The plugin cannot work around it on its own side: the registration happens inside nice-plug after `Plugin::default()` returns, so no lock of ours can cover it.

**Acceptance criteria:** the user's choice is recorded in the final summary with the date; if it is 2, a follow-up task to apply it exists.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user's choice (accept, vendor/patch, or upstream report) is recorded in the final summary with the date
- [x] #2 If the choice is a patch, a follow-up task to apply it exists
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user on 2026-11-25: option 1, accept. The race (two threads creating nice-plug instances within one 100 ns QueryPerformanceCounter tick register the same window class and abort) stays documented in docs/plugin.md section 7 (task-302); the test side is fixed (exports.rs creates instances one at a time). No patch or vendored nice-plug, no upstream report, and no host check now; act only on a user report. Criterion 2 needs nothing (no patch chosen).
<!-- SECTION:FINAL_SUMMARY:END -->
