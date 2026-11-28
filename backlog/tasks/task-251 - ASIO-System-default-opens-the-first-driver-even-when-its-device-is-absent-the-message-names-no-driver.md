---
id: TASK-251
title: >-
  ASIO: System default opens the first driver even when its device is absent;
  the message names no driver
status: Done
assignee: []
created_date: '2026-11-28T10:56:00Z'
updated_date: '2026-11-28T12:23:00Z'
labels:
  - player
  - ui
milestone: m-7
dependencies: []
references:
  - crates/orb-player/src/devices.rs
  - crates/orb-player/src/control.rs
  - crates/orb-player/src/output.rs
  - crates/orb-ui/src/audio_window.rs
  - docs/accessibility_plan.md
priority: medium
type: bug
ordinal: 268000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: the user's NVDA pass of 2026-11-28 (docs/accessibility_plan.md Part 3, item 31). In Options > Audio Output the user set Driver to ASIO, with Device left at System default, and NVDA read: "Audio output: the output stream: sample clock or rate cannot be determined or is not present; using the System default with the default settings". The program went back to WASAPI on the system default device, as open_chain is built to do, but the user got no sound on ASIO and a message that does not say which driver failed or why.

The cause (corrected by the by-hand check of this task; the supervisor's first reading was wrong): the ASIO drivers installed on the user's PC (HKLM\SOFTWARE\ASIO) are "MiniFuse ASIO Driver" (no device connected), "Realtek ASIO" and "ReaRoute ASIO (x64)", but cpal 0.18.2 lists only "ReaRoute ASIO (x64)" there; MiniFuse and Realtek ASIO are not enumerated (cpal's Devices::next skips a driver that fails to load, init, or report channels or a buffer range; the exact reason is not yet known). ASIO has no default device: cpal's ASIO host and devices::default_device return the first listed driver, which is ReaRoute. ReaRoute works only while REAPER runs; with REAPER off, building the stream returned ASIO's ASE_NoClock, whose text is "sample clock or rate cannot be determined or is not present" (asio-sys-0.4.0 src/bindings/errors.rs). So the user's message came from ReaRoute, not MiniFuse, and Realtek ASIO is never tried on this PC.

What to build:
1. On ASIO with Device at System default, open the first driver, in the driver's order, that opens; a driver that refuses is skipped. The In use lines and the Device box show the driver actually in use. If none opens, the existing fallback (WASAPI, System default, default settings) applies. Loading ASIO drivers one after another must respect what docs/building.md "ASIO on Windows x86_64" and devices.rs say about ASIO (one driver loaded at a time in asio-sys, DriverHold, TerminateProcess at exit); say in the notes how it does. If trying the drivers in turn turns out unsafe or slow (a driver that hangs, a control panel that pops up), stop and record it in the notes and propose instead that the window choose a working driver itself; do not decide that alone, it becomes a Decide: task for @dengopaiv.
2. A message about an ASIO driver that did not open names the driver and says in plain words what happened, and then what the program did, for example: "Audio output: ReaRoute ASIO (x64) did not open (sample clock or rate cannot be determined or is not present; is its device connected, or the program it needs running?). Back on WASAPI, System default." or, after a skip to another driver, "... Using NAME." Spoken once (it is an error event; nothing is spoken twice).
3. A named ASIO device that refuses gets the same kind of message (its name, the driver's reason, what the program did).
4. After any fallback, the Driver and Device boxes read what is in use (as task-245 did for a lost device), so NVDA does not read ASIO while WASAPI plays.

Tests: the choice of driver as a unit with a fake opener (the order, a refusal skipped, all refused gives the WASAPI fallback, the message texts); no test may need ASIO hardware. The agent may check by hand on this machine with the release build (note the result), keeping the volume low.

Docs: docs/accessibility.md (Audio Output), docs/accessibility_plan.md (Part 1.2, 2.1, and a Part 3 checklist item), docs/building.md's ASIO section if the driver choice is described there. Add the by-ear check to TASK-244 as a new criterion naming the checklist item.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 On ASIO with System default, the first driver that opens is used and one that refuses is skipped; when none opens, the WASAPI fallback applies (unit tests with a fake opener; no ASIO hardware needed)
- [x] #2 A message about an ASIO driver or device that did not open names it, gives the driver's reason in plain words and says what the program does instead (tests of the texts)
- [x] #3 After a fallback the Driver and Device boxes and the In use lines read what is in use (a test)
- [x] #4 The notes say how loading drivers in turn respects asio-sys's one-driver rule and DriverHold, and the result of a by-hand check on this machine if one was made
- [x] #5 docs/accessibility.md and docs/accessibility_plan.md (Part 1.2, 2.1, a new Part 3 item) describe it; TASK-244 has the by-ear check as a new criterion naming the checklist item
- [x] #6 cargo test, clippy -D warnings and nightly fmt --check pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. control.rs: on ASIO with Device at System default, open_chain lists the ASIO devices and tries each as a named request in turn (try_in_turn, pure, tested with a fake opener); a refusal is skipped, one error event names the drivers and reasons and says what was done (refusal_text, plain_reason). 2. A named ASIO device that refuses gets the same text, then the existing fallback to the default request. 3. The controller's request becomes the named driver after a skip, so the window follows (task-245 way); test in a11y_windows. 4. Docs, checklist item 40, TASK-244 criterion. 5. By-hand check with the release build.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a88e65782ec9bde73

Design: open_chain (control.rs) on ASIO with Device at System default lists the ASIO devices (devices::list_devices, the driver order cpal enumerates) and opens each as a named request through try_in_turn; a refusal is skipped. The first that opens runs; when any was skipped the controller's request becomes that named driver, so the window's Driver and Device boxes and the In use lines show it by task-245's in_use_choice path and the saved choice stays System default (test audio_output_shows_the_asio_driver_that_opened_after_a_refusal). When none opens, or a named ASIO device refuses, the existing fallback (default request, WASAPI) runs with the message from refusal_text: 'Audio output: NAME did not open (REASON). Back on WASAPI, System default.' One Event::Error, so spoken once. plain_reason drops the 'the output stream: ' prefix and adds '; is its device connected?' to ASIO's no-clock text. Unit tests (control.rs, fake opener): order, refusal skipped, stop at first, all refused, texts.
One-driver rule: try_open -> output::open drops the cpal device and stream on a refusal before returning, so the refused driver is unloaded before the next try loads another; a successful stream takes a DriverHold as before. Each try re-enumerates (all_devices, nothing held), which loads each driver briefly in turn as the window already does. Nothing hangs or pops up here; the driver loads were quick (whole by-hand run 6.7 s, incl. a 2 s hold and the WASAPI fallback). No stop-and-propose needed.
By hand on this PC (release build, ignored test fallbacks.rs asio_system_default_opens_the_first_driver_that_opens, engine idle so silent; no program left running): ASIO devices listed by cpal 0.18.2: only 'ReaRoute ASIO (x64)'. MiniFuse ASIO Driver and Realtek ASIO are NOT in the enumeration (cpal's Devices::next silently skips a driver that fails to load, init or report channels/buffer range), so neither is offered or tried. ReaRoute (REAPER not running) refused with no-clock; result: Error 'Audio output: ReaRoute ASIO (x64) did not open (sample clock or rate cannot be determined or is not present; is its device connected?). Back on WASAPI, System default.'; request = default, in use WASAPI / Speakers (AB13X USB Audio). So the supervisor's expectation that Realtek ASIO opens here was NOT confirmed: it never reaches the list. Why cpal skips it is unknown (not investigated; would need a probe of load_driver/channels on that driver). A driver skipped by enumeration is invisible to the user and to this fix.

Checks: cargo test --release --workspace 1236 pass, 0 fail, 8 ignored; clippy -D warnings clean; nightly fmt --check clean. AC 1/2: control.rs tests (try_in_turn x3, refusal_text). AC 3: a11y_windows audio_output_shows_the_asio_driver_that_opened_after_a_refusal. AC 4: notes above. AC 5: docs/accessibility.md 3.6, accessibility_plan.md 1.2, 2.1, 2.3, item 40, building.md; TASK-244 has a new criterion for item 40. Limits: a named-ASIO-device refusal uses the same refusal_text but has no controller-level test (needs a device); no driver other than ReaRoute is enumerated on this PC, so the skip-and-use-next path ran only with the fake opener.

Review round 1 (send back), all fixed in the amended commit: (1) audio_choose now saves a pick of ASIO at System default as ASIO + no device (choice_to_save in audio_window.rs; rate and buffer from what ran), whichever driver opened or none did; the window shows what is in use through in_use_choice; other picks keep task-234's rule. Tests: a11y_windows audio_output_shows_the_asio_driver_that_opened_after_a_refusal and an_asio_pick_that_falls_back_is_saved_as_the_pick, both through audio_choose. (2) Seam: Controller::for_test_opener(request, list, opener) fakes the ASIO list and output::open, so tests drive the real open_chain: won skip (request becomes the named driver, one error), first opens (no event, request kept), all refused (default request, one error), default refused too, named device refused, host error and empty list (control.rs tests). (3) subject shortened. (4) Docs corrected: the cause was ReaRoute (REAPER off), not MiniFuse; Realtek and MiniFuse are not listed by cpal here, cause open (likely cpal Devices::next skipping a driver on a load/channels/buffer-size error; to be a follow-up task by the supervisor); the description's cause paragraph rewritten. (5) Part 4 ground rules list now names this bug fix as applying with the mode off. (6) Hint is now 'is its device connected, or the program it needs running?'. (7) A host error from listing is reported in the message instead of 'no ASIO driver is installed'.

After review round 1: cargo test --release --workspace 1243 pass, 0 fail, 8 ignored; clippy -D warnings and nightly fmt --check clean. (The seam's log uses rtrb: orb-player's no_alloc test forbids Mutex and mpsc in its sources.)
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
System default on ASIO tries the listed ASIO drivers in turn (control.rs try_in_turn), skips a refusing one and says once, naming the driver and its reason, what was done ('Using X.' or 'Back on WASAPI, System default.'); a named ASIO device that refuses gets the same text. The Driver and Device boxes and In use read what is in use. A pick of ASIO at System default is saved as that (audio_window.rs choice_to_save), so every start tries the drivers in turn; other picks keep task-234's rule. Tests drive the real open_chain through a seam (Controller::for_test_opener): 1243 pass workspace-wide. By hand: cpal lists only ReaRoute ASIO (x64) on the user's PC; with REAPER off it refused and the program went back to WASAPI with the new message. MiniFuse and Realtek ASIO are not listed by cpal (cause open; follow-up for the supervisor). Branch worktree-agent-a88e65782ec9bde73, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
