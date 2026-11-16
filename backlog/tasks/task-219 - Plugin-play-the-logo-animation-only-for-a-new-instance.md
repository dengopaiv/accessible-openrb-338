---
id: TASK-219
title: 'Plugin: play the logo animation only for a new instance'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-16T15:30:00Z'
labels:
  - ui
  - player
milestone: m-7
dependencies:
  - TASK-217
references:
  - crates/orb-ui/src/app.rs
  - crates/orb-ui/src/embed.rs
  - crates/orb-plugin/src/editor.rs
  - crates/orb-plugin/src/instance.rs
  - docs/plugin.md
priority: medium
type: enhancement
ordinal: 226000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why (the user, 2026-11-16):** the panel's logo animation, the standalone app's startup animation, plays every time the plugin's editor opens. In Reaper that means every time the FX window is opened, or the user switches FX in the chain, and also after reopening a project. The user wants it only for a new instance: the first editor open after the plugin is inserted fresh.

**Rule:**
- The animation plays at most once per instance, at the first editor open, and only if no state was ever loaded into the instance.
- A state load (CLAP `state.load`, VST3 `setState`, through nice-plug's state restore) marks the instance as restored. This covers reopening a project, duplicating or pasting the FX or its track, undo/redo, and loading a preset.
- A restored instance never plays it; neither does a later editor open of an instance that already played it.
- Mark it in the plugin instance (instance.rs), not the editor. The flags are runtime only, not part of the saved state: "restored" is set by the state load, and "played" is set when the editor starts the animation.

**Editor:** orb-ui's App already skips the animation in screenshot mode (app.rs, around lines 339-340). Give the embed (embed.rs; editor.rs's Embed impl) a way to say "skip the intro", and use it for the rule above. The standalone app is unchanged: its screenshots stay byte-identical and the animation still plays at its startup.

**Caveat (doc only):** a host that loads a default state on insertion makes the instance count as restored. Reaper does this when the user has saved a default preset for the plugin. Note that in docs/plugin.md.

**No compatibility work for old plugin states** (the user's rule until release).

Work only after task-217 is merged: it touches instance.rs and the state.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A new instance (no state ever loaded) plays the logo animation at its first editor open only; closing and reopening the editor does not replay it (test)
- [ ] #2 An instance that had a state loaded (project reopen, duplicate, preset, undo) never plays it (test through the state load path)
- [ ] #3 The flags are runtime only, not saved in the state; the standalone app still animates at startup and its screenshots are unchanged
- [ ] #4 docs/plugin.md describes the rule and the default-preset caveat
- [ ] #5 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->
