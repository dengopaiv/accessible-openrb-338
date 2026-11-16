//! OpenRB-338 as a VST3 and CLAP instrument plugin (milestone m-7; task-185's decisions, task-197
//! the core): the whole ReBirth (two 303s, 808, 909, effects, mixer) in one instance, on
//! nice-plug. `docs/plugin.md` describes it.
//!
//! - [`instance`]: the parameters, the state and the main thread's calls (load a song, set a mod,
//!   edit a pattern), and the UI half of the player.
//! - [`core`]: the process side, the engine in 64-frame blocks for any host buffer size.
//! - [`resample`]: the resampler for host rates other than 44.1 kHz.
//! - [`params`]: the parameter table.
//!
//! Not yet: host sync and pattern selection by notes (task-198), note mode (task-200), the
//! editor (task-201), packaging (task-204).

pub mod core;
pub mod instance;
pub mod params;
pub mod resample;

use std::num::NonZeroU32;
use std::sync::Arc;

use nice_plug::prelude::*;

use crate::core::Core;
use crate::instance::OrbParams;

/// The plugin's name, permanent after the first release (task-185).
pub const NAME: &str = "OpenRB-338";
/// The vendor, permanent.
pub const VENDOR: &str = "OpenRB-338";
/// The CLAP id, permanent.
pub const CLAP_ID: &str = "io.github.pingw33n.openrb-338";
/// The VST3 class id, the GUID EFAB08AF-B39D-4399-8A80-3DC1F84ED3D6 made once with `uuidgen`
/// (2026-11-16, task-197). It never changes: hosts find the plugin in saved projects by it.
pub const VST3_CLASS_ID: [u8; 16] = [
    0xEF, 0xAB, 0x08, 0xAF, 0xB3, 0x9D, 0x43, 0x99, 0x8A, 0x80, 0x3D, 0xC1, 0xF8, 0x4E, 0xD3, 0xD6,
];
/// The project's page.
pub const URL: &str = "https://github.com/pingw33n/OpenRB-338";

/// One plugin instance: its parameters and main-thread state, and the process side once
/// activated.
pub struct OrbPlugin {
    params: Arc<OrbParams>,
    core: Option<Core>,
}

impl Default for OrbPlugin {
    fn default() -> Self {
        OrbPlugin {
            params: Arc::new(OrbParams::new()),
            core: None,
        }
    }
}

impl OrbPlugin {
    /// The parameters and the main thread's calls (the editor's handle).
    pub fn orb_params(&self) -> &Arc<OrbParams> {
        &self.params
    }
}

impl Plugin for OrbPlugin {
    const NAME: &'static str = NAME;
    const VENDOR: &'static str = VENDOR;
    const URL: &'static str = URL;
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type Editor = ();
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    /// A fresh player and engine from the state and the parameters, for the host's rate: every
    /// activation starts from the same state, so an offline bounce does not depend on what
    /// played before it.
    fn activate(
        &mut self,
        _layout: &AudioIOLayout,
        config: &BufferConfig,
        context: &mut impl ActivateContext<Self>,
    ) -> bool {
        let rate = config.sample_rate.round() as u32;
        // the old player is dropped here, on the main thread
        self.core = None;
        let core = instance::core(&self.params, rate);
        context.set_latency_samples(core.latency());
        self.core = Some(core);
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // host notes select patterns and play the devices in later tasks (198, 200)
        while context.next_event().is_some() {}
        let values = self.params.values();
        let out = buffer.as_slice();
        match (&mut self.core, out) {
            (Some(core), [l, r, ..]) => core.process(l, r, &values),
            (_, out) => {
                for ch in out.iter_mut() {
                    ch.fill(0.0);
                }
            }
        }
        ProcessStatus::KeepAlive
    }
}

impl ClapPlugin for OrbPlugin {
    const CLAP_ID: &'static str = CLAP_ID;
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("ReBirth RB-338 2.01: two TB-303s, a TR-808 and a TR-909");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = Some(URL);
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for OrbPlugin {
    const VST3_CLASS_ID: [u8; 16] = VST3_CLASS_ID;
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Synth];
}

nice_export_clap!(OrbPlugin);
nice_export_vst3!(OrbPlugin);
