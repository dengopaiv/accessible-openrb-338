---
id: TASK-260
title: >-
  Plugin on Linux: Bitwig shows the editor black (VST3) or stale screen content
  (CLAP) on a software-GL VM
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-21T08:29:00Z'
labels:
  - player
  - ui
milestone: m-7
dependencies: []
references:
  - /Users/pingw33n/win_exchange/log/engine.log
  - /Users/pingw33n/win_exchange/log/BitwigStudio.log
  - docs/plugin.md
  - docs/building.md
  - crates/orb-plugin/src/editor.rs
  - crates/orb-ui/src/main.rs
priority: medium
type: bug
ordinal: 253000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Reported by the user on 2026-11-21, the first test of the plugin in Bitwig on Linux.

**Setup**
- Ubuntu in a VirtualBox VM (Mesa 26.0.8, no GPU), opened over RDP from Windows.
- An Xfce X11 session: `XDG_SESSION_TYPE=x11`.
- OpenGL is llvmpipe (software): `glxinfo -B` reports llvmpipe (LLVM 21.1.8), core 4.5, not accelerated. Vulkan has a software ICD; Bitwig itself uses Vulkan "llvmpipe" through Skia.
- The plugin is from the package OpenRB-338-2026.1118.0919-x86_64-unknown-linux-gnu. The user thinks later builds changed nothing relevant.

**Symptoms**
- **VST3:** the editor window shows the menu bar (File, Mods, Options, Help) and the mod's splash image, but the panel area is black.
- **CLAP:** the window shows whatever was on screen when it opened. Sometimes it shows only the menu and splash, as the VST3 does; it varies between openings.
- Moving the mouse, clicking or resizing doesn't make the panel appear.
- **The standalone OpenRB-338 works in the same VM and session,** with the default backend and with `WGPU_BACKEND=gl` alike. So drawing on wgpu's GL backend on llvmpipe works in eframe's window; the problem is specific to the plugin's window path (nice-plug-egui 0.6 → egui-baseview 0.9 → wgpu 30, an X11 child window that Bitwig's plugin host embeds).

**Evidence**
- Bitwig's logs are at /Users/pingw33n/win_exchange/log (engine.log, BitwigStudio.log).
- When the editor opens, the plugin host (BitwigPluginHost-X64-AVX2) logs `libEGL warning: DRI3 error: Could not get DRI3 device` and `Ensure your X server supports DRI3 to get accelerated rendering`. So the plugin's wgpu took the GL (EGL) backend, not Vulkan.
- egui-baseview's renderer (`egui-baseview-0.9.0/src/renderer/wgpu/renderer.rs`) builds its instance with `InstanceDescriptor::new_with_display_handle`. That uses all backends and does not read `WGPU_BACKEND` (wgpu reads the environment only via `with_env()`). The standalone chooses its backends itself: `crates/orb-ui/src/main.rs` `wgpu_backends`, docs/building.md.
- `NICE_LOG=$HOME/orb-plugin.log` set on bitwig-studio produced no file. Bitwig's plugin host apparently doesn't pass it through. Logging needs another route: a file the plugin opens itself, gated by an `ORB_*` variable or a settings flag, listed in docs/building.md's variable table.
- The black panel with a visible splash suggests that some draws reach the screen but most frames are not presented, or are presented to the wrong surface. The panel's SVG parts are placeholders rasterized at the end of a frame (`orb-ui` `part_texture`/`flush_part_textures`), so they need later frames to be presented. CLAP's stale content says no frame was presented at all in that case.

**What to do**
1. Add a plugin log that works inside a host. Through it, report the wgpu adapter (backend, name, device type) and the surface format and present mode, plus every wgpu error and failed surface acquire or present. Make it available in release builds.
2. Find the cause: why frames are not presented in an embedded baseview X11 window on llvmpipe while eframe's window works. Candidates:
   - EGL on a child window of a foreign parent without DRI3 (the software present path);
   - the surface configured before the window is mapped or given its size;
   - surface acquire errors (Outdated, Timeout) being swallowed;
   - the present mode.
   Check egui-baseview's and baseview's X11 code.
3. Fix it. Likely options:
   - choose the backends the way the standalone does: honor `WGPU_BACKEND`, and on Linux prefer Vulkan when an adapter exists. This may need a patched or forked egui-baseview, or nice-plug-egui's options if they expose the instance descriptor;
   - handle surface errors with reconfigure and retry;
   - delay the surface configuration until the window is mapped.
   If the fix needs vendoring or forking a dependency, or a choice of default backend, create a `Decide:` task for the user first.
4. The agent cannot run Bitwig or the VM itself. Give the user a test build: CI on a pushed branch, or `scripts/package.sh x86_64-unknown-linux-gnu` with Docker. Write the exact steps for the user's check, including the log variable. The user checks in Bitwig in the VM before the agent review.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The plugin can write a log file inside a host (release build), enabled by a documented ORB_* variable or setting; it records the wgpu adapter, surface configuration and surface/present errors
- [ ] #2 The cause of the unpresented frames in Bitwig on the user's software-GL Linux VM is found and written in the task notes with the log evidence
- [ ] #3 The fix makes the VST3 and CLAP editors draw the full panel in Bitwig on the user's VM, confirmed by the user
- [ ] #4 The plugin's wgpu backend choice is documented in docs/plugin.md (and follows WGPU_BACKEND like the standalone, unless a Decide task chose otherwise)
- [ ] #5 macOS and Windows editors still work: the existing plugin tests pass and the user checks the macOS plugin in one host
<!-- AC:END -->
