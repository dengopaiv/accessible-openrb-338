---
id: TASK-199
title: Screen reader detection on macOS (VoiceOver) and Linux (AT-SPI)
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-05T19:59:00Z'
labels:
  - ui
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/src/screen_reader.rs
  - docs/accessibility_plan.md
priority: low
type: feature
ordinal: 221000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
TASK-185 added screen reader mode (setting screen_reader = auto/on/off in ui.toml, App::screen_reader_mode()) and the automatic detection on Windows only (crates/orb-ui/src/screen_reader.rs, system_reports_screen_reader(), SPI_GETSCREENREADER). Other platforms report false, so Automatic never turns the mode on there; the user can still choose On.

Work: implement system_reports_screen_reader() for macOS (VoiceOver running: NSWorkspace isVoiceOverEnabled, or the com.apple.universalaccess voiceOverOnOffKey preference) and Linux (the AT-SPI session bus property org.a11y.Status IsEnabled / ScreenReaderEnabled), by FFI or the crates already in the tree; no new heavy dependency without a Decide: task. It is asked at start and on focus regain (App::check_screen_reader).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 On macOS with VoiceOver running, Automatic reports the mode on, without it off (checked by hand, recorded in the notes)
- [ ] #2 On Linux with Orca running, Automatic reports the mode on, without it off (checked by hand, recorded in the notes)
- [ ] #3 Other platforms and a failed query still report false; cargo test -p orb-ui passes on all three CI platforms
<!-- AC:END -->
