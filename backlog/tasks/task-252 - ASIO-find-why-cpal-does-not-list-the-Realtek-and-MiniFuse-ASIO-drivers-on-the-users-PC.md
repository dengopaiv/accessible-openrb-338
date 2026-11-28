---
id: TASK-252
title: >-
  ASIO: find why cpal does not list the Realtek and MiniFuse ASIO drivers on the
  user's PC
status: Done
assignee: []
created_date: '2026-11-28T12:31:00Z'
updated_date: '2026-11-28T12:50:00Z'
labels:
  - player
milestone: m-7
dependencies:
  - TASK-251
references:
  - crates/orb-player/src/devices.rs
  - docs/building.md
priority: low
type: spike
ordinal: 269000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: TASK-251's by-hand check (2026-11-28) found that cpal 0.18.2 lists only "ReaRoute ASIO (x64)" on the user's PC, although HKLM\SOFTWARE\ASIO has three drivers: "MiniFuse ASIO Driver" (no MiniFuse connected), "Realtek ASIO" and "ReaRoute ASIO (x64)". A driver cpal does not list is invisible to the user in Options > Audio Output and to TASK-251's try-in-turn, so the user cannot choose Realtek ASIO, which may well work here.

The likely cause, not checked: cpal's ASIO Devices iterator (cpal-0.18.2 src/host/asio/device.rs, Devices::next) skips a driver silently when loading it, initializing it, or asking its channels or buffer size fails. MiniFuse is expected to fail without its hardware. Realtek ASIO failing is the question: it may need the Realtek device not to be in use by WASAPI (NVDA speaks through it), an exclusive mode, or a newer Realtek ASIO driver.

What to do: on this PC (the user's, also the dev machine) load each driver through asio-sys directly in a small ignored test or example, log which step fails and the ASIO error, and record it in the notes and in docs/building.md's ASIO section. If a cpal or asio-sys change would list a driver that can actually open, propose it (a follow-up task, or a Decide: task for @dengopaiv if it means patching cpal). Keep the volume low, play nothing longer than a few seconds, and leave no program running. Respect the one-driver-at-a-time rule (docs/building.md "ASIO on Windows x86_64").
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The notes say, for each of the three drivers, at which step cpal's listing drops it or keeps it, with the ASIO error, from a run on the user's PC
- [x] #2 docs/building.md's ASIO section states what was found and what a user can do (for example, which driver works here and under what condition)
- [x] #3 Any change to list more drivers is proposed as a task (or a Decide: task), not made in this spike
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Ignored test crates/orb-player/tests/asio_probe.rs loads each driver through asio-sys, one at a time, logging load/init, channels, rate, buffer range, sample types, can_sample_rate, then set rate, prepare buffers, optional start. 2. Read cpal 0.18.2 Devices::next, HostTrait::output_devices (supports_output filter) and stream.rs rate check to say where each driver drops. 3. Record in notes and docs/building.md. 4. A cpal change goes into a Decide: task, not made.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a8f40696c1e43d78f

Run on the user's PC, 2026-11-28 (cargo test --release -p orb-player --test asio_probe -- --ignored --nocapture; also with ORB_ASIO_PROBE_START=1: ASIOStart 1.5 s on the driver's own unwritten buffers, no program audio; no process left running).
MiniFuse ASIO Driver: load_driver fails in ASIOInit, InitializationFailed(NoDrivers) = ASE_NotPresent (no MiniFuse connected). cpal Devices::next: Err(_) => continue, silent skip.
Realtek ASIO: loads and inits; channels 2 in/2 out; sample_rate Ok(0.0) (also while running); buffersize 1024..1024; Int24LSB; can_sample_rate(44100/48000/88200/96000) = Err(NoDrivers) (ASE_NotPresent) every time; set_sample_rate(44100) = Err(NoDrivers); prepare_output_stream(2 ch) ok; start/stop ok. cpal Devices::next KEEPS it (it accepts rate 0 with a comment naming Realtek), but its filter .filter(|r| driver.can_sample_rate(r).unwrap_or(false)) leaves supported_sample_rates empty, configs_for returns no configs, and HostTrait::output_devices filters it with supports_output (traits.rs 174). So it is dropped after Devices::next, not by a load or init error (the task's guess about Devices::next was half right). Even if listed, stream.rs ~1150 compares the requested rate with driver.sample_rate() (0.0), calls can_sample_rate and fails: a listing patch alone would not make it play. Not tried: whether the driver answers differently with NVDA's WASAPI output off the device (it would silence NVDA); I cannot confirm that is the cause.
ReaRoute ASIO (x64): loads; 16/16; 48000; 1024; Int32LSB; all four rates Ok(true); listed. With REAPER off it prepares and starts in the probe; the earlier no-clock refusal comes from cpal building the real stream.
Proposal: TASK-253 (Decide, @dengopaiv) on patching cpal. Docs: docs/building.md ASIO section. The test needs asio-sys as a Windows x86_64 dev-dependency (Cargo.lock gains one edge).

Review round 1 (send back), fixed in the amended commit: subject shortened; building.md now covers ReaRoute (opens only while REAPER runs, else no-clock and WASAPI fallback) and states the Realtek condition plainly (rejected by its own rate answers, busy device untested); probe doc says the driver plays what its buffers hold with ORB_ASIO_PROBE_START=1; run command has --release in test and docs; ASIO4ALL advice points to the exit-hang item. Probe compiles (cargo test --release -p orb-player --test asio_probe: 1 ignored); clippy and fmt clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Found: cpal lists only ReaRoute because MiniFuse fails ASIOInit (ASE_NotPresent, no device) and Realtek ASIO, which loads and starts, reports rate 0.0 and answers every canSampleRate/setSampleRate with ASE_NotPresent, so cpal keeps no rate for it and output_devices filters it out (supports_output); stream.rs would also fail its rate check. Documented in docs/building.md; ignored test crates/orb-player/tests/asio_probe.rs (asio-sys as a Windows dev-dependency); the cpal change is proposed as TASK-253 (Decide, @dengopaiv). No listing behavior changed. cargo test --release --workspace 1243 pass, 0 fail, 9 ignored; clippy -D warnings and nightly fmt --check clean. Branch worktree-agent-a8f40696c1e43d78f.
<!-- SECTION:FINAL_SUMMARY:END -->
