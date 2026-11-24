---
id: TASK-292
title: >-
  Plugin crash: egui-baseview frees textures before submit; use the
  pingw33n/egui-baseview fork
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-24T13:39:00Z'
labels:
  - player
  - ui
milestone: m-5
dependencies: []
references:
  - Cargo.toml
  - crates/orb-plugin/Cargo.toml
  - crates/orb-ui/src/menu.rs
  - docs/plugin.md
priority: high
type: bug
ordinal: 100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user reproduced a host crash on Windows on 2026-11-24 while testing task-289.
- **Steps:** in REAPER 7.8.6 with the plugin's window open, File > Open, go to a different folder than the current mods folder, and click "Use as mods folder".
- **Result:** REAPER dies with 0xc0000409 / FAST_FAIL_FATAL_APP_EXIT.
- **plugin.log:** wgpu "Validation Error ... In Queue::submit ... In a draw command, kind: Draw ... Texture with 'egui_texid_Managed(1)' label has been destroyed", then "panic in a function that cannot unwind".
- The standalone app survives the same steps.

**Cause** (the supervisor's analysis):
- `App::refresh_mods` (crates/orb-ui/src/menu.rs around 345) drops the mod's splash texture (`self.splash = None`) in a frame that has already drawn it. egui therefore lists it in that frame's `textures_delta.free`.
- egui-baseview 0.9.0 (crates.io; upstream codeberg.org/RustAudio/egui-baseview, commit 1f645a3) frees `textures_delta.free` in src/renderer/wgpu/renderer.rs around 280, before `queue.submit`.
- egui-wgpu's `free_texture` destroys the texture at once, so the submitted command buffer still draws a destroyed texture.
- eframe, which the standalone app uses, frees after submit. Choosing the same folder again does not drop the texture, so it does not crash.
- Any texture the plugin drops in a frame that drew it is exposed. The same crash may be behind GitHub issue 23 (Ableton Live crashes), though that is unconfirmed.

**The fix, the user's decision of 2026-11-24:** use a fork. The supervisor made it: github.com/pingw33n/egui-baseview, branch OpenRB-338, commit e3db196 on 1f645a3 (the published 0.9.0). It moves the free loop after `queue.submit` and keeps `present` last.
- Use it through `[patch.crates-io]` in the workspace Cargo.toml, next to the existing raw-window-metal patch, pinned to that commit: `egui-baseview = { git = "https://github.com/pingw33n/egui-baseview", rev = "e3db19695653fb05dead3916a553c514c1c88c23" }`.
- Cargo.lock must resolve egui-baseview to the fork. Check that every crate that depends on it gets the patched one, e.g. nice-plug-egui.
- Document it where docs/plugin.md lists the patched dependencies, as the nice-plug fork was documented (the comment above [patch.crates-io] too).

**Regression test:** a test fails before the fix and passes after: a texture is drawn and freed in the same frame through egui-baseview's wgpu renderer path, or the closest test the code allows (e.g. a headless wgpu device if one is available on CI, otherwise a test of the order of operations). If no automatic test can reproduce it, say so in the notes. In that case the REAPER steps above, done by the user on Windows, are the check.

**Do not** open an upstream pull request. The user has not asked for one; the supervisor will ask.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The workspace patches egui-baseview to github.com/pingw33n/egui-baseview at e3db196 and Cargo.lock resolves every user of it to the fork
- [ ] #2 A regression test (or a documented reason none is possible plus the manual check) covers a texture freed in a frame that drew it
- [ ] #3 docs/plugin.md and the Cargo.toml comment record the fork, its branch, its commit and why
- [ ] #4 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [ ] #5 The user has repeated the REAPER steps on Windows (a different folder as the mods folder) without a crash
<!-- AC:END -->
