---
id: TASK-204
title: The Export dialog keeps the format the user picks
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-06T05:43:00Z'
updated_date: '2026-11-07T09:19:00Z'
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
- [x] #1 After exporting as FLAC through the dialog, the next Export dialog offers FLAC first, and a name typed without extension gets .flac (headless test with the fake picker and a settings round trip, if a setting is added)
- [x] #2 With the mode off the browser is unchanged
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Design: new ui.toml key export_format (wav|flac|ogg, default wav) in Settings. The system Export dialog is asked with a new Request::Export(format): system_picker puts that format's filter first (first filter is the default extension on Windows; the other platforms also follow filter order); with_extension takes the default format for a name typed without extension; App::finish_pick (system path only) stores the extension of the saved path as export_format and saves ui.toml. The in-app browser is untouched (session-only, WAV at open). Tests in tests/system_dialog.rs and tests/settings.rs. Docs: accessibility_plan 2.2 item 26, and the dialog description.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-adb7af0f939f419a1

Design: persisted ui.toml key export_format (wav default) used by the system dialog only. New Request::Export(format) puts that format's filter first (rfd add_filter order; first = default extension on Windows) and with_extension(mode, path, format) gives a bare name that extension; App::finish_pick stores the saved path's extension and saves ui.toml. Persisted rather than session-only because a user who always exports FLAC would otherwise pick again each launch; the browser's own format is per-open and was left alone (mode off unchanged). Checks: fmt --check, clippy -D warnings clean; cargo test --release -p orb-ui: all pass except crash.rs a_deliberate_panic_exits_101_and_writes_the_log (the known TASK-200 failure). Not verified here: the real dialog's filter order on macOS/Linux (NVDA item 26 added unchecked).

Merged by the supervisor 2026-11-07: review merge on the first round. system_dialog (9) and settings (11) pass on main after the squash.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The system Export dialog now remembers the format: ui.toml key export_format (wav|flac|ogg), Request::Export(format) puts that filter first, with_extension gives a bare name that extension, finish_pick updates the setting. Test the_export_dialog_keeps_the_format_the_user_picked (fake picker, restart round trip) and the settings round trip pass; browser untouched. Docs: ui_spec File menu row, settings.rs module text, accessibility_plan 2.2 item 26 (unchecked). Branch worktree-agent-adb7af0f939f419a1. crash.rs failure is TASK-200.
<!-- SECTION:FINAL_SUMMARY:END -->
