---
id: TASK-286
title: No TR-/TB- names in user-visible text (app, plugin, CLI)
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-24T08:15:00Z'
labels:
  - ui
  - cli
milestone: m-9
dependencies:
  - TASK-284
references:
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/drum_editor.rs
  - crates/orb-cli/src/render.rs
priority: medium
type: enhancement
ordinal: 260260
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user's rule (2026-11-24): no direct references to the original Roland hardware. User-visible text says 303, 808 and 909, never TB-303, TR-808 or TR-909. task-284 applies it to the Keyboard Shortcuts help. This task applies it everywhere else a user can see text.

**Scope:** every string a user can see, in crates/orb-ui, crates/orb-plugin and crates/orb-cli. That covers:
- menu items, window titles, tooltips, status messages and error boxes;
- the pattern clipboard marker (`ROW_NAMES` / `ROW_KINDS` in crates/orb-ui/src/menu.rs: "TB-303 1" becomes "303 1", "TR-808" becomes "808"; the "The clipboard holds a {kind} pattern" status follows);
- the drum editors' Machine names and texts (crates/orb-ui/src/drum_editor.rs around lines 165 and 454);
- CLI output and --help (e.g. crates/orb-cli/src/render.rs:91-118, the drum machines listed for a song), including any --json value. A changed JSON value is a format change: note it in the notes and in docs/ where the format is described.

Grep for TR-?808, TR-?909 and TB-?303 in string literals. Rust identifiers (`Machine::Tr808`, `tr808.bg`), comments and the reverse-engineering docs (docs/initial_investigation.md, docs/RBS42.txt and the like, which describe the original) are out of scope. docs/ui_spec.md and docs/plugin.md change only where they quote a user-visible string. List any user-visible text that can't change in the notes, e.g. text inside a mod's panel bitmap, which is the mod's art.

**Tests:** update the tests that assert the old strings. Add one test that scans the UI's user-visible text tables (menus, the help, the editors' texts) for TR-/TB- names, as far as the code structure allows; record in the notes what it can't cover.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 No user-visible string in orb-ui, orb-plugin or orb-cli contains TB-303, TR-808 or TR-909 (or without the hyphen); the grep is in the notes
- [ ] #2 The pattern clipboard marker and the paste status use 303 1, 303 2, 808, 909
- [ ] #3 Any changed CLI or JSON output is noted and its docs updated
- [ ] #4 A test guards against TR-/TB- names in the UI's text tables
- [ ] #5 cargo test --workspace, clippy -D warnings and nightly fmt --check pass
- [ ] #6 The user has checked it in the app
<!-- AC:END -->
