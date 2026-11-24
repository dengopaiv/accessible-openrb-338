---
id: TASK-287
title: >-
  Copy Pattern puts the whole pattern on the system clipboard as JSON; Paste
  reads it back
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-24T08:17:00Z'
updated_date: '2026-11-24T10:24:00Z'
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
  - `"device"`: `"303"`, `"808"` or `"909"` (no TB-/TR- names, the user's rule, task-286);
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
- A paste checks the identifying key, the version and the device. A 303 pattern pastes into either 303 row; an 808 or 909 pattern only into its own device. It also checks every value.
- On any problem it shows a status message ("The clipboard holds no pattern", "The clipboard holds a 909 pattern", "The clipboard's pattern is not valid: <reason>") and writes nothing. Never a partial pattern.
- A valid paste is one `write_pattern` with Undo, as today.

**Docs and tests:**
- docs/ui_spec.md section 16 gets the format, with a short example per device.
- Tests:
  - the round trip over every corpus pattern (`tests/refs` songs; the CD songs when `ORB_CD_ISO` is set, skipped otherwise);
  - out-of-range bytes survive;
  - malformed and wrong-device JSON are refused with nothing written;
  - copy in one App and paste in a second App instance.
  - The plugin's direct clipboard read is behind a small function a test can replace; no test touches the real system clipboard.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Copy/Cut Pattern puts the documented JSON on the system clipboard in the app and the plugin; no pid marker remains
- [x] #2 Paste reads it in the app (Cmd/Ctrl+V, Edit menu, editors) and in the plugin (Edit menu, direct clipboard read), across instances
- [x] #3 Copy then paste reproduces the pattern's bytes exactly for every corpus pattern, including values the readable form can't name
- [x] #4 cargo test --workspace, clippy -D warnings and nightly fmt --check pass
- [x] #5 The user has checked it in the app and the plugin, including a paste from the app into the plugin
- [x] #6 Malformed, wrong-version or wrong-device JSON is refused with a status message and nothing written
- [x] #7 docs/ui_spec.md section 16 documents the format with an example per device
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. pattern_json.rs: writer and strict parser over the link's pattern bytes (98/194), every byte carried (names where the form has one, else the number). 2. clipboard.rs: Clipboard::System (arboard) or Memory (tests); App::clipboard replaces Clip, clip_text, clip_count. 3. menu.rs: copy writes JSON; paste parses, checks device, writes once with Undo; status messages. 4. Standalone and plugin editor switch to System; tests keep Memory. 5. Tests: unit round trip of all 256 values at every position, corpus (163 songs), two apps, refusals. 6. docs/ui_spec.md section 16 plus the places that named the marker. 7. fmt, clippy, tests, app and VST3 bundles.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-aa690deec471be4a2

Design decisions: (1) Schema: top-level keys openrb338 'pattern', version 1, device '303'|'808'|'909', length (number), shuffle (bool), steps (16 objects, one per line). 303 step: note 'C'..'C\'' (13 names), slide, accent, up, down, gate 'note'|'pause'. 808 step: ac bd sd lt mt ht rs cp cb cy oh ch as flags. 909 step: ac (false=0, true=2) and bd sd lt mt ht rs cp ch oh cc rc as 'off'|'low'|'high'|'flam'. Any byte the form has no word for is written as its number in the same field; a paste takes numbers anywhere. (2) The memory layout is the link's (a 303 step is 6 bytes: note, slide, accent, up, down, rest), not the 2-byte file layout. (3) A length byte outside 1..16 (and any 0..255) is carried and pasted as is, because a song file may hold one (AC 3 asks for exact bytes); the paste checks type and range 0..255, not 1..16. (4) Unknown or missing keys are refused (typo catching). (5) 'No pattern' = not JSON or no openrb338:'pattern'; everything after that key is 'not valid: <reason>', checked before the device. (6) Clip is removed, not kept as a cache: a cache would let tests pass on a path the real clipboard does not take. The editors' Paste buttons are therefore always enabled (no per-frame clipboard read). (7) Clipboard abstraction: App::clipboard defaults to Memory (shared between apps by clone); main.rs and the plugin's EditorApp::build set System, so no test reaches the real clipboard. (8) Dependency: serde_json (already in the workspace) and arboard 3.7 with default-features off (already in Cargo.lock via egui-winit): reads the text on macOS/Windows; on Linux X11 only (no wayland-data-control), a pure Wayland session without XWayland reads nothing and shows 'holds no pattern'. (9) Pasting into the single-line text fields strips the newlines (egui), so the Info web-page test checks the text contains the key. Tests: cargo test --workspace --no-fail-fast 1079 passed, 0 failed, 5 ignored (107 suites); clippy -D warnings and nightly fmt --check clean. Corpus: 163 songs, 20864 patterns round trip.

Review round 1 (user). (1) A paste now refuses a length outside 1..16: 'The clipboard's pattern is not valid: "length" must be 1 to 16, not N'; a copy still writes whatever byte the pattern holds. The unit round trip covers all 256 values at every position except the length byte (checked written and refused); the corpus test round-trips every pattern with a length in 1..16 and checks the others copy and are refused: 163 songs, 20864 patterns, 0 of them with a length outside 1..16 (so none excluded). docs/ui_spec.md section 16 updated. (2) The user first asked for clipboard polling (Paste buttons enabled only while the clipboard holds a matching pattern), then withdrew it: 'keep paste always on'. No polling was shipped; the code was removed entirely; Paste buttons and Edit > Paste Pattern stay always enabled. Finding kept for the record: Apple's macOS 15.4 pasteboard privacy preview (NSPasteboard.accessBehavior, a prompt/banner on programmatic reads without user action, opt-in in 15.4 and aimed at macOS 26) means a periodic text read on macOS could show the alert; Apple's documented non-alerting route is the detect* methods, and changeCount is not documented as exempt. Re-run: cargo test --workspace 1079 passed, 0 failed, 5 ignored (107 suites); clippy -D warnings and nightly fmt --check clean; dev app and VST3 rebuilt.

Sign-off: the user checked commit 07efb20 in the app and the plugin on 2026-11-24 and said 'good' (format, length 1..16 refused, Paste always enabled, Linux X11 only).

Review round 2 (reviewer, send back): commit message reworded (a length outside 1..16 is written but refused); stale marker comments fixed in orb-plugin/tests/editor.rs, orb-ui/tests/menu.rs, error_box.rs; ui_spec section 16: shuffle may be a number; corpus: no CD-songs leg, because orb-testkit gives the CD's DAT, DLL, EXE and mods but not its songs, and the shipped songs are already committed in tests/songs/shipped (comment fixed); error messages cut an echoed value or key to 40 characters plus '...'; duplicate keys in one object are accepted and the last wins (serde_json Value; no custom parser; documented in ui_spec section 16); new unit tests: negative, float, exponent and u64-overflow numbers, top-level unknown and missing keys, the truncated echo, the duplicate key.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Edit > Copy/Cut Pattern and the editors' Copy put the whole pattern on the system clipboard as readable JSON; Paste (Cmd/Ctrl+V, Edit menu, editors; the plugin reads the system clipboard directly with arboard) reads it back across windows, instances and from the app into the plugin. Schema (version 1): openrb338 'pattern', version, device '303'|'808'|'909', length (1..16, refused outside), shuffle, 16 steps one per line; 303 step note/slide/accent/up/down/gate, 808 step ac+11 drums as flags, 909 step ac (false=0,true=2) + 11 drums off/low/high/flam; any byte without a name travels as its number. Decisions: Clip removed (no cache); Clipboard::Memory is the default so no test touches the real clipboard, the binary and the plugin editor use System; unknown or missing keys refused; no clipboard polling, Paste always enabled (user); Linux X11 only (arboard without wayland-data-control); a length outside 1..16 is refused on paste. Tests: cargo test --workspace 1079 passed, 0 failed, 5 ignored; clippy -D warnings and nightly fmt clean; corpus 163 songs, 20864 patterns round trip (none with a length outside 1..16); all 256 values at every position of the three layouts; two apps sharing a clipboard; refusals write nothing. Docs: docs/ui_spec.md section 16. Commit on branch worktree-agent-aa690deec471be4a2.
<!-- SECTION:FINAL_SUMMARY:END -->
