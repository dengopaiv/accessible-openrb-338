---
id: TASK-229
title: 'UI settings: a save from one instance overwrites another''s ui.toml changes'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-17T07:48:00Z'
updated_date: '2026-11-17T08:18:00Z'
labels:
  - ui
  - player
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/src/settings.rs
  - crates/orb-ui/src/browser.rs
  - crates/orb-ui/src/menu.rs
  - crates/orb-plugin/src/editor.rs
priority: low
type: bug
ordinal: 2500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The settings file OpenRB-338/ui.toml (crates/orb-ui/src/settings.rs) is shared by the standalone app and every plugin instance (docs/plugin.md, File menu bullet). Each one loads it once (the plugin when its editor opens, crates/orb-plugin/src/editor.rs Settings::load()) and every save writes its whole in-memory copy. With two instances open, a save from one (a file chosen in the browser: recent folders and last_song_dir/last_mod_dir; the songs/mods folder; preferred_mods; startup_sound) silently reverts keys another instance changed since it loaded. The write is atomic (temp file + rename), so the file is never corrupt; the loss is of other instances' changes.

Fix: every save re-reads the file just before writing and applies only this change: a recent folder is pushed onto the list read from disk (dedupe, MAX_RECENT), a single key set overwrites only that key, a preferred mod sets only its entry. The in-memory copy is refreshed from the merged result. Concurrent saves in the same instant may still race; that is accepted (no locking). Keep the existing behaviors: a damaged file stays until a save, unknown keys ignored, non-UTF-8 paths left out.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Two Settings values loaded from the same file: A sets mods_dir and saves, then B notes a recent folder and saves; the file has A's mods_dir and both recent folders (test).
- [x] #2 The same holds for songs_dir, last_song_dir/last_mod_dir, preferred_mods and startup_sound: a save changes only the key it was for (tests).
- [x] #3 An instance's in-memory settings reflect the merged file after its save.
- [x] #4 The existing tests in crates/orb-ui/tests/settings.rs pass unchanged; cargo test, clippy -D warnings and nightly fmt pass.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Settings keeps a base snapshot (values at last read/write) and a list of noted folders; save() re-reads the file, applies only keys that differ from base, pushes noted folders onto the file's recent, sets/clears only changed preferred_mods entries, writes atomically, then refreshes memory and base. save takes &mut self.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a3515c456c5129504

Evidence: tests a_save_keeps_the_changes_another_instance_made and merged_recent_folders_are_deduped_and_limited in crates/orb-ui/tests/settings.rs; existing tests unchanged. Bookkeeping fields base/noted are pub #[doc(hidden)] only because tests/browser.rs uses struct update (..Settings::default()). No plugin-side change needed (callers use settings.save() through the same API). No drawing code touched, so no screenshot check. cargo test 859 pass (90 binaries), clippy -D warnings and nightly fmt clean.

Review round 1: explicit sets are now recorded by setters (set_songs_dir, set_mods_dir, set_last_song_dir, set_last_mod_dir, set_startup_sound, set_preferred_mod; call sites in browser.rs and menu.rs use them) instead of diffing against base alone (diff kept as fallback for direct assignment); read distinguishes NotFound/damaged (whole in-memory copy is written, supervisor decision) from other IO errors (save fails, nothing changed); unique temp file name (pid + counter); module doc and ui_spec updated (recent only via note_recent, base/noted/dirty public for struct literals, base is the defaults). New tests: explicit set equal to loaded value, clearing to None, missing/damaged file, unreadable file (unix, skips if still readable). cargo test 862 pass, clippy and fmt clean.

Review round 2: commit message rewritten for the setters/dirty, read-error, whole-copy and unique temp name rules; module doc note names base, noted and dirty. cargo test 863 pass, clippy and fmt clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
A save now merges only this instance's change into the re-read file (crates/orb-ui/src/settings.rs); memory refreshed from the merged result. Tests added; cargo test 859 pass, clippy and fmt clean. Docs: ui_spec.md settings, plugin.md File bullet, module doc.
<!-- SECTION:FINAL_SUMMARY:END -->
