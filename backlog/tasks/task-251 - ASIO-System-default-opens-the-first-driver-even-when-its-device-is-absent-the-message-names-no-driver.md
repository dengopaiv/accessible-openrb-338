---
id: TASK-251
title: >-
  ASIO: System default opens the first driver even when its device is absent;
  the message names no driver
status: To Do
assignee: []
created_date: '2026-11-28T10:56:00Z'
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

The cause, found by the supervisor on the user's PC (this development machine): the ASIO drivers installed (HKLM\SOFTWARE\ASIO) are, in this order, "MiniFuse ASIO Driver", "Realtek ASIO" and "ReaRoute ASIO (x64)". No MiniFuse (an Arturia USB interface) is connected; the audio endpoints present are Realtek(R) Audio and AB13X USB Audio. ASIO has no default device: cpal's ASIO host returns the first driver (cpal-0.18.2 src/host/asio/mod.rs, default_output_device), and devices::default_device does the same (crates/orb-player/src/devices.rs, "for ASIO, which has none, the first"). So System default on ASIO loaded the MiniFuse driver, whose hardware is absent; building the stream returned ASIO's ASE_NoClock, whose text is "sample clock or rate cannot be determined or is not present" (asio-sys-0.4.0 src/bindings/errors.rs). ReaRoute also fails unless REAPER runs. Realtek ASIO is the one that can open here.

What to build:
1. On ASIO with Device at System default, open the first driver, in the driver's order, that opens; a driver that refuses is skipped. The In use lines and the Device box show the driver actually in use. If none opens, the existing fallback (WASAPI, System default, default settings) applies. Loading ASIO drivers one after another must respect what docs/building.md "ASIO on Windows x86_64" and devices.rs say about ASIO (one driver loaded at a time in asio-sys, DriverHold, TerminateProcess at exit); say in the notes how it does. If trying the drivers in turn turns out unsafe or slow (a driver that hangs, a control panel that pops up), stop and record it in the notes and propose instead that the window choose a working driver itself; do not decide that alone, it becomes a Decide: task for @dengopaiv.
2. A message about an ASIO driver that did not open names the driver and says in plain words what happened, and then what the program did, for example: "Audio output: MiniFuse ASIO Driver did not open (sample clock or rate cannot be determined or is not present; is its device connected?). Using Realtek ASIO." or, when nothing on ASIO opens, "... Back on WASAPI, System default." Spoken once (it is an error event; nothing is spoken twice).
3. A named ASIO device that refuses gets the same kind of message (its name, the driver's reason, what the program did).
4. After any fallback, the Driver and Device boxes read what is in use (as task-245 did for a lost device), so NVDA does not read ASIO while WASAPI plays.

Tests: the choice of driver as a unit with a fake opener (the order, a refusal skipped, all refused gives the WASAPI fallback, the message texts); no test may need ASIO hardware. The agent may check by hand on this machine with the release build that switching to ASIO now opens Realtek ASIO (note the result), keeping the volume low.

Docs: docs/accessibility.md (Audio Output), docs/accessibility_plan.md (Part 1.2, 2.1, and a Part 3 checklist item), docs/building.md's ASIO section if the driver choice is described there. Add the by-ear check to TASK-244 as a new criterion naming the checklist item.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 On ASIO with System default, the first driver that opens is used and one that refuses is skipped; when none opens, the WASAPI fallback applies (unit tests with a fake opener; no ASIO hardware needed)
- [ ] #2 A message about an ASIO driver or device that did not open names it, gives the driver's reason in plain words and says what the program does instead (tests of the texts)
- [ ] #3 After a fallback the Driver and Device boxes and the In use lines read what is in use (a test)
- [ ] #4 The notes say how loading drivers in turn respects asio-sys's one-driver rule and DriverHold, and the result of a by-hand check on this machine if one was made
- [ ] #5 docs/accessibility.md and docs/accessibility_plan.md (Part 1.2, 2.1, a new Part 3 item) describe it; TASK-244 has the by-ear check as a new criterion naming the checklist item
- [ ] #6 cargo test, clippy -D warnings and nightly fmt --check pass
<!-- AC:END -->
