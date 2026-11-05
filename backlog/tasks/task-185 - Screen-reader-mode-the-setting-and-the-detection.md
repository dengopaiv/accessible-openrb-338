---
id: TASK-185
title: 'Screen reader mode: the setting and the detection'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-05T19:31:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-177
references:
  - crates/orb-ui/src/settings.rs
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/main.rs
  - docs/ui_spec.md
  - docs/accessibility_plan.md
  - docs/accessibility.md
priority: high
type: feature
ordinal: 208000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-1; the analysis is docs/accessibility.md. Read both before starting.

- What (with D1's recommendation): a setting `screen_reader` in `ui.toml` with the values
  `auto` (default), `on`, `off`; an Options menu item "Screen Reader Mode" with the three
  choices; `App::screen_reader_mode()` true when on, or auto and the system reports a screen
  reader. Detection on Windows through `SystemParametersInfoW(SPI_GETSCREENREADER)` by FFI
  (no new crate), checked at start and when the window regains the focus; on macOS and Linux
  `false` in this task (a later task can add VoiceOver and AT-SPI detection; note it as a
  follow-up task if not done). Nothing else changes yet: later tasks read the mode.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The setting round-trips through `ui.toml` (a test in the style of `crates/orb-ui/tests/settings.rs`), and an old `ui.toml` without the key reads as `auto`.
- [ ] #2 The menu item shows and changes the setting; with `off` no behavior differs from main.
- [ ] #3 On Windows with NVDA running, `auto` reports the mode on; without a screen reader, off (checked by hand, recorded in the notes with the NVDA version).
- [ ] #4 docs/ui_spec.md section 18 lists the item.
<!-- AC:END -->
