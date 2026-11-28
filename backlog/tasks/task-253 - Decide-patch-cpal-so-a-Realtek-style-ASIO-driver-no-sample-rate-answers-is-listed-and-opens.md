---
id: TASK-253
title: >-
  Decide: patch cpal so a Realtek-style ASIO driver (no sample rate answers) is
  listed and opens
status: To Do
assignee:
  - '@dengopaiv'
created_date: '2026-11-28T12:35:00Z'
labels:
  - decision
  - player
milestone: m-7
dependencies:
  - TASK-252
references:
  - docs/building.md
  - crates/orb-player/tests/asio_probe.rs
priority: low
type: spike
ordinal: 270000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: task-252 (docs/building.md, ASIO section) found that the user's Realtek ASIO driver loads, creates buffers and starts, but reports sample rate 0.0 and answers every canSampleRate/setSampleRate with ASE_NotPresent. cpal 0.18.2 (src/host/asio/device.rs Devices::next) then keeps no common rate, the device has no supported configuration and HostTrait::output_devices filters it out; also build_output_stream (stream.rs ~1150) fails on the rate check. So the user cannot choose Realtek ASIO. Fixing it means a patch of cpal (a [patch.crates-io] vendored copy, or a fork), which is a maintenance cost the user decides.

Options: (1) leave as is: use WASAPI, or install a driver that answers canSampleRate (FlexASIO, ASIO4ALL); (2) vendor cpal with two changes: in Devices::next treat a rate call that fails with ASE_NotPresent as 'unknown' and offer the common rates, and in build_output_stream skip the rate check when the driver reports 0.0 and the request is the driver's own; then check by hand that Realtek ASIO really plays (not verified: it may refuse later, and NVDA shares the device, so a by-ear check by the user is needed); (3) report it upstream to cpal and wait.

Recommendation: (3) plus (1) first; (2) only if the user wants Realtek ASIO. Ask the user.
<!-- SECTION:DESCRIPTION:END -->
