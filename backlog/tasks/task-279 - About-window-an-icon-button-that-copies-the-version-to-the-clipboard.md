---
id: TASK-279
title: 'About window: an icon button that copies the version to the clipboard'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-22T08:25:00Z'
updated_date: '2026-11-22T10:04:00Z'
labels:
  - ui
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/src/about.rs
  - docs/ui_spec.md
  - crates/orb-plugin/tests/editor.rs
priority: low
type: enhancement
ordinal: 259800
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** Help > About shows `Version YYYY.MMDD.HHMM (<[dev-]hash[-dirty]>)` as a plain egui label (crates/orb-ui/src/about.rs:139). The user can't select and copy that text in the plugin. Users need the exact version for bug reports. The user asked for this on 2026-11-22.

**What:**
- **The button:**
  - A small copy-to-clipboard button right after the version text, on the same line.
  - An icon only, no text: the usual "two overlapping sheets" copy glyph.
  - Drawn with the painter or taken from a font glyph that egui's default fonts have. Check it renders on all three platforms; no new image dependency unless needed.
  - Sized to the text line, so the About layout doesn't move (the existing about.rs layout tests).
  - A hover tooltip "Copy version".
- **What it copies:** the version string exactly as shown after "Version " (the `VERSION` constant). The copy goes through `ctx.copy_text`.
  - egui-baseview 0.9 (crates.io) writes egui's CopyText command to the system clipboard (its window.rs, `OutputCommand::CopyText`), so this works in the plugin as well as in the standalone app.
  - Only reading the clipboard is limited in the plugin: egui-baseview reads it on the paste key only. This task doesn't read.
- **Feedback:** after a click, show brief confirmation, for example the icon turning into a check mark or the tooltip saying "Copied" for about 1.5 s. It repaints by itself, so no stale state stays.
- **Where:** in both the standalone app and the plugin, since they share about.rs. Update docs/ui_spec.md (the Help > About row and the About section).

**Tests:**
- An egui test drives a click on the button and checks the CopyText output command carries VERSION.
- The layout tests still pass.
- The plugin's editor tests show the About window with the button.

**User check:** the user clicks it in the plugin (REAPER) and the standalone app, and pastes the result, before the agent review.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Help > About has a small icon-only copy button right after the version text, with a 'Copy version' tooltip, in both the standalone app and the plugin; the About layout is otherwise unchanged
- [x] #2 Clicking it copies exactly the version string (VERSION) via ctx.copy_text, with a brief visible confirmation; tested by driving a click and checking the CopyText output
- [x] #3 docs/ui_spec.md describes the button; cargo test --workspace passes, clippy -D warnings and nightly fmt clean
- [x] #4 The user checked the copy in the plugin and the standalone app before the agent review
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Painter-drawn copy glyph (two sheets) + check mark, button after the version in about.rs, tooltip, ctx.copy_text(VERSION), 1.5 s confirm. 2. Test driving a click and checking CopyText. 3. ui_spec.md. 4. test/clippy/fmt, build app and VST3.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a625e4b1a912cc6a4

Painter-drawn glyph (two sheets, check mark for 1.5 s, tooltip Copy version/Copied), 14 pt, in about.rs; test the_about_copy_button_copies_the_version (menu.rs). cargo test --workspace all pass, clippy -D warnings and nightly fmt clean (a deprecated Id::new fixed to Id::unique). AC 1 plugin part: the button is drawn by the shared about.rs; the actual clipboard write in REAPER is the user's check (AC 4, left unchecked). Built target/OpenRB-338.app and target/bundled/OpenRB-338.vst3.

User sign-off 2026-11-22 ('looks good'): copied and pasted in the dev app and the plugin. Added plugin editor test the_about_window_has_a_copy_version_button (orb-plugin/tests/editor.rs).

Review round 1: commit body Result now records the user's verification; added test the_about_copy_feedback_reverts_after_1_5_seconds (orb-ui/tests/menu.rs) with pub fn copy_confirmation_shows (a tooltip check was not reachable in the headless harness: a click suppresses it, so the test checks the shared state that picks the check mark and the tooltip text).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Help > About has a painter-drawn icon-only copy button after the version (14 pt, two sheets, check mark and 'Copied' tooltip for 1.5 s, tooltip 'Copy version'); a click calls ctx.copy_text(VERSION). Tests: an orb-ui menu test and a plugin editor test drive the click and check CopyText; workspace tests, clippy -D warnings, nightly fmt clean. User verified in dev app and plugin. Branch worktree-agent-a625e4b1a912cc6a4, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
