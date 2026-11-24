---
id: TASK-287
title: >-
  Copy Pattern puts the whole pattern on the system clipboard as JSON; Paste
  reads it back
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-24T08:17:00Z'
labels:
  - ui
milestone: m-9
dependencies:
  - TASK-286
references:
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/app.rs
  - crates/orb-ui/src/link.rs
  - docs/RBS42.txt
  - docs/ui_spec.md
priority: medium
type: feature
ordinal: 260270
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user decided on 2026-11-24.
- Today Edit > Copy/Cut Pattern keeps the pattern's bytes inside the app (`Clip`, crates/orb-ui/src/menu.rs:130). Only a marker text goes on the system clipboard (`OpenRB-338 pattern: <row> <name> [pid.count]`, menu.rs:1157).
- A paste works only when the clipboard still holds that exact marker in the same process (menu.rs:1244).
- So a pattern can't move between app windows, between plugin instances, or from the app into the plugin, and it can't be kept or shared as text.
The user chose a readable JSON format (not a raw-bytes wrapper) that carries the whole pattern.

**Format** (the agent designs the exact schema; it is documented in docs/ui_spec.md section 16 and the user reviews it in the app):
- One JSON object, pretty or compact, with:
  - an identifying key and a version, e.g. `"openrb338": "pattern", "version": 1`;
  - `"machine"`: `"303"`, `"808"` or `"909"` (no TB-/TR- names, the user's rule, task-286);
  - `"length"` (1..16) and `"shuffle"` (bool);
  - all 16 steps, including the steps past the length.
- **303** (docs/RBS42.txt, the 303 chunk): per step:
  - the note (tone 0..12 as a note name, C..C');
  - transpose up/down;
  - accent and slide;
  - gate: note or pause.
- **808**: per step, the AC row and the 11 drums (BD SD LT MT HT RS CP CB CY OH CH) on/off.
- **909**: per step, AC on/off and the 11 drums (BD SD LT MT HT RS CP CH OH CC RC) as off, low, high or flam.
- **Exact round trip:** copy then paste gives back the same bytes, for every pattern in every corpus song and the user's songs. A value the readable form can't name must still be carried, e.g. a 909 byte above 3, a 303 tone above 12, unused flag bits, an AC byte other than 0 or 2, or a length outside 1..16. Carry it as its number in the same field, or in an extra field. Never drop it silently.

**Copy:** Edit > Copy/Cut Pattern (and the pattern editors' Copy) puts the JSON on the system clipboard, in the standalone app and in the plugin. egui's `copy_text`, which egui-baseview writes too. The marker and the in-process-only `Clip` matching go. Whether `Clip` stays as a cache is the agent's call; record it in the notes.

**Paste:** Edit > Paste Pattern and Cmd/Ctrl+V, plus the pattern editors' Paste.
- The standalone app reads the clipboard text from egui's Paste event, as today.
- The plugin takes no keys, and egui-baseview reads the clipboard only on the paste key (window.rs:878). Its Edit > Paste Pattern must read the system clipboard directly: arboard 3.7 and copypasta 0.10 are already in Cargo.lock as dependencies of other crates. Any crate added is noted, as is how it behaves on Linux (X11/Wayland).
- A paste checks the identifying key, the version and the machine. A 303 pattern pastes into either 303 row; an 808 or 909 pattern only into its own machine. It also checks every value.
- On any problem it shows a status message ("The clipboard holds no pattern", "The clipboard holds a 909 pattern", "The clipboard's pattern is not valid: <reason>") and writes nothing. Never a partial pattern.
- A valid paste is one `write_pattern` with Undo, as today.

**Docs and tests:**
- docs/ui_spec.md section 16 gets the format, with a short example per machine.
- Tests:
  - the round trip over every corpus pattern (`tests/refs` songs; the CD songs when `ORB_CD_ISO` is set, skipped otherwise);
  - out-of-range bytes survive;
  - malformed and wrong-machine JSON are refused with nothing written;
  - copy in one App and paste in a second App instance.
  - The plugin's direct clipboard read is behind a small function a test can replace; no test touches the real system clipboard.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Copy/Cut Pattern puts the documented JSON on the system clipboard in the app and the plugin; no pid marker remains
- [ ] #2 Paste reads it in the app (Cmd/Ctrl+V, Edit menu, editors) and in the plugin (Edit menu, direct clipboard read), across instances
- [ ] #3 Copy then paste reproduces the pattern's bytes exactly for every corpus pattern, including values the readable form can't name
- [ ] #4 Malformed, wrong-version or wrong-machine JSON is refused with a status message and nothing written
- [ ] #5 docs/ui_spec.md section 16 documents the format with an example per machine
- [ ] #6 cargo test --workspace, clippy -D warnings and nightly fmt --check pass
- [ ] #7 The user has checked it in the app and the plugin, including a paste from the app into the plugin
<!-- AC:END -->
