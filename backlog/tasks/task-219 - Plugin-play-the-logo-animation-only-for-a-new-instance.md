---
id: TASK-219
title: 'Plugin: play the logo animation only for a new instance'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-16T15:30:00Z'
updated_date: '2026-11-16T16:08:00Z'
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
- [x] #1 A new instance (no state ever loaded) plays the logo animation at its first editor open only; closing and reopening the editor does not replay it (test)
- [x] #2 An instance that had a state loaded (project reopen, duplicate, preset, undo) never plays it (test through the state load path)
- [x] #3 The flags are runtime only, not saved in the state; the standalone app still animates at startup and its screenshots are unchanged
- [x] #4 docs/plugin.md describes the rule and the default-preset caveat
- [x] #5 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. OrbParams gets runtime flags restored (set in deserialize_fields) and intro_played; take_intro() = first call && !restored. 2. editor.rs build calls enable_logo_anim only if take_intro(); standalone untouched. 3. Tests in tests/state.rs; docs/plugin.md 5.1 with the default-preset caveat.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a035782db196a465b

Evidence: take_intro()/restored/intro_played in instance.rs; tests the_logo_animation_plays_once_for_a_new_instance and a_restored_instance_never_plays_the_logo_animation (tests/state.rs, state load via deserialize path, flags absent from saved JSON). Standalone: orb-ui untouched (main.rs/app.rs unchanged), so screenshots unchanged. cargo test: 792 passed, 0 failed; clippy -D warnings and nightly fmt --check clean.

Review round 1: fixed the doc reference in instance.rs (section 5.1), added the ordering risk to docs/plugin.md 5.1, rebased on main; cargo test now 813 passed, 0 failed, 5 ignored (earlier 792 was before the rebase); clippy and nightly fmt clean. Design: editor.rs calls App::enable_logo_anim only when OrbParams::take_intro() is true instead of adding an Embed method, because the Embed trait is only about song/mod/state, and the App already owns the animation switch (enable_logo_anim), so nothing in orb-ui changes and the standalone stays byte-identical. Residual risk: a host that opens the editor before its state.load on project reopen plays the animation once for an instance that is then restored.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The plugin editor plays the logo animation only at the first editor open of an instance that never had a state loaded. OrbParams has runtime flags restored (set by deserialize_fields) and intro_played; editor.rs calls App::enable_logo_anim only when take_intro() is true (no Embed method: orb-ui is untouched, so the standalone and its screenshots are unchanged). docs/plugin.md 5.1 documents the rule, the default-preset caveat and the editor-before-state.load ordering risk. Two new tests in crates/orb-plugin/tests/state.rs. cargo test 813 pass, 5 ignored, clippy and nightly fmt clean.
<!-- SECTION:FINAL_SUMMARY:END -->
