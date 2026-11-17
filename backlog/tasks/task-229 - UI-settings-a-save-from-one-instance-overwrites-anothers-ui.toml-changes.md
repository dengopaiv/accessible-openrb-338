---
id: TASK-229
title: 'UI settings: a save from one instance overwrites another''s ui.toml changes'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-17T07:48:00Z'
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
- [ ] #1 Two Settings values loaded from the same file: A sets mods_dir and saves, then B notes a recent folder and saves; the file has A's mods_dir and both recent folders (test).
- [ ] #2 The same holds for songs_dir, last_song_dir/last_mod_dir, preferred_mods and startup_sound: a save changes only the key it was for (tests).
- [ ] #3 An instance's in-memory settings reflect the merged file after its save.
- [ ] #4 The existing tests in crates/orb-ui/tests/settings.rs pass unchanged; cargo test, clippy -D warnings and nightly fmt pass.
<!-- AC:END -->
