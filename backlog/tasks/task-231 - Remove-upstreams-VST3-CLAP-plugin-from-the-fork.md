---
id: TASK-231
title: Remove upstream's VST3/CLAP plugin from the fork
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-18T19:10:00Z'
updated_date: '2026-11-26T12:28:00Z'
labels:
  - tools
  - ui
  - docs
milestone: m-7
dependencies:
  - TASK-230
references:
  - docs/plugin.md
  - crates/orb-plugin
priority: medium
type: chore
ordinal: 249000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: the user decided on 2026-11-18 (task-229, question 1) to leave upstream's VST3/CLAP plugin out of the fork. The options put to them were to ship it in the portable zip, to keep the code but leave it out of the zip, or to leave it out; they chose to leave it out. task-230 merges upstream's main with the plugin in it, so later merges have the right base; this task then removes it. AGENTS.md "This fork": an upstream part the fork does not need may be removed, each removal its own task, decided by the user, with the reason in its description.

What goes: the crate crates/orb-plugin (workspace member), xtask (the plugin bundler, nice-plug-xtask), vendor/raw-window-metal and its [patch.crates-io] entry (needed only by the plugin's macOS editor), scripts/validate-plugin.sh, the plugin steps of scripts/build.sh, scripts/package.sh (the plugins/ folder and the README.txt plugin lines), scripts/package-installer.sh and scripts/installer/*, scripts/package-pkg.sh, scripts/release-assets.sh and the workflows (.github/workflows/ci.yml, release.yml), docs/plugin.md, assets/steinberg (the VST logo, used only for the plugin), and the plugin's lines in README.md, docs/building.md and the other docs. Cargo.lock loses the crates only the plugin used.

What stays, and why: the code upstream shares between the plugin and the app (orb-ui's embedding seam such as embed.rs and session.rs, the standalone/default feature split, orb-player's MIDI layer and host event code, orb-engine's note-mode paths). Removing shared code would make every later upstream merge conflict in the crates the fork edits most; it is dead weight, not a cost to the user. Something in orb-ui that only the plugin shows (for example Help > Plugin Usage, plugin_usage.rs, if the standalone app never shows it) goes if it can go without touching shared code; otherwise it stays and the notes say so. If something in this split turns out to be the user's choice, it becomes a Decide: task rather than being guessed.

Also from task-230's review: the merge made orb-ui's App `Send` for the plugin's editor (crates/orb-ui/src/system_dialog.rs: `unsafe impl Send for Parent` and the `+ Send` bound on Picker; crates/orb-ui/tests/system_dialog.rs moved from Rc<RefCell> to Arc<Mutex> for it): with the plugin gone, put Picker back without `+ Send`, drop the unsafe impl and the test change, unless something else needs them. In menu.rs, menu_items' `if plugin { return; }` in Options sits before the screen reader items; with the plugin gone the plugin branches of menu_items can go only where that does not touch shared code (see above). The README's short "The plugin" section and orb-ui's plugin_usage test of the README link go together. Until this task is done the Windows zip build needs the plugin bundles (scripts/package.sh), so no fork release is made before it.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 crates/orb-plugin, xtask, vendor/raw-window-metal and scripts/validate-plugin.sh are gone, and the workspace Cargo.toml and Cargo.lock no longer name them or the crates only they used
- [ ] #2 scripts/package.sh x86_64-pc-windows-msvc builds the portable zip with no plugins/ folder and no plugin lines in its README.txt
- [ ] #3 No workflow, script or doc still refers to the plugin bundles, the validators or docs/plugin.md (git grep -i -E 'orb-plugin|vst3|clap|validate-plugin' finds only the merged history notes and CHANGELOG lines that say it was removed)
- [ ] #4 The standalone app is unchanged: cargo build --release, cargo test --release, clippy -D warnings and nightly fmt --check pass
- [ ] #5 CHANGELOG.md's next section says, in plain language, that the plugin from upstream is not part of the fork
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Supervisor, 2026-11-26 (merge of upstream f83c54f, task-233): upstream's plugin grew since this task was written, and all of it came in with the merge. CLAP is gone (VST3 only, stock nice-plug again, upstream's task-278); new plugin parts to remove as well: the egui-baseview fork dependency (task-292), crates/orb-ui/src/routing_window.rs and Options > Output Routing... (plugin only, task-300), the per-device and per-drum outputs in orb-player (tasks 256, 300; check what the standalone shares before removing engine code), the REAPER render tests behind ORB_REAPER (tests of orb-plugin and orb-player, task-244, 277, 305), the texture_free and exports tests of orb-plugin (task-302), and the plugin log (task-289: logs\plugin.log). In scripts/package.sh the Windows zip now copies the plugin as the single file OpenRB-338.vst3 beside the exe; the README.txt plugin and 'Portable mode' paragraphs name it. upstream's portable mode (portable.rs folder_of) knows the plugin's bundle path: that can stay, as shared code.
<!-- SECTION:NOTES:END -->
