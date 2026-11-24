---
id: TASK-292
title: >-
  Plugin crash: egui-baseview frees textures before submit; use the
  pingw33n/egui-baseview fork
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-24T13:39:00Z'
updated_date: '2026-11-24T14:23:00Z'
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
- [x] #1 The workspace patches egui-baseview to github.com/pingw33n/egui-baseview at e3db196 and Cargo.lock resolves every user of it to the fork
- [x] #2 A regression test (or a documented reason none is possible plus the manual check) covers a texture freed in a frame that drew it
- [x] #3 docs/plugin.md and the Cargo.toml comment record the fork, its branch, its commit and why
- [x] #4 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [x] #5 The user has repeated the REAPER steps on Windows (a different folder as the mods folder) without a crash
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Workspace Cargo.toml: [patch.crates-io] egui-baseview to the fork at rev e3db196, with its comment; cargo update -p egui-baseview; check Cargo.lock and cargo tree -i egui-baseview.
2. Diff the fork against the crates.io 0.9.0 source: free after submit, present last.
3. Regression test crates/orb-plugin/tests/texture_free.rs: (a) egui-baseview's frame replayed on a headless wgpu device through egui-wgpu's Renderer, a texture drawn and freed in one egui frame, freed before the submit (0.9.0: validation error) and after it (the fork: none), skipped without an adapter; (b) the resolved egui-baseview's renderer.rs (cargo metadata) must call free_texture after .submit( and before .present( (fails on 0.9.0, passes on the fork).
4. docs/plugin.md section 10 paragraph on the fork (and section 1's mention).
5. cargo test --workspace, clippy -D warnings on macOS and x86_64-pc-windows-msvc (cargo xwin), nightly fmt --check; scripts/package.sh x86_64-pc-windows-msvc; commit with the task To Do; the user repeats the REAPER steps.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a85d8da279799fb64 (worktree .claude/worktrees/agent-a85d8da279799fb64), rebased on f1bf47d.

Evidence:
- Cargo.toml [patch.crates-io] egui-baseview = { git = "https://github.com/pingw33n/egui-baseview", rev = "e3db19695653fb05dead3916a553c514c1c88c23" }. cargo update -p egui-baseview: Removing egui-baseview v0.9.0, Adding egui-baseview v0.9.0 (https://github.com/pingw33n/egui-baseview?rev=e3db196...#e3db1969). Cargo.lock: source = "git+https://github.com/pingw33n/egui-baseview?rev=e3db19695653fb05dead3916a553c514c1c88c23#e3db19695653fb05dead3916a553c514c1c88c23", the registry checksum gone; only that entry changed (its dependencies list is the same). One egui-baseview package in the lock. cargo tree -i egui-baseview --workspace -e normal: egui-baseview (fork) <- nice-plug-egui v0.6.0 <- orb-plugin; no other user.
- diff -ru of the crates.io egui-baseview-0.9.0/src against the fork's checkout: one hunk in src/renderer/wgpu/renderer.rs render(): encoder.finish() and queue.submit moved before the textures_delta.free loop (free_texture), present(output_frame) last; plus a comment. egui-wgpu 0.37 Renderer::free_texture (renderer.rs 790) calls wgpu::Texture::destroy at once.
- Observation, not changed (the fork is not to be touched): render() returns early on a surface acquire of Occluded/Timeout (return) and Outdated/Suboptimal/Lost (reconfigure, return), before the free loop, in 0.9.0 and in the fork; the textures freed in such a frame are never destroyed until the renderer drops (a leak, no crash). Written in docs/plugin.md.
- Regression test crates/orb-plugin/tests/texture_free.rs, 3 tests: freeing_before_the_submit_fails_validation and freeing_after_the_submit_is_valid replay egui-baseview's frame (update_texture for textures_delta.set, update_buffers, a render pass on a 64x64 Rgba8Unorm offscreen target, render, free/submit in either order) on a headless wgpu device (InstanceDescriptor::new_without_display_handle, default adapter; skipped with a message when there is none, e.g. a CI runner without a GPU), under a Validation error scope; the egui frame draws a 4x4 managed texture and drops its last handle in the same frame, and the test asserts the frame both draws and frees it. Before submit: a validation error containing "destroyed"; after: none (Metal on this Mac). the_resolved_egui_baseview_frees_textures_after_the_submit takes egui-baseview's manifest_path from cargo metadata --offline and requires .submit( < free_texture( < .present( in src/renderer/wgpu/renderer.rs (each once).
- Before/after of the source test: with the patch line commented out (crates.io 0.9.0, --offline) it FAILED: "egui-baseview-0.9.0/src/renderer/wgpu/renderer.rs frees the textures before the submit ... (submit at 9905, free_texture at 9764, present at 9990)"; with the fork: ok. The two wgpu tests pass in both (they test the mechanism through egui-wgpu, not egui-baseview's code). egui-baseview's own renderer needs a baseview window and its surface, so no automatic test drives it; the REAPER steps (criterion 5) are the end-to-end check.
- Windows clippy needs CL=-Wno-incompatible-pointer-types (docs/building.md, aotuv_lancer_vorbis_sys with clang-cl 21) as scripts/package.sh sets it.

Checks (2026-11-24, this Mac): cargo test --workspace: 109 test binaries, 1109 passed, 0 failed, 5 ignored (texture_free 3 of them, the wgpu pair on Metal). cargo clippy --all-targets -D warnings: clean. CL=-Wno-incompatible-pointer-types XWIN_ACCEPT_LICENSE=1 cargo xwin clippy --target x86_64-pc-windows-msvc --all-targets -D warnings: clean. cargo +nightly fmt --all --check: clean. scripts/package.sh x86_64-pc-windows-msvc: dist/x86_64-pc-windows-msvc/OpenRB-338-2026.1124.1405-x86_64-pc-windows-msvc.zip (portable: OpenRB-338.exe, orb.exe, the single-file OpenRB-338.vst3, an empty OpenRB-338.toml, LICENSE, README.txt), for the user's REAPER check (criterion 5).

Sign-off: the user repeated the REAPER steps on Windows with OpenRB-338-2026.1124.1405-x86_64-pc-windows-msvc.zip (build of b35e118) and signed off on 2026-11-24: "the fix works". Main had not moved (f1bf47d); no rebase needed.

Review round 1 (send back, two text defects): (1) the Cargo.toml comment said egui-baseview 0.9.0 is MIT; its manifest (and the fork's, e3db196 Cargo.toml line 7) says license = "MIT OR Apache-2.0": fixed. (2) docs/plugin.md section 1 had one 141-column line (my edit left the next sentence on it): rewrapped to 100 columns, and it names the fork's license, MIT OR Apache-2.0 as upstream. Re-ran clippy -D warnings (macOS, x86_64-pc-windows-msvc) and nightly fmt --check: clean. No code change, so the tests were not re-run (1109 pass at the previous amend).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The plugin no longer aborts the host when a texture is freed in a frame that drew it. The workspace Cargo.toml patches egui-baseview under [patch.crates-io] to the user's fork github.com/pingw33n/egui-baseview, branch OpenRB-338, rev e3db19695653fb05dead3916a553c514c1c88c23 (upstream 1f645a3 = 0.9.0, with the texture free loop after queue.submit and present last). Cargo.lock resolves its one user, nice-plug-egui 0.6.0, to the fork.

Cause: egui-baseview 0.9.0 freed textures_delta.free before submit, and egui-wgpu's free_texture destroys the texture at once. So App::refresh_mods dropping the splash picture ("Use as mods folder" on another folder) produced a wgpu validation error and aborted REAPER (0xc0000409).

Test: crates/orb-plugin/tests/texture_free.rs. A source-order test on the resolved egui-baseview fails on crates.io 0.9.0 and passes on the fork. A headless wgpu pair replays the frame: freeing before submit fails validation with "destroyed", freeing after passes; it skips without an adapter. egui-baseview's own renderer needs a window, so the user's REAPER check on Windows is the end-to-end check: "the fix works", 2026-11-24.

Docs: docs/plugin.md section 10 "The egui-baseview fork" and section 1. It also records a leak the fork leaves: frames whose surface acquire fails return before the free.

Checks: cargo test --workspace 1109 pass, 0 fail, 5 ignored; clippy -D warnings clean on macOS and x86_64-pc-windows-msvc; nightly fmt --check clean.

Branch worktree-agent-a85d8da279799fb64, one commit. No upstream pull request.
<!-- SECTION:FINAL_SUMMARY:END -->
