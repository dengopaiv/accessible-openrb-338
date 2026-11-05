---
id: TASK-204
title: The Export dialog keeps the format the user picks
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-06T05:43:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-194
references:
  - crates/orb-ui/src/system_dialog.rs
  - crates/orb-ui/src/browser.rs
  - crates/orb-ui/src/settings.rs
priority: low
type: enhancement
ordinal: 224000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: in screen reader mode Export uses the system's file dialog (TASK-194). Its default extension is always WAV: `system_picker` adds the filters in the order wav, flac, ogg, and `with_extension` gives a name without extension `.wav`. The in-app browser keeps the format the user picked with its format buttons (`Browser::format`, `with_name`), so with the mode on a user who exports FLAC every time picks the filter again each time.

What: the Export dialog offers or keeps the format the user picks, for example one filter per format in the order of the last-used format (the first filter is the default extension on Windows), or the last format remembered in `ui.toml` (a new key in settings.rs) and used by `with_extension` for a name without extension. Decide the simplest that works on Windows, macOS and Linux (rfd's `add_filter` order; the saved path's extension tells the format chosen).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 After exporting as FLAC through the dialog, the next Export dialog offers FLAC first, and a name typed without extension gets .flac (headless test with the fake picker and a settings round trip, if a setting is added)
- [ ] #2 With the mode off the browser is unchanged
<!-- AC:END -->
