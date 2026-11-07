---
id: TASK-217
title: >-
  Windows: prefer DirectX 12 over Vulkan, Intel's Vulkan driver crashes the
  start
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-07T13:36:00Z'
labels:
  - ui
  - docs
milestone: m-7
dependencies:
  - TASK-214
references:
  - crates/orb-ui/src/main.rs
  - crates/orb-ui/src/runlog.rs
  - docs/building.md
  - tools/win_diag.ps1
priority: high
type: bug
ordinal: 235000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: OpenRB-338 does not start on the user's Windows 10 PC. The debug kit's first report (task-215; 2026-11-07, `diag-reports\win_diag-20261107-153117.txt`, kept out of git because it holds the user's Windows account name) shows it, 3 runs of 3:
- the PC: HP EliteDesk 800 G2 SFF, Windows 10 Pro 22H2 build 19045.7807, Intel Core i5-6500, Intel HD Graphics 530 (PCI device 0x1912) with driver 31.0.101.2111 of 2022-07-19, a 5120 by 2880 screen at 60 Hz, NVDA running, vcruntime140 present;
- each run: the audio output starts (Realtek, 44100 Hz, f32), run_native starts, egui_wgpu lists 4 adapters (Vulkan Intel HD 530; Dx12 Intel HD 530; Dx12 "Microsoft Basic Render Driver", device type Cpu; Gl Intel HD 530, 4.6.0 Build 31.0.101.2111), and 20 ms later the process dies with an access violation in `igvk64.dll` (Intel's Vulkan driver, iigd_dch.inf_amd64_51f685305808e3a5) at +0x64EA72 (writing; runs 1 and 3) or +0x630E61 (reading; run 2). No window, no frame. The Application log's event 1000 names the same module, version 31.0.101.2111.
- egui-wgpu 0.37 (`egui_wgpu::RenderState::create`, lib.rs) enumerates the adapters of `instance_descriptor.backends` (default `Backends::from_env()` or `PRIMARY | GL`), takes one with `request_adapter` by power preference (HighPerformance), which here is the Vulkan one (it comes first), then calls `adapter.request_device`. The crash is between the adapter list and `log_adapter_info`, so in the Vulkan device's creation (or the surface's), inside the driver. A crash inside a driver cannot be caught and retried in the same process, so the fix must choose another backend before the device is made.

What: on Windows, when `WGPU_BACKEND` is not set, OpenRB-338 chooses its graphics itself instead of egui-wgpu's default:
- the instance's backends are DX12 and GL only (no Vulkan instance is made, so no Vulkan driver is loaded at all; Vulkan stays reachable with `WGPU_BACKEND=vulkan`);
- `native_adapter_selector` (`egui_wgpu::WgpuSetupCreateNew`, set through `eframe::NativeOptions::wgpu_options`) takes, in order: a DX12 adapter that is a discrete GPU, a DX12 integrated GPU, a GL adapter, the DX12 software adapter (device type Cpu, "Microsoft Basic Render Driver"); an error naming the adapters found when there is none;
- with `WGPU_BACKEND` set, egui-wgpu's own choice is kept as it is (the variable stays the way to try another backend: the debug kit's `-Backend`);
- macOS and Linux unchanged.
The selection order is a pure function of the adapters' (backend, device type) list, unit tested. The run log (task-214) gets a stage naming the rule used ("graphics: DX12 preferred on Windows" or "graphics: WGPU_BACKEND=..."), before the adapter is made, so a crash in the next driver still says which one was tried.

The user's decision: this changes behavior also with screen reader mode off (the fork's rule is "as upstream" there); the user asked for the fix on 2026-11-07 ("Write up a way to fix the error"). Before the code, the user may confirm on the Windows 10 PC that `win_diag.ps1 -Backend dx12` starts 3 of 3 (the same choice made by hand); if DX12 fails there too, `-Backend opengl` is the next to try, and the order above changes accordingly.

Workaround until this is done: `setx WGPU_BACKEND dx12` once on that PC (a user variable; programs started afterwards see it).

Docs: docs/building.md, the run log section: the Windows 10 finding (the report's numbers above) and the default graphics choice on Windows.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 On Windows with WGPU_BACKEND unset, the wgpu instance has only the DX12 and GL backends and the adapter is chosen DX12 discrete, DX12 integrated, GL, DX12 software, in that order; with WGPU_BACKEND set, egui-wgpu's default choice is kept; macOS and Linux are unchanged
- [ ] #2 The choice is a pure function of the adapters' backend and device type, with unit tests for: the Windows 10 PC's list (DX12 Intel HD 530 chosen), a discrete plus integrated list, GL only, software only, and an empty list (an error naming that no adapter was found)
- [ ] #3 The run log has a stage naming the rule (DX12 preferred on Windows, or WGPU_BACKEND=VALUE) before the device is made, and the window-created stage names the adapter chosen
- [ ] #4 On this development machine (Windows 11, Intel UHD 630 and Quadro P2000) OpenRB-338 starts with the default (the run log shows a Dx12 adapter) and with WGPU_BACKEND=vulkan (a Vulkan adapter): the run logs in the notes
- [ ] #5 On the user's Windows 10 PC, win_diag.ps1 with no -Backend gives 3 runs of 3 started (the user's check; the report's summary in the notes)
- [ ] #6 docs/building.md records the Windows 10 finding and the default graphics choice on Windows; cargo test -p orb-ui, clippy -D warnings and nightly fmt --check pass
<!-- AC:END -->
