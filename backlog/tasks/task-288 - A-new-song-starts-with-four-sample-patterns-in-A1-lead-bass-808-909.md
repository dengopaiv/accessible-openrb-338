---
id: TASK-288
title: A new song starts with four sample patterns in A1 (lead, bass, 808, 909)
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-24T10:47:00Z'
updated_date: '2026-11-24T11:09:00Z'
labels:
  - ui
milestone: m-9
dependencies: []
references:
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/link.rs
  - crates/orb-ui/src/pattern_json.rs
  - crates/orb-plugin/src/instance.rs
priority: medium
type: feature
ordinal: 263700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user asked on 2026-11-24 (relayed from session openrb-338-3b): "create task to use these patterns in A1 in each synth as a default template (File -> New, or when starting app). lead should go into first 303, bass into second".

**What:** a new song gets four sample patterns in pattern A1 instead of empty ones. A new song is File > New, and the song the app starts with (`App::start`, `App::new_song` in crates/orb-ui/src/menu.rs around 672-710; `Link`'s File > New in crates/orb-ui/src/link.rs around 804).
- the first 303, A1: the lead pattern (303-lead.json below);
- the second 303, A1: the bass pattern (303-bass.json);
- the 808, A1: 808.json;
- the 909, A1: 909.json.
Every other pattern stays empty. A song opened from a file is unchanged. The engine's own empty song (the rbs crate's song constructor, the CLI, the corpus) is unchanged: this is a UI template applied on top of it.

**The patterns** are original acid-house samples in A minor that play together, written in that session (not transcriptions of any track). They are in the Edit > Copy Pattern clipboard JSON format (crates/orb-ui/src/pattern_json.rs, version 1), with all 16 steps and shuffle off. They were checked only against the paste parser's rules; nobody has pasted them into the app or listened to them yet. Ship them as files in the repository, e.g. assets/patterns/*.json, embedded with include_str! and parsed with pattern_json at startup, or converted at build time; the agent decides. A test checks that each parses and lands in A1 of its row. The exact files follow, verbatim (sha256 in each heading).

**Open points, recommendations pending the user's word in review:**
1. The rows keep selecting A1, which is already the default selection, so the template plays when Play is pressed.
2. Knobs and other settings stay as a new song has them today.
3. The plugin: a new plugin instance's initial song, and the plugin's File > New (crates/orb-plugin/src/instance.rs `new_song`, around 1012), get the same template. Projects saved with an earlier version keep their saved state; the plugin state compatibility rule holds, since only a fresh state changes.
4. Tests that assume a new song is empty, or render a new song, are updated to the template or switched to the engine's empty song; list them in the notes.

The grid that fully defines the patterns, in case the JSON and the grid disagree (the JSON wins):
- 303 steps: NOTE, then "-" = down true, "+" = up true, "!" = accent true, "~" = slide true; gate "note". "." = the empty step (note "C", all flags false, gate "pause").
  - lead: . . E+! . E+~ G+ . A+! . D+ E+~ C'+! . A C+~ D+!
  - bass: A-! A- A A- . A-~ C A- A-! . G- A- A! . E-~ G-
- 808 (x = true; unlisted columns all false): ac at steps 4 and 12; cb at 4, 7, 12 and 15; rs at 2 and 10; ht at 11; mt at 14; lt at 16.
- 909 (h high, l low; unlisted columns all off): ac true at 1, 5, 9 and 13; bd h at 1, 5, 9 and 13; cp h at 5 and 13; sd l at 8 and 16; ch h at 1, 5, 9 and 13 and l at 2, 4, 6, 8, 10, 12, 14 and 16; oh h at 3, 7, 11 and 15.

### 303-lead.json (sha256 43f5915b465e5ed5f9c8f5763f0e1a8f5a0ea442d755f7d3dd6dc00e95c42cb4)

```json
{
  "openrb338": "pattern",
  "version": 1,
  "device": "303",
  "length": 16,
  "shuffle": false,
  "steps": [
    {"note": "C", "slide": false, "accent": false, "up": false, "down": false, "gate": "pause"},
    {"note": "C", "slide": false, "accent": false, "up": false, "down": false, "gate": "pause"},
    {"note": "E", "slide": false, "accent": true, "up": true, "down": false, "gate": "note"},
    {"note": "C", "slide": false, "accent": false, "up": false, "down": false, "gate": "pause"},
    {"note": "E", "slide": true, "accent": false, "up": true, "down": false, "gate": "note"},
    {"note": "G", "slide": false, "accent": false, "up": true, "down": false, "gate": "note"},
    {"note": "C", "slide": false, "accent": false, "up": false, "down": false, "gate": "pause"},
    {"note": "A", "slide": false, "accent": true, "up": true, "down": false, "gate": "note"},
    {"note": "C", "slide": false, "accent": false, "up": false, "down": false, "gate": "pause"},
    {"note": "D", "slide": false, "accent": false, "up": true, "down": false, "gate": "note"},
    {"note": "E", "slide": true, "accent": false, "up": true, "down": false, "gate": "note"},
    {"note": "C'", "slide": false, "accent": true, "up": true, "down": false, "gate": "note"},
    {"note": "C", "slide": false, "accent": false, "up": false, "down": false, "gate": "pause"},
    {"note": "A", "slide": false, "accent": false, "up": false, "down": false, "gate": "note"},
    {"note": "C", "slide": true, "accent": false, "up": true, "down": false, "gate": "note"},
    {"note": "D", "slide": false, "accent": true, "up": true, "down": false, "gate": "note"}
  ]
}
```

### 303-bass.json (sha256 5ad14e134348b9d1e37327990ca61bf7dffce8ea695269b540ae5b1c4d7a5ce8)

```json
{
  "openrb338": "pattern",
  "version": 1,
  "device": "303",
  "length": 16,
  "shuffle": false,
  "steps": [
    {"note": "A", "slide": false, "accent": true, "up": false, "down": true, "gate": "note"},
    {"note": "A", "slide": false, "accent": false, "up": false, "down": true, "gate": "note"},
    {"note": "A", "slide": false, "accent": false, "up": false, "down": false, "gate": "note"},
    {"note": "A", "slide": false, "accent": false, "up": false, "down": true, "gate": "note"},
    {"note": "C", "slide": false, "accent": false, "up": false, "down": false, "gate": "pause"},
    {"note": "A", "slide": true, "accent": false, "up": false, "down": true, "gate": "note"},
    {"note": "C", "slide": false, "accent": false, "up": false, "down": false, "gate": "note"},
    {"note": "A", "slide": false, "accent": false, "up": false, "down": true, "gate": "note"},
    {"note": "A", "slide": false, "accent": true, "up": false, "down": true, "gate": "note"},
    {"note": "C", "slide": false, "accent": false, "up": false, "down": false, "gate": "pause"},
    {"note": "G", "slide": false, "accent": false, "up": false, "down": true, "gate": "note"},
    {"note": "A", "slide": false, "accent": false, "up": false, "down": true, "gate": "note"},
    {"note": "A", "slide": false, "accent": true, "up": false, "down": false, "gate": "note"},
    {"note": "C", "slide": false, "accent": false, "up": false, "down": false, "gate": "pause"},
    {"note": "E", "slide": true, "accent": false, "up": false, "down": true, "gate": "note"},
    {"note": "G", "slide": false, "accent": false, "up": false, "down": true, "gate": "note"}
  ]
}
```

### 808.json (sha256 8cee68c8aea7443559a3f774f80a3114268644ed4180096e108e8756802bcd2f)

```json
{
  "openrb338": "pattern",
  "version": 1,
  "device": "808",
  "length": 16,
  "shuffle": false,
  "steps": [
    {"ac": false, "bd": false, "sd": false, "lt": false, "mt": false, "ht": false, "rs": false, "cp": false, "cb": false, "cy": false, "oh": false, "ch": false},
    {"ac": false, "bd": false, "sd": false, "lt": false, "mt": false, "ht": false, "rs": true, "cp": false, "cb": false, "cy": false, "oh": false, "ch": false},
    {"ac": false, "bd": false, "sd": false, "lt": false, "mt": false, "ht": false, "rs": false, "cp": false, "cb": false, "cy": false, "oh": false, "ch": false},
    {"ac": true, "bd": false, "sd": false, "lt": false, "mt": false, "ht": false, "rs": false, "cp": false, "cb": true, "cy": false, "oh": false, "ch": false},
    {"ac": false, "bd": false, "sd": false, "lt": false, "mt": false, "ht": false, "rs": false, "cp": false, "cb": false, "cy": false, "oh": false, "ch": false},
    {"ac": false, "bd": false, "sd": false, "lt": false, "mt": false, "ht": false, "rs": false, "cp": false, "cb": false, "cy": false, "oh": false, "ch": false},
    {"ac": false, "bd": false, "sd": false, "lt": false, "mt": false, "ht": false, "rs": false, "cp": false, "cb": true, "cy": false, "oh": false, "ch": false},
    {"ac": false, "bd": false, "sd": false, "lt": false, "mt": false, "ht": false, "rs": false, "cp": false, "cb": false, "cy": false, "oh": false, "ch": false},
    {"ac": false, "bd": false, "sd": false, "lt": false, "mt": false, "ht": false, "rs": false, "cp": false, "cb": false, "cy": false, "oh": false, "ch": false},
    {"ac": false, "bd": false, "sd": false, "lt": false, "mt": false, "ht": false, "rs": true, "cp": false, "cb": false, "cy": false, "oh": false, "ch": false},
    {"ac": false, "bd": false, "sd": false, "lt": false, "mt": false, "ht": true, "rs": false, "cp": false, "cb": false, "cy": false, "oh": false, "ch": false},
    {"ac": true, "bd": false, "sd": false, "lt": false, "mt": false, "ht": false, "rs": false, "cp": false, "cb": true, "cy": false, "oh": false, "ch": false},
    {"ac": false, "bd": false, "sd": false, "lt": false, "mt": false, "ht": false, "rs": false, "cp": false, "cb": false, "cy": false, "oh": false, "ch": false},
    {"ac": false, "bd": false, "sd": false, "lt": false, "mt": true, "ht": false, "rs": false, "cp": false, "cb": false, "cy": false, "oh": false, "ch": false},
    {"ac": false, "bd": false, "sd": false, "lt": false, "mt": false, "ht": false, "rs": false, "cp": false, "cb": true, "cy": false, "oh": false, "ch": false},
    {"ac": false, "bd": false, "sd": false, "lt": true, "mt": false, "ht": false, "rs": false, "cp": false, "cb": false, "cy": false, "oh": false, "ch": false}
  ]
}
```

### 909.json (sha256 0c63e002b4043e81f3992e235e668de3d81fd3e47a25e371dd777a2b31c04fc6)

```json
{
  "openrb338": "pattern",
  "version": 1,
  "device": "909",
  "length": 16,
  "shuffle": false,
  "steps": [
    {"ac": true, "bd": "high", "sd": "off", "lt": "off", "mt": "off", "ht": "off", "rs": "off", "cp": "off", "ch": "high", "oh": "off", "cc": "off", "rc": "off"},
    {"ac": false, "bd": "off", "sd": "off", "lt": "off", "mt": "off", "ht": "off", "rs": "off", "cp": "off", "ch": "low", "oh": "off", "cc": "off", "rc": "off"},
    {"ac": false, "bd": "off", "sd": "off", "lt": "off", "mt": "off", "ht": "off", "rs": "off", "cp": "off", "ch": "off", "oh": "high", "cc": "off", "rc": "off"},
    {"ac": false, "bd": "off", "sd": "off", "lt": "off", "mt": "off", "ht": "off", "rs": "off", "cp": "off", "ch": "low", "oh": "off", "cc": "off", "rc": "off"},
    {"ac": true, "bd": "high", "sd": "off", "lt": "off", "mt": "off", "ht": "off", "rs": "off", "cp": "high", "ch": "high", "oh": "off", "cc": "off", "rc": "off"},
    {"ac": false, "bd": "off", "sd": "off", "lt": "off", "mt": "off", "ht": "off", "rs": "off", "cp": "off", "ch": "low", "oh": "off", "cc": "off", "rc": "off"},
    {"ac": false, "bd": "off", "sd": "off", "lt": "off", "mt": "off", "ht": "off", "rs": "off", "cp": "off", "ch": "off", "oh": "high", "cc": "off", "rc": "off"},
    {"ac": false, "bd": "off", "sd": "low", "lt": "off", "mt": "off", "ht": "off", "rs": "off", "cp": "off", "ch": "low", "oh": "off", "cc": "off", "rc": "off"},
    {"ac": true, "bd": "high", "sd": "off", "lt": "off", "mt": "off", "ht": "off", "rs": "off", "cp": "off", "ch": "high", "oh": "off", "cc": "off", "rc": "off"},
    {"ac": false, "bd": "off", "sd": "off", "lt": "off", "mt": "off", "ht": "off", "rs": "off", "cp": "off", "ch": "low", "oh": "off", "cc": "off", "rc": "off"},
    {"ac": false, "bd": "off", "sd": "off", "lt": "off", "mt": "off", "ht": "off", "rs": "off", "cp": "off", "ch": "off", "oh": "high", "cc": "off", "rc": "off"},
    {"ac": false, "bd": "off", "sd": "off", "lt": "off", "mt": "off", "ht": "off", "rs": "off", "cp": "off", "ch": "low", "oh": "off", "cc": "off", "rc": "off"},
    {"ac": true, "bd": "high", "sd": "off", "lt": "off", "mt": "off", "ht": "off", "rs": "off", "cp": "high", "ch": "high", "oh": "off", "cc": "off", "rc": "off"},
    {"ac": false, "bd": "off", "sd": "off", "lt": "off", "mt": "off", "ht": "off", "rs": "off", "cp": "off", "ch": "low", "oh": "off", "cc": "off", "rc": "off"},
    {"ac": false, "bd": "off", "sd": "off", "lt": "off", "mt": "off", "ht": "off", "rs": "off", "cp": "off", "ch": "off", "oh": "high", "cc": "off", "rc": "off"},
    {"ac": false, "bd": "off", "sd": "low", "lt": "off", "mt": "off", "ht": "off", "rs": "off", "cp": "off", "ch": "low", "oh": "off", "cc": "off", "rc": "off"}
  ]
}
```
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 File > New and the app's starting song hold the lead in the first 303's A1, the bass in the second 303's A1, and 808.json and 909.json in A1 of the 808 and the 909; every other pattern is empty
- [x] #2 The four JSON files are in the repository verbatim (sha256 as in the description) and a test parses each and checks its bytes in A1 of its row
- [x] #3 The plugin's new instance and its File > New get the same template; a saved plugin state loads unchanged
- [x] #4 A song opened from a file, the engine's empty song, the CLI and the corpus are unchanged
- [x] #5 Tests that assumed an empty new song are updated and listed in the notes
- [x] #6 cargo test --workspace, clippy -D warnings and nightly fmt --check pass
- [x] #7 The user has listened to the template in the app and the plugin
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. assets/patterns/*.json verbatim (sha256 checked). 2. orb-ui template.rs: embeds and parses them, apply() writes A1 of each row into an engine. 3. Link::load_new applies it (File > New and App::start both go through it; Link::new, the engine empty song, rbs, CLI, corpus untouched). 4. Plugin: Doc::new_song() (patterns of the empty song with the template in A1) for OrbParams::new, a restore with no saved state, and OrbParams::new_song; Doc::from_json untouched so a saved state loads unchanged. 5. Update tests that assume an empty new song; list them. 6. cargo test --workspace, clippy, nightly fmt; build app and VST3.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ab0cc56c413e20fb0

Implementation (awaiting the user's listen, AC 7 open). Files: assets/patterns/{303-lead,303-bass,808,909}.json (sha256 verified against the task, each file ends with a newline, hash is of the text block plus newline as in the task); crates/orb-ui/src/template.rs (embeds, parses with pattern_json, apply() writes A1); Link::load_new applies it, which both App::start and File > New go through (Link::new, rbs::Song::empty, CLI, corpus untouched; a song opened from a file untouched); plugin Doc::new_song() (patterns of the empty song with the template in A1) used by OrbParams::new, a restore with no saved state, and OrbParams::new_song via build_song. Doc::from_json untouched, so a saved state loads unchanged (test: a saved state with empty pattern arrays stays empty). Open points: 1 A1 selection kept; 2 knobs unchanged; 3 plugin done as recommended; 4 tests updated. Awkward: the drum machines hold patterns only when built with samples, so the plugin's template_patterns builds with the built-in kit (with None the 808/909 A1 stayed empty, caught by the new plugin test). Error fallbacks in the plugin (a saved song that fails to load) still give the empty song. Tests changed: orb-ui tests/menu.rs file_new_gives_the_state_of_a_fresh_start (start state is now the template, compared with a second File > New, and the mirror compared with a started app), tests/repaint.rs new_song() helper (uses the engine's empty song: the template is not silent after Stop). Tests added: template.rs files_are_verbatim (sha256), each_file_parses_for_its_row; menu.rs start_and_file_new_hold_the_template; plugin tests/state.rs a_new_instance_and_file_new_hold_the_template, a_saved_state_does_not_get_the_template. cargo test --workspace --no-fail-fast: 1088 pass, 0 fail; clippy -D warnings and nightly fmt --check clean. Built target/OpenRB-338.app and target/bundled/OpenRB-338.vst3.

The user listened to the template in the app and the plugin and signed it off on 2026-11-24 ("good"), including the plugin's error fallback giving the empty song.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
A new song (the app's start, File > New, the plugin's new instance and File > New) starts with the lead in the first 303's A1, the bass in the second 303's A1, and the 808 and 909 patterns; everything else is empty. Files in assets/patterns/ (sha256 checked), orb-ui template.rs, Link::load_new, plugin Doc::new_song. Saved plugin states, songs opened from files, the engine's empty song, the CLI and the corpus are unchanged. cargo test --workspace 1088 pass, clippy and nightly fmt clean. Commit on branch worktree-agent-ab0cc56c413e20fb0.
<!-- SECTION:FINAL_SUMMARY:END -->
