//! The graphics adapter OpenRB-338 chooses (task-217).
//!
//! Why: on the user's Windows 10 PC (Intel HD Graphics 530, driver 31.0.101.2111 of 2022-07-19)
//! every start crashed with an access violation in `igvk64.dll`, Intel's Vulkan driver, about
//! 20 ms after egui-wgpu listed the adapters, before any window (the debug kit's report of
//! 2026-11-07; docs/building.md, "The graphics on Windows"). egui-wgpu's own choice takes the
//! first adapter of the preferred power, and the Vulkan one comes first. A crash inside a driver
//! cannot be caught and retried in the same process, so the choice is made before the device is.
//!
//! The rule ([`Rule`]):
//! - Windows with `WGPU_BACKEND` unset (or empty): the wgpu instance gets only the DX12 and GL
//!   backends (no Vulkan driver is loaded at all), and [`choose`] takes DX12 on a discrete GPU,
//!   then DX12 on an integrated GPU, then other DX12 hardware, then GL hardware, then the DX12
//!   software adapter ("Microsoft Basic Render Driver"), then GL in software;
//! - `WGPU_BACKEND` set: egui-wgpu's own choice among the backends it names (the way to try another
//!   backend, as the debug kit's `-Backend` does);
//! - macOS and Linux: egui-wgpu's own choice, unchanged.

use std::sync::Arc;

use eframe::egui_wgpu::{WgpuConfiguration, WgpuSetup};
use eframe::wgpu::{self, Backend, DeviceType};

/// How the graphics are chosen.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Rule {
    /// Windows with `WGPU_BACKEND` unset: DX12 and GL only, DX12 first ([`choose`]).
    PreferDx12,
    /// `WGPU_BACKEND` set (its value): egui-wgpu's own choice among those backends.
    Env(String),
    /// Not Windows: egui-wgpu's own choice.
    Default,
}

impl Rule {
    /// The rule for the system `os` (`std::env::consts::OS`) and the value of `WGPU_BACKEND`.
    /// An empty value counts as unset.
    pub fn new(os: &str, wgpu_backend: Option<&str>) -> Self {
        match wgpu_backend.map(str::trim).filter(|v| !v.is_empty()) {
            Some(v) => Rule::Env(v.to_string()),
            None if os == "windows" => Rule::PreferDx12,
            None => Rule::Default,
        }
    }

    /// The rule of this process.
    pub fn from_env() -> Self {
        let v = std::env::var("WGPU_BACKEND").ok();
        Rule::new(std::env::consts::OS, v.as_deref())
    }

    /// The run log's line for the rule.
    pub fn describe(&self) -> String {
        match self {
            Rule::PreferDx12 => "graphics: DirectX 12 preferred on Windows (a discrete GPU, \
                                 then an integrated one, then OpenGL, then the DirectX 12 \
                                 software adapter); Vulkan is not loaded; WGPU_BACKEND \
                                 overrides"
                .to_string(),
            Rule::Env(v) => format!("graphics: WGPU_BACKEND={v}, egui-wgpu's own choice"),
            Rule::Default => "graphics: egui-wgpu's own choice".to_string(),
        }
    }
}

/// The rank of an adapter under [`Rule::PreferDx12`], lower first; `None` for one never taken
/// (Vulkan, Metal, the browser's).
pub fn rank(backend: Backend, kind: DeviceType) -> Option<u8> {
    match (backend, kind) {
        (Backend::Dx12, DeviceType::DiscreteGpu) => Some(0),
        (Backend::Dx12, DeviceType::IntegratedGpu) => Some(1),
        (Backend::Dx12, DeviceType::VirtualGpu | DeviceType::Other) => Some(2),
        (Backend::Gl, DeviceType::DiscreteGpu | DeviceType::IntegratedGpu) => Some(3),
        (Backend::Gl, DeviceType::VirtualGpu | DeviceType::Other) => Some(4),
        (Backend::Dx12, DeviceType::Cpu) => Some(5),
        (Backend::Gl, DeviceType::Cpu) => Some(6),
        _ => None,
    }
}

/// The index of the adapter [`Rule::PreferDx12`] takes among `adapters` (backend and device
/// type of each, in the order wgpu lists them): the lowest [`rank`], the first listed of equal
/// rank. An error when none can be taken.
pub fn choose(adapters: &[(Backend, DeviceType)]) -> Result<usize, String> {
    adapters
        .iter()
        .enumerate()
        .filter_map(|(i, &(b, k))| rank(b, k).map(|r| (r, i)))
        .min()
        .map(|(_, i)| i)
        .ok_or_else(|| {
            if adapters.is_empty() {
                "no graphics adapter was found".to_string()
            } else {
                format!(
                    "no DirectX 12 or OpenGL graphics adapter among the {} found",
                    adapters.len()
                )
            }
        })
}

/// An adapter as the run log and errors name it.
fn name(a: &wgpu::Adapter) -> String {
    let i = a.get_info();
    format!("{:?} {:?} \"{}\"", i.backend, i.device_type, i.name)
}

/// The adapter selector of [`Rule::PreferDx12`]: among the adapters that can present to the
/// window's surface, [`choose`]'s; the choice goes to the run log before egui-wgpu makes the
/// device, so a crash in that driver still says which one was tried.
fn select(
    adapters: &[wgpu::Adapter],
    surface: Option<&wgpu::Surface<'_>>,
) -> Result<wgpu::Adapter, String> {
    let usable: Vec<&wgpu::Adapter> = adapters
        .iter()
        .filter(|a| surface.is_none_or(|s| a.is_surface_supported(s)))
        .collect();
    let kinds: Vec<(Backend, DeviceType)> = usable
        .iter()
        .map(|a| {
            let i = a.get_info();
            (i.backend, i.device_type)
        })
        .collect();
    match choose(&kinds) {
        Ok(i) => {
            crate::runlog::stage(&format!("graphics adapter chosen: {}", name(usable[i])));
            Ok(usable[i].clone())
        }
        Err(e) => {
            // adapters found, but none can draw to the window: say that, not choose's
            // "no graphics adapter was found"
            let e = if usable.is_empty() && !adapters.is_empty() {
                "none of the DirectX 12 or OpenGL graphics adapters can draw to the window"
                    .to_string()
            } else {
                e
            };
            let found: Vec<String> = adapters.iter().map(name).collect();
            let found = if found.is_empty() {
                "none".to_string()
            } else {
                found.join(", ")
            };
            Err(format!(
                "{e} (adapters found: {found}). Setting WGPU_BACKEND to vulkan, dx12 or \
                 opengl tries a backend by hand."
            ))
        }
    }
}

/// eframe's wgpu configuration for `rule`: egui-wgpu's default, and for
/// [`Rule::PreferDx12`] the DX12 and GL backends only with [`select`].
pub fn wgpu_options(rule: &Rule) -> WgpuConfiguration {
    let mut c = WgpuConfiguration::default();
    if *rule == Rule::PreferDx12
        && let WgpuSetup::CreateNew(n) = &mut c.wgpu_setup
    {
        n.instance_descriptor.backends = wgpu::Backends::DX12 | wgpu::Backends::GL;
        n.native_adapter_selector = Some(Arc::new(select));
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use Backend::{Dx12, Gl, Metal, Vulkan};
    use DeviceType::{Cpu, DiscreteGpu, IntegratedGpu};

    #[test]
    fn the_windows_10_pcs_list_takes_dx12_on_the_intel_gpu() {
        // the order egui_wgpu listed on the user's PC (2026-11-07): Vulkan HD 530, Dx12 HD 530,
        // Dx12 Microsoft Basic Render Driver, Gl HD 530
        let list = [
            (Vulkan, IntegratedGpu),
            (Dx12, IntegratedGpu),
            (Dx12, Cpu),
            (Gl, IntegratedGpu),
        ];
        assert_eq!(choose(&list), Ok(1));
    }

    #[test]
    fn a_discrete_gpu_comes_before_an_integrated_one() {
        // the development machine: Intel UHD 630 and Quadro P2000, both listed per backend
        let list = [
            (Vulkan, IntegratedGpu),
            (Vulkan, DiscreteGpu),
            (Dx12, IntegratedGpu),
            (Dx12, DiscreteGpu),
            (Dx12, Cpu),
            (Gl, DiscreteGpu),
        ];
        assert_eq!(choose(&list), Ok(3));
        // two of the same rank: the first listed
        assert_eq!(choose(&[(Dx12, DiscreteGpu), (Dx12, DiscreteGpu)]), Ok(0));
    }

    #[test]
    fn gl_then_software_when_there_is_no_dx12_hardware() {
        assert_eq!(choose(&[(Dx12, Cpu), (Gl, IntegratedGpu)]), Ok(1));
        assert_eq!(choose(&[(Gl, Cpu), (Dx12, Cpu)]), Ok(1));
        assert_eq!(choose(&[(Vulkan, DiscreteGpu), (Gl, Cpu)]), Ok(1));
        // GL only, software only
        assert_eq!(choose(&[(Gl, IntegratedGpu)]), Ok(0));
        assert_eq!(choose(&[(Dx12, Cpu)]), Ok(0));
    }

    #[test]
    fn vulkan_or_metal_only_or_nothing_is_an_error() {
        assert_eq!(
            choose(&[(Vulkan, IntegratedGpu), (Metal, DiscreteGpu)]),
            Err("no DirectX 12 or OpenGL graphics adapter among the 2 found".to_string())
        );
        assert_eq!(
            choose(&[]),
            Err("no graphics adapter was found".to_string())
        );
    }

    #[test]
    fn the_rule_follows_the_system_and_wgpu_backend() {
        assert_eq!(Rule::new("windows", None), Rule::PreferDx12);
        assert_eq!(Rule::new("windows", Some("")), Rule::PreferDx12);
        assert_eq!(Rule::new("windows", Some(" ")), Rule::PreferDx12);
        assert_eq!(
            Rule::new("windows", Some("vulkan")),
            Rule::Env("vulkan".to_string())
        );
        assert_eq!(Rule::new("linux", None), Rule::Default);
        assert_eq!(Rule::new("macos", None), Rule::Default);
        assert_eq!(
            Rule::new("macos", Some("metal")),
            Rule::Env("metal".to_string())
        );
        assert!(Rule::PreferDx12.describe().contains("DirectX 12 preferred"));
        assert_eq!(
            Rule::Env("dx12".into()).describe(),
            "graphics: WGPU_BACKEND=dx12, egui-wgpu's own choice"
        );
    }

    #[test]
    fn only_prefer_dx12_changes_egui_wgpus_configuration() {
        let backends = |c: &WgpuConfiguration| match &c.wgpu_setup {
            WgpuSetup::CreateNew(n) => (
                n.instance_descriptor.backends,
                n.native_adapter_selector.is_some(),
            ),
            WgpuSetup::Existing(_) => panic!("an existing setup"),
        };
        assert_eq!(
            backends(&wgpu_options(&Rule::PreferDx12)),
            (wgpu::Backends::DX12 | wgpu::Backends::GL, true)
        );
        let default = backends(&WgpuConfiguration::default());
        assert_eq!(backends(&wgpu_options(&Rule::Default)), default);
        assert_eq!(
            backends(&wgpu_options(&Rule::Env("vulkan".into()))),
            default
        );
    }
}
