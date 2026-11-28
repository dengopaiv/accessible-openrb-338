---
id: TASK-252
title: >-
  ASIO: find why cpal does not list the Realtek and MiniFuse ASIO drivers on the
  user's PC
status: To Do
assignee: []
created_date: '2026-11-28T12:31:00Z'
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
- [ ] #1 The notes say, for each of the three drivers, at which step cpal's listing drops it or keeps it, with the ASIO error, from a run on the user's PC
- [ ] #2 docs/building.md's ASIO section states what was found and what a user can do (for example, which driver works here and under what condition)
- [ ] #3 Any change to list more drivers is proposed as a task (or a Decide: task), not made in this spike
<!-- AC:END -->
