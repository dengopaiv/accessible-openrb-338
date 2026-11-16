//! task-197 AC #1 and #2: the plugin library exports a CLAP and a VST3 entry with the plugin's
//! identity, and a host that loads it through the CLAP API, restores a state with a song and
//! pattern mode, activates it at 44.1 kHz and processes with the host playing, gets the pattern
//! playback: the
//! same samples as the process side run directly.
//!
//! The test builds the library (`cargo build -p orb-plugin --lib`, the cdylib the bundles hold;
//! `cargo xtask bundle orb-plugin` lays out the bundles from the same file) and loads it with the
//! dynamic loader. It is a minimal host of its own; loading in Reaper and the validators is
//! task-204's.

mod common;

use std::collections::BTreeMap;
use std::ffi::{CStr, CString, c_char, c_void};
use std::path::PathBuf;
use std::ptr::{null, null_mut};
use std::sync::OnceLock;

use clap_sys::audio_buffer::clap_audio_buffer;
use clap_sys::entry::clap_plugin_entry;
use clap_sys::events::{
    CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_NOTE_ON, CLAP_EVENT_TRANSPORT, CLAP_TRANSPORT_HAS_TEMPO,
    CLAP_TRANSPORT_IS_PLAYING, clap_event_header, clap_event_note, clap_event_transport,
    clap_input_events, clap_output_events,
};
use clap_sys::ext::latency::{CLAP_EXT_LATENCY, clap_plugin_latency};
use clap_sys::ext::state::{CLAP_EXT_STATE, clap_plugin_state};
use clap_sys::factory::plugin_factory::{CLAP_PLUGIN_FACTORY_ID, clap_plugin_factory};
use clap_sys::host::clap_host;
use clap_sys::process::{CLAP_PROCESS_ERROR, clap_process};
use clap_sys::stream::clap_istream;
use clap_sys::version::CLAP_VERSION;
use common::{activate, host, with_song};
use nice_plug::prelude::{Params, Vst3Plugin};
use orb_plugin::OrbPlugin;

/// The plugin library, built once per test process.
fn library() -> &'static PathBuf {
    static LIB: OnceLock<PathBuf> = OnceLock::new();
    LIB.get_or_init(|| {
        let root = common::root();
        let release = !cfg!(debug_assertions);
        let mut cmd = std::process::Command::new(env!("CARGO"));
        cmd.args(["build", "-p", "orb-plugin", "--lib"])
            .current_dir(&root);
        if release {
            cmd.arg("--release");
        }
        assert!(cmd.status().unwrap().success(), "cargo build -p orb-plugin");
        let target = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("target"));
        target
            .join(if release { "release" } else { "debug" })
            .join(format!(
                "{}orb_plugin{}",
                std::env::consts::DLL_PREFIX,
                std::env::consts::DLL_SUFFIX
            ))
    })
}

fn load() -> libloading::Library {
    unsafe { libloading::Library::new(library()) }.expect("the plugin library loads")
}

fn text(p: *const c_char) -> String {
    unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
}

fn chars(b: &[c_char]) -> String {
    let b: Vec<u8> = b
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();
    String::from_utf8_lossy(&b).into_owned()
}

unsafe extern "C" fn host_extension(_: *const clap_host, _: *const c_char) -> *const c_void {
    null()
}
unsafe extern "C" fn host_request(_: *const clap_host) {}

unsafe extern "C" fn no_events(_: *const clap_input_events) -> u32 {
    0
}
unsafe extern "C" fn no_event(_: *const clap_input_events, _: u32) -> *const clap_event_header {
    null()
}
unsafe extern "C" fn drop_event(_: *const clap_output_events, _: *const clap_event_header) -> bool {
    true
}

/// A state blob read by the plugin.
struct Reader {
    data: Vec<u8>,
    pos: usize,
}

unsafe extern "C" fn read(s: *const clap_istream, buf: *mut c_void, size: u64) -> i64 {
    let r = unsafe { &mut *((*s).ctx as *mut Reader) };
    let n = (size as usize).min(r.data.len() - r.pos);
    unsafe {
        std::ptr::copy_nonoverlapping(r.data[r.pos..].as_ptr(), buf as *mut u8, n);
    }
    r.pos += n;
    n as i64
}

/// nice-plug's CLAP state of an instance: the length (u64, little-endian), then the
/// `PluginState` as JSON, its parameters by id and its persisted fields.
fn state_blob(p: &orb_plugin::instance::OrbParams) -> Vec<u8> {
    let params: BTreeMap<String, serde_json::Value> = p
        .ids()
        .iter()
        .zip(p.values())
        .map(|(id, v)| (id.clone(), serde_json::json!({ "i32": v })))
        .collect();
    let fields: BTreeMap<String, String> = p.serialize_fields();
    let json = serde_json::to_vec(&serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "params": params,
        "fields": fields,
    }))
    .unwrap();
    let mut blob = (json.len() as u64).to_le_bytes().to_vec();
    blob.extend(json);
    blob
}

/// The CLAP tests init and deinit the one entry of the library: one at a time.
static CLAP_ENTRY: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn the_clap_entry_has_the_identity_and_plays_a_restored_song() {
    const FRAMES: usize = 30_000;
    const BUF: usize = 512;
    let _entry = CLAP_ENTRY.lock().unwrap_or_else(|e| e.into_inner());
    let lib = load();
    let entry: libloading::Symbol<*const clap_plugin_entry> =
        unsafe { lib.get(b"clap_entry\0") }.unwrap();
    let entry = unsafe { &**entry };
    let path = CString::new(library().to_string_lossy().as_bytes()).unwrap();
    unsafe {
        assert!((entry.init.unwrap())(path.as_ptr()));
        let factory = (entry.get_factory.unwrap())(CLAP_PLUGIN_FACTORY_ID.as_ptr())
            as *const clap_plugin_factory;
        assert!(!factory.is_null());
        let f = &*factory;
        assert_eq!((f.get_plugin_count.unwrap())(factory), 1);
        let d = &*(f.get_plugin_descriptor.unwrap())(factory, 0);
        assert_eq!(text(d.id), "io.github.pingw33n.openrb-338");
        assert_eq!(text(d.name), "OpenRB-338");
        assert_eq!(text(d.vendor), "OpenRB-338");
        assert_eq!(text(d.url), "https://github.com/pingw33n/OpenRB-338");
        let mut features = Vec::new();
        let mut fp = d.features;
        while !(*fp).is_null() {
            features.push(text(*fp));
            fp = fp.add(1);
        }
        // task-185: Instrument|Synth; stereo is the port layout
        assert_eq!(features, ["instrument", "synthesizer", "stereo"]);

        let host_info = clap_host {
            clap_version: CLAP_VERSION,
            host_data: null_mut(),
            name: c"orb-plugin test".as_ptr(),
            vendor: c"OpenRB-338".as_ptr(),
            url: c"".as_ptr(),
            version: c"0".as_ptr(),
            get_extension: Some(host_extension),
            request_restart: Some(host_request),
            request_process: Some(host_request),
            request_callback: Some(host_request),
        };
        let plugin = (f.create_plugin.unwrap())(factory, &host_info, d.id);
        assert!(!plugin.is_null());
        let pl = &*plugin;
        assert!((pl.init.unwrap())(plugin));

        // the state of an instance with a song in pattern mode
        let song = with_song("full_mix");
        let state = (pl.get_extension.unwrap())(plugin, CLAP_EXT_STATE.as_ptr())
            as *const clap_plugin_state;
        assert!(!state.is_null());
        let mut reader = Reader {
            data: state_blob(&song),
            pos: 0,
        };
        let stream = clap_istream {
            ctx: &mut reader as *mut Reader as *mut c_void,
            read: Some(read),
        };
        assert!(((*state).load.unwrap())(plugin, &stream), "the state loads");

        assert!((pl.activate.unwrap())(plugin, 44_100.0, 1, BUF as u32));
        let latency = (pl.get_extension.unwrap())(plugin, CLAP_EXT_LATENCY.as_ptr())
            as *const clap_plugin_latency;
        assert_eq!(((*latency).get.unwrap())(plugin), 64);
        assert!((pl.start_processing.unwrap())(plugin));
        let (mut l, mut r) = (vec![0f32; FRAMES], vec![0f32; FRAMES]);
        let in_events = clap_input_events {
            ctx: null_mut(),
            size: Some(no_events),
            get: Some(no_event),
        };
        let out_events = clap_output_events {
            ctx: null_mut(),
            try_push: Some(drop_event),
        };
        // the host plays, without a tempo or a position: pattern mode plays the song
        // free-running at its tempo (common::PLAYING)
        let transport = clap_event_transport {
            header: clap_event_header {
                size: std::mem::size_of::<clap_event_transport>() as u32,
                time: 0,
                space_id: CLAP_CORE_EVENT_SPACE_ID,
                type_: CLAP_EVENT_TRANSPORT,
                flags: 0,
            },
            flags: CLAP_TRANSPORT_IS_PLAYING,
            song_pos_beats: 0,
            song_pos_seconds: 0,
            tempo: 0.0,
            tempo_inc: 0.0,
            loop_start_beats: 0,
            loop_end_beats: 0,
            loop_start_seconds: 0,
            loop_end_seconds: 0,
            bar_start: 0,
            bar_number: 0,
            tsig_num: 4,
            tsig_denom: 4,
        };
        let mut pos = 0;
        while pos < FRAMES {
            let n = BUF.min(FRAMES - pos);
            let mut chans = [l[pos..].as_mut_ptr(), r[pos..].as_mut_ptr()];
            let mut out = clap_audio_buffer {
                data32: chans.as_mut_ptr(),
                data64: null_mut(),
                channel_count: 2,
                latency: 0,
                constant_mask: 0,
            };
            let p = clap_process {
                steady_time: pos as i64,
                frames_count: n as u32,
                transport: &transport,
                audio_inputs: null(),
                audio_outputs: &mut out,
                audio_inputs_count: 0,
                audio_outputs_count: 1,
                in_events: &in_events,
                out_events: &out_events,
            };
            assert_ne!((pl.process.unwrap())(plugin, &p), CLAP_PROCESS_ERROR);
            pos += n;
        }
        (pl.stop_processing.unwrap())(plugin);
        (pl.deactivate.unwrap())(plugin);
        (pl.destroy.unwrap())(plugin);
        (entry.deinit.unwrap())();

        let got: Vec<f32> = l.into_iter().zip(r).flat_map(|(a, b)| [a, b]).collect();
        assert!(common::peak(&got) > 0.05, "the song plays");
        let want = host(&song, &mut activate(&song, 44_100), FRAMES, BUF, &[]);
        let first = want.iter().zip(&got).position(|(a, b)| a != b);
        assert_eq!(first, None, "first difference at sample {first:?}");
    }
}

/// task-226: a note at frame 0 of the call where the host plays first.
const NOTE_VELOCITY: u8 = 100;
/// The frames after the host starts.
const NOTE_FRAMES: usize = 8_192;
const NOTE_BUF: usize = 512;
const NOTE_BPM: f64 = 120.0;

/// One run of a wrapper test at host rate `rate`: `stopped` frames with the host stopped, in
/// calls of [`NOTE_BUF`] frames and the rest (0: an offline render, the first call after the
/// activation plays; more: a realtime start), then the host plays, in calls of [`NOTE_BUF`],
/// with the note `key` on `channel` (0-based) at frame 0 of its first call.
#[derive(Clone, Copy, Debug)]
struct FirstNote {
    rate: u32,
    stopped: usize,
    channel: u8,
    key: u8,
}

const fn first_note(rate: u32, stopped: usize, channel: u8, key: u8) -> FirstNote {
    FirstNote {
        rate,
        stopped,
        channel,
        key,
    }
}

/// The 808's bass drum, the 909's and 303 #1's C, in an offline render and in a realtime start
/// after a stopped stretch of whole blocks (8192 frames) and of a part of one (7777 frames: the
/// user's live case of round 3, the 909's bass drum at 48 kHz, whose attack was cut).
const FIRST_NOTES: [FirstNote; 8] = [
    first_note(44_100, 0, 2, 36),
    first_note(44_100, 0, 0, 36),
    first_note(44_100, 8_192, 2, 36),
    first_note(44_100, 8_192, 0, 36),
    first_note(44_100, 7_777, 3, 36),
    first_note(48_000, 0, 3, 36),
    first_note(48_000, 7_777, 3, 36),
    first_note(48_000, 7_777, 0, 36),
];

impl FirstNote {
    /// The host frames of the run.
    fn frames(&self) -> usize {
        self.stopped + NOTE_FRAMES
    }

    /// The frames of the call that starts at host frame `pos`.
    fn len_at(&self, pos: usize) -> usize {
        if pos < self.stopped {
            NOTE_BUF.min(self.stopped - pos)
        } else {
            NOTE_BUF.min(self.frames() - pos)
        }
    }

    /// Whether the host plays in the call that starts at host frame `pos`.
    fn playing(&self, pos: usize) -> bool {
        pos >= self.stopped
    }

    /// Whether the call that starts at host frame `pos` carries the note.
    fn note_at(&self, pos: usize) -> bool {
        pos == self.stopped
    }

    /// A new instance (note mode) activated at `rate`, a host at 120 BPM that reports no
    /// position, through the process side directly; and its latency.
    fn reference(&self) -> (Vec<f32>, usize) {
        use orb_player::midi::Message;
        use orb_plugin::core::{HostEvent, HostTransport};
        let p = orb_plugin::instance::OrbParams::new();
        let mut core = activate(&p, self.rate);
        let note = [HostEvent {
            frame: 0,
            message: Message::NoteOn {
                channel: self.channel,
                note: self.key,
                velocity: NOTE_VELOCITY,
            },
        }];
        let frames = self.frames();
        let (mut l, mut r) = (vec![0f32; frames], vec![0f32; frames]);
        let mut pos = 0;
        while pos < frames {
            let n = self.len_at(pos);
            let t = HostTransport {
                playing: self.playing(pos),
                tempo: Some(NOTE_BPM),
                pos_beats: None,
            };
            let ev: &[HostEvent] = if self.note_at(pos) { &note } else { &[] };
            core.process_with(
                &mut l[pos..pos + n],
                &mut r[pos..pos + n],
                &p.values(),
                &t,
                ev,
            );
            pos += n;
        }
        let latency = core.latency() as usize;
        let out = l.into_iter().zip(r).flat_map(|(a, b)| [a, b]).collect();
        (out, latency)
    }

    /// A wrapper's output compared with [`FirstNote::reference`]: the note sounds (it was lost
    /// in an offline render before task-226), nothing of it comes before the play start after
    /// the latency compensation (where the host drops it: the cut attack of round 3), and the
    /// samples are the process side's.
    fn check(&self, what: &str, l: Vec<f32>, r: Vec<f32>) {
        let got: Vec<f32> = l.into_iter().zip(r).flat_map(|(a, b)| [a, b]).collect();
        let (want, latency) = self.reference();
        let start = self.stopped + latency;
        let (early, heard) = (
            common::peak(&got[..2 * start]),
            common::peak(&got[2 * start..]),
        );
        assert!(heard > 0.1, "{what} {self:?}: the note at frame 0 sounds");
        // at 44.1 kHz nothing; resampled, the kernel's pre-ringing at most
        let most = if self.rate == 44_100 { 0.0 } else { 1e-3 };
        assert!(
            early <= most,
            "{what} {self:?}: {early} before the play start"
        );
        let first = want.iter().zip(&got).position(|(a, b)| a != b);
        assert_eq!(
            first, None,
            "{what} {self:?}: first difference at sample {first:?}"
        );
    }
}

/// The input events of a CLAP `process` call: `ctx` is a `Vec<clap_event_note>`.
unsafe extern "C" fn notes_size(l: *const clap_input_events) -> u32 {
    unsafe { (*((*l).ctx as *const Vec<clap_event_note>)).len() as u32 }
}
unsafe extern "C" fn notes_get(l: *const clap_input_events, i: u32) -> *const clap_event_header {
    let notes = unsafe { &*((*l).ctx as *const Vec<clap_event_note>) };
    &notes[i as usize].header
}

/// task-226 through the CLAP wrapper: an offline render as Reaper starts it (activate, then
/// `process` from steady time 0 with the transport playing and a note-on at time 0), and a
/// realtime start (stopped calls, then the transport plays with a note-on at time 0 of that
/// call), the cases of [`FIRST_NOTES`].
#[test]
fn a_note_at_frame_0_of_the_first_playing_clap_call_sounds() {
    let _entry = CLAP_ENTRY.lock().unwrap_or_else(|e| e.into_inner());
    let lib = load();
    let entry: libloading::Symbol<*const clap_plugin_entry> =
        unsafe { lib.get(b"clap_entry\0") }.unwrap();
    let entry = unsafe { &**entry };
    let path = CString::new(library().to_string_lossy().as_bytes()).unwrap();
    unsafe {
        assert!((entry.init.unwrap())(path.as_ptr()));
        let factory = (entry.get_factory.unwrap())(CLAP_PLUGIN_FACTORY_ID.as_ptr())
            as *const clap_plugin_factory;
        for case in FIRST_NOTES {
            let (l, r) = clap_first_note(factory, case);
            case.check("CLAP", l, r);
        }
        (entry.deinit.unwrap())();
    }
}

/// One [`FirstNote`] run in a new CLAP instance of `factory`.
unsafe fn clap_first_note(
    factory: *const clap_plugin_factory,
    case: FirstNote,
) -> (Vec<f32>, Vec<f32>) {
    let frames = case.frames();
    let (mut l, mut r) = (vec![0f32; frames], vec![0f32; frames]);
    unsafe {
        let f = &*factory;
        let d = &*(f.get_plugin_descriptor.unwrap())(factory, 0);
        let host_info = clap_host {
            clap_version: CLAP_VERSION,
            host_data: null_mut(),
            name: c"orb-plugin test".as_ptr(),
            vendor: c"OpenRB-338".as_ptr(),
            url: c"".as_ptr(),
            version: c"0".as_ptr(),
            get_extension: Some(host_extension),
            request_restart: Some(host_request),
            request_process: Some(host_request),
            request_callback: Some(host_request),
        };
        let plugin = (f.create_plugin.unwrap())(factory, &host_info, d.id);
        let pl = &*plugin;
        assert!((pl.init.unwrap())(plugin));
        assert!((pl.activate.unwrap())(
            plugin,
            case.rate as f64,
            1,
            NOTE_BUF as u32
        ));
        assert!((pl.start_processing.unwrap())(plugin));
        let note = clap_event_note {
            header: clap_event_header {
                size: std::mem::size_of::<clap_event_note>() as u32,
                time: 0,
                space_id: CLAP_CORE_EVENT_SPACE_ID,
                type_: CLAP_EVENT_NOTE_ON,
                flags: 0,
            },
            note_id: -1,
            port_index: 0,
            channel: case.channel as i16,
            key: case.key as i16,
            velocity: NOTE_VELOCITY as f64 / 127.0,
        };
        let out_events = clap_output_events {
            ctx: null_mut(),
            try_push: Some(drop_event),
        };
        let mut transport = clap_event_transport {
            header: clap_event_header {
                size: std::mem::size_of::<clap_event_transport>() as u32,
                time: 0,
                space_id: CLAP_CORE_EVENT_SPACE_ID,
                type_: CLAP_EVENT_TRANSPORT,
                flags: 0,
            },
            flags: CLAP_TRANSPORT_IS_PLAYING | CLAP_TRANSPORT_HAS_TEMPO,
            song_pos_beats: 0,
            song_pos_seconds: 0,
            tempo: NOTE_BPM,
            tempo_inc: 0.0,
            loop_start_beats: 0,
            loop_end_beats: 0,
            loop_start_seconds: 0,
            loop_end_seconds: 0,
            bar_start: 0,
            bar_number: 0,
            tsig_num: 4,
            tsig_denom: 4,
        };
        let mut pos = 0;
        while pos < frames {
            let n = case.len_at(pos);
            transport.flags = if case.playing(pos) {
                CLAP_TRANSPORT_IS_PLAYING | CLAP_TRANSPORT_HAS_TEMPO
            } else {
                CLAP_TRANSPORT_HAS_TEMPO
            };
            let mut notes: Vec<clap_event_note> = if case.note_at(pos) {
                vec![note]
            } else {
                Vec::new()
            };
            let in_events = clap_input_events {
                ctx: &mut notes as *mut Vec<clap_event_note> as *mut c_void,
                size: Some(notes_size),
                get: Some(notes_get),
            };
            let mut chans = [l[pos..].as_mut_ptr(), r[pos..].as_mut_ptr()];
            let mut out = clap_audio_buffer {
                data32: chans.as_mut_ptr(),
                data64: null_mut(),
                channel_count: 2,
                latency: 0,
                constant_mask: 0,
            };
            let p = clap_process {
                steady_time: pos as i64,
                frames_count: n as u32,
                transport: &transport,
                audio_inputs: null(),
                audio_outputs: &mut out,
                audio_inputs_count: 0,
                audio_outputs_count: 1,
                in_events: &in_events,
                out_events: &out_events,
            };
            assert_ne!((pl.process.unwrap())(plugin, &p), CLAP_PROCESS_ERROR);
            pos += n;
        }
        (pl.stop_processing.unwrap())(plugin);
        (pl.deactivate.unwrap())(plugin);
        (pl.destroy.unwrap())(plugin);
    }
    (l, r)
}

/// The event list of a VST3 `process` call.
struct Events(std::cell::RefCell<Vec<vst3::Steinberg::Vst::Event>>);

impl vst3::Class for Events {
    type Interfaces = (vst3::Steinberg::Vst::IEventList,);
}

impl vst3::Steinberg::Vst::IEventListTrait for Events {
    unsafe fn getEventCount(&self) -> i32 {
        self.0.borrow().len() as i32
    }

    unsafe fn getEvent(&self, index: i32, e: *mut vst3::Steinberg::Vst::Event) -> i32 {
        match self.0.borrow().get(index as usize) {
            Some(&ev) => {
                unsafe { *e = ev };
                vst3::Steinberg::kResultOk
            }
            None => vst3::Steinberg::kInvalidArgument,
        }
    }

    unsafe fn addEvent(&self, _e: *mut vst3::Steinberg::Vst::Event) -> i32 {
        vst3::Steinberg::kResultFalse
    }
}

/// task-226 through the VST3 wrapper: an offline render (`setupProcessing` in offline mode,
/// `setActive`, `setProcessing`, then `process` with the playing flag in the first block's
/// `processContext` and a note-on at sample offset 0), and a realtime start (realtime mode,
/// stopped blocks, then the playing flag with a note-on at offset 0 of that block), the cases of
/// [`FIRST_NOTES`].
#[test]
fn a_note_at_frame_0_of_the_first_playing_vst3_block_sounds() {
    use vst3::Steinberg::kResultOk;
    use vst3::Steinberg::{IPluginFactory, IPluginFactory2, IPluginFactory2_iid, PClassInfo2};

    let lib = load();
    let get: libloading::Symbol<unsafe extern "system" fn() -> *mut IPluginFactory> =
        unsafe { lib.get(b"GetPluginFactory\0") }.unwrap();
    unsafe {
        let f = get();
        let vt = &*(*f).vtbl;
        let mut f2: *mut c_void = null_mut();
        assert_eq!(
            (vt.base.queryInterface)(f as *mut _, &IPluginFactory2_iid, &mut f2),
            kResultOk
        );
        let f2 = f2 as *mut IPluginFactory2;
        let mut ci: PClassInfo2 = std::mem::zeroed();
        assert_eq!(((*(*f2).vtbl).getClassInfo2)(f2, 0, &mut ci), kResultOk);
        for case in FIRST_NOTES {
            let (l, r) = vst3_first_note(f, &ci, case);
            case.check("VST3", l, r);
        }
        ((*(*f2).vtbl).base.base.release)(f2 as *mut _);
        (vt.base.release)(f as *mut _);
    }
}

/// One [`FirstNote`] run in a new VST3 instance of class `ci` of factory `f`: offline mode
/// without stopped blocks, realtime mode with them.
unsafe fn vst3_first_note(
    f: *mut vst3::Steinberg::IPluginFactory,
    ci: &vst3::Steinberg::PClassInfo2,
    case: FirstNote,
) -> (Vec<f32>, Vec<f32>) {
    use vst3::Steinberg::Vst::{
        AudioBusBuffers, AudioBusBuffers__type0, Event, Event_::EventTypes_, Event__type0,
        IAudioProcessor, IAudioProcessorTrait, IComponent, IComponentTrait, IEventList,
        NoteOnEvent, ProcessContext, ProcessContext_::StatesAndFlags_, ProcessData, ProcessModes_,
        ProcessSetup, SymbolicSampleSizes_,
    };
    use vst3::Steinberg::{IPluginBaseTrait, kResultOk};
    use vst3::{ComPtr, ComWrapper};

    let frames = case.frames();
    let (mut l, mut r) = (vec![0f32; frames], vec![0f32; frames]);
    let mode = if case.stopped == 0 {
        ProcessModes_::kOffline as i32
    } else {
        ProcessModes_::kRealtime as i32
    };
    unsafe {
        let vt = &*(*f).vtbl;
        let mut obj: *mut c_void = null_mut();
        let iid = vst3::Steinberg::Vst::IComponent_iid;
        assert_eq!(
            (vt.createInstance)(f, ci.cid.as_ptr(), iid.as_ptr(), &mut obj),
            kResultOk
        );
        let component = ComPtr::<IComponent>::from_raw(obj as *mut IComponent).unwrap();
        assert_eq!(component.initialize(null_mut()), kResultOk);
        let processor = component.cast::<IAudioProcessor>().unwrap();
        let mut setup = ProcessSetup {
            processMode: mode,
            symbolicSampleSize: SymbolicSampleSizes_::kSample32 as i32,
            maxSamplesPerBlock: NOTE_BUF as i32,
            sampleRate: case.rate as f64,
        };
        assert_eq!(processor.setupProcessing(&mut setup), kResultOk);
        assert_eq!(component.setActive(1), kResultOk);
        assert_eq!(processor.setProcessing(1), kResultOk);

        let mut note: Event = std::mem::zeroed();
        note.r#type = EventTypes_::kNoteOnEvent as u16;
        note.__field0 = Event__type0 {
            noteOn: NoteOnEvent {
                channel: case.channel as i16,
                pitch: case.key as i16,
                tuning: 0.0,
                velocity: NOTE_VELOCITY as f32 / 127.0,
                length: 0,
                noteId: -1,
            },
        };
        let events = ComWrapper::new(Events(std::cell::RefCell::new(Vec::new())));
        let list = events.to_com_ptr::<IEventList>().unwrap();
        let mut context: ProcessContext = std::mem::zeroed();
        // the flags are a c_int on Windows and a c_uint elsewhere
        #[allow(clippy::unnecessary_cast)]
        let (playing, stopped) = (
            (StatesAndFlags_::kPlaying | StatesAndFlags_::kTempoValid) as u32,
            StatesAndFlags_::kTempoValid as u32,
        );
        context.sampleRate = case.rate as f64;
        context.tempo = NOTE_BPM;
        context.timeSigNumerator = 4;
        context.timeSigDenominator = 4;
        let mut pos = 0;
        while pos < frames {
            let n = case.len_at(pos);
            context.state = if case.playing(pos) { playing } else { stopped };
            *events.0.borrow_mut() = if case.note_at(pos) {
                vec![note]
            } else {
                Vec::new()
            };
            let mut chans = [l[pos..].as_mut_ptr(), r[pos..].as_mut_ptr()];
            let mut out = AudioBusBuffers {
                numChannels: 2,
                silenceFlags: 0,
                __field0: AudioBusBuffers__type0 {
                    channelBuffers32: chans.as_mut_ptr(),
                },
            };
            context.projectTimeSamples = pos as i64;
            context.continousTimeSamples = pos as i64;
            let mut data = ProcessData {
                processMode: mode,
                symbolicSampleSize: SymbolicSampleSizes_::kSample32 as i32,
                numSamples: n as i32,
                numInputs: 0,
                numOutputs: 1,
                inputs: null_mut(),
                outputs: &mut out,
                inputParameterChanges: null_mut(),
                outputParameterChanges: null_mut(),
                inputEvents: list.as_ptr(),
                outputEvents: null_mut(),
                processContext: &mut context,
            };
            assert_eq!(processor.process(&mut data), kResultOk);
            pos += n;
        }
        assert_eq!(processor.setProcessing(0), kResultOk);
        assert_eq!(component.setActive(0), kResultOk);
        assert_eq!(component.terminate(), kResultOk);
    }
    (l, r)
}

#[test]
fn the_vst3_factory_has_the_identity() {
    use vst3::Steinberg::{
        IPluginFactory, IPluginFactory2, IPluginFactory2_iid, PClassInfo2, PFactoryInfo, kResultOk,
    };
    let lib = load();
    let get: libloading::Symbol<unsafe extern "system" fn() -> *mut IPluginFactory> =
        unsafe { lib.get(b"GetPluginFactory\0") }.unwrap();
    unsafe {
        let f = get();
        assert!(!f.is_null());
        let vt = &*(*f).vtbl;
        let mut fi: PFactoryInfo = std::mem::zeroed();
        assert_eq!((vt.getFactoryInfo)(f, &mut fi), kResultOk);
        assert_eq!(chars(&fi.vendor), "OpenRB-338");
        assert_eq!(chars(&fi.url), "https://github.com/pingw33n/OpenRB-338");
        assert!((vt.countClasses)(f) >= 1);
        let mut f2: *mut c_void = null_mut();
        assert_eq!(
            (vt.base.queryInterface)(f as *mut _, &IPluginFactory2_iid, &mut f2),
            kResultOk
        );
        let f2 = f2 as *mut IPluginFactory2;
        let mut ci: PClassInfo2 = std::mem::zeroed();
        assert_eq!(((*(*f2).vtbl).getClassInfo2)(f2, 0, &mut ci), kResultOk);
        let cid: Vec<u8> = ci.cid.iter().map(|&c| c as u8).collect();
        assert_eq!(cid, <OrbPlugin as Vst3Plugin>::PLATFORM_VST3_CLASS_ID);
        // the permanent GUID, as written down once (task-197)
        let guid: Vec<u8> = "EFAB08AFB39D43998A803DC1F84ED3D6"
            .as_bytes()
            .chunks(2)
            .map(|h| u8::from_str_radix(std::str::from_utf8(h).unwrap(), 16).unwrap())
            .collect();
        assert_eq!(<OrbPlugin as Vst3Plugin>::VST3_CLASS_ID.to_vec(), guid);
        if !cfg!(windows) {
            assert_eq!(cid, guid);
        }
        assert_eq!(chars(&ci.name), "OpenRB-338");
        assert_eq!(chars(&ci.category), "Audio Module Class");
        assert_eq!(chars(&ci.subCategories), "Instrument|Synth");
        assert_eq!(chars(&ci.vendor), "OpenRB-338");
        ((*(*f2).vtbl).base.base.release)(f2 as *mut _);
        (vt.base.release)(f as *mut _);
    }
}
