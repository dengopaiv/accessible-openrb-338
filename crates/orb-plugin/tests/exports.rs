//! The plugin library exports a VST3 entry with the plugin's identity (task-197), plays a note
//! at frame 0 of the first playing block (task-226) and has continuous 0..127 parameters
//! (task-275), loaded through the VST3 C API.
//!
//! The test builds the library (`cargo build -p orb-plugin --lib`, the cdylib the bundles hold;
//! `cargo xtask bundle orb-plugin` lays out the bundles from the same file) and loads it with the
//! dynamic loader. It is a minimal host of its own; loading in Reaper and the validators is
//! task-204's.

mod common;

use std::ffi::{c_char, c_void};
use std::path::PathBuf;
use std::ptr::null_mut;
use std::sync::OnceLock;

use common::activate;
use nice_plug::prelude::Vst3Plugin;
use orb_plugin::{OrbPlugin, params};

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

fn chars(b: &[c_char]) -> String {
    let b: Vec<u8> = b
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();
    String::from_utf8_lossy(&b).into_owned()
}

/// The per-device output buses a host gives the plugin besides the main one (task-256): each
/// stereo, `frames` long. A host passes every bus the plugin declares (nice-plug reads the
/// buffers of all of them).
struct Aux {
    chans: Vec<Vec<f32>>,
}

impl Aux {
    fn new(frames: usize) -> Aux {
        Aux {
            chans: vec![vec![0f32; frames]; 2 * orb_plugin::core::DEVICE_OUTPUTS],
        }
    }

    /// The channel pointers of the buses from frame `pos`.
    fn ptrs(&mut self, pos: usize) -> [[*mut f32; 2]; orb_plugin::core::DEVICE_OUTPUTS] {
        let mut it = self.chans.iter_mut();
        std::array::from_fn(|_| {
            let (l, r) = (it.next().unwrap(), it.next().unwrap());
            [l[pos..].as_mut_ptr(), r[pos..].as_mut_ptr()]
        })
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
        AudioBusBuffers, AudioBusBuffers__type0, BusDirections_, BusInfo, BusTypes_, Event,
        Event_::EventTypes_, Event__type0, IAudioProcessor, IAudioProcessorTrait, IComponent,
        IComponentTrait, IEventList, MediaTypes_, NoteOnEvent, ProcessContext,
        ProcessContext_::StatesAndFlags_, ProcessData, ProcessModes_, ProcessSetup,
        SymbolicSampleSizes_,
    };
    use vst3::Steinberg::{IPluginBaseTrait, kResultOk};
    use vst3::{ComPtr, ComWrapper};

    let frames = case.frames();
    let (mut l, mut r) = (vec![0f32; frames], vec![0f32; frames]);
    let mut aux = Aux::new(frames);
    let mode: i32 = if case.stopped == 0 {
        ProcessModes_::kOffline as _
    } else {
        ProcessModes_::kRealtime as _
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
        // task-256: the main Mix bus and the four per-device aux buses, stereo
        // `as _`: the vst3 enum constants are i32 on Windows and u32 elsewhere, and clippy's
        // unnecessary_cast rejects `as i32` on Windows
        let audio: vst3::Steinberg::Vst::MediaType = MediaTypes_::kAudio as _;
        let output: vst3::Steinberg::Vst::BusDirection = BusDirections_::kOutput as _;
        assert_eq!(component.getBusCount(audio, output), 5);
        let mut names = Vec::new();
        for i in 0..5 {
            let mut info: BusInfo = std::mem::zeroed();
            assert_eq!(component.getBusInfo(audio, output, i, &mut info), kResultOk);
            assert_eq!(info.channelCount, 2);
            let kmain: vst3::Steinberg::Vst::BusType = BusTypes_::kMain as _;
            let main = info.busType == kmain;
            assert_eq!(main, i == 0);
            let n = info.name.iter().position(|&c| c == 0).unwrap();
            names.push(String::from_utf16(&info.name[..n]).unwrap());
        }
        assert_eq!(names, ["Mix", "303-1", "303-2", "808", "909"]);
        let processor = component.cast::<IAudioProcessor>().unwrap();
        let mut setup = ProcessSetup {
            processMode: mode,
            symbolicSampleSize: SymbolicSampleSizes_::kSample32 as _,
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
        let (playing, stopped): (u32, u32) = (
            (StatesAndFlags_::kPlaying | StatesAndFlags_::kTempoValid) as _,
            StatesAndFlags_::kTempoValid as _,
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
            let mut ap = aux.ptrs(pos);
            let [a0, a1, a2, a3] = &mut ap;
            let stereo = |chans: &mut [*mut f32; 2]| AudioBusBuffers {
                numChannels: 2,
                silenceFlags: 0,
                __field0: AudioBusBuffers__type0 {
                    channelBuffers32: chans.as_mut_ptr(),
                },
            };
            let mut out = [
                stereo(&mut chans),
                stereo(a0),
                stereo(a1),
                stereo(a2),
                stereo(a3),
            ];
            context.projectTimeSamples = pos as i64;
            context.continousTimeSamples = pos as i64;
            let mut data = ProcessData {
                processMode: mode,
                symbolicSampleSize: SymbolicSampleSizes_::kSample32 as _,
                numSamples: n as i32,
                numInputs: 0,
                numOutputs: out.len() as i32,
                inputs: null_mut(),
                outputs: out.as_mut_ptr(),
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

/// task-275: through the VST3 export, the 0..127 values have no step count and every other
/// parameter its range's; the normalized value of every integer is `(v - min) / (max - min)`,
/// as the released build's.
#[test]
fn the_vst3_parameters_are_continuous() {
    use vst3::ComPtr;
    use vst3::Steinberg::Vst::{IComponent, IEditController, IEditControllerTrait, ParameterInfo};
    use vst3::Steinberg::{IPluginBaseTrait, IPluginFactory, kResultOk};
    let lib = load();
    let get: libloading::Symbol<unsafe extern "system" fn() -> *mut IPluginFactory> =
        unsafe { lib.get(b"GetPluginFactory\0") }.unwrap();
    unsafe {
        let f = get();
        let vt = &*(*f).vtbl;
        let mut ci: vst3::Steinberg::PClassInfo = std::mem::zeroed();
        assert_eq!((vt.getClassInfo)(f, 0, &mut ci), kResultOk);
        let mut obj: *mut c_void = null_mut();
        let iid = vst3::Steinberg::Vst::IComponent_iid;
        assert_eq!(
            (vt.createInstance)(f, ci.cid.as_ptr(), iid.as_ptr(), &mut obj),
            kResultOk
        );
        let component = ComPtr::<IComponent>::from_raw(obj as *mut IComponent).unwrap();
        assert_eq!(component.initialize(null_mut()), kResultOk);
        let ctl = component.cast::<IEditController>().unwrap();
        let mut continuous = 0;
        let mut seen = 0;
        for i in 0..ctl.getParameterCount() {
            let mut info: ParameterInfo = std::mem::zeroed();
            assert_eq!(ctl.getParameterInfo(i, &mut info), kResultOk);
            let n = info.title.iter().position(|&c| c == 0).unwrap();
            let title = String::from_utf16(&info.title[..n]).unwrap();
            // nice-plug's hidden MIDI CC parameters
            let Some(s) = params::SPECS
                .iter()
                .find(|s| params::name(s.target) == title)
            else {
                continue;
            };
            seen += 1;
            let want = if s.continuous() {
                0
            } else {
                (s.max - s.min) as i32
            };
            assert_eq!(info.stepCount, want, "{title}");
            continuous += s.continuous() as usize;
            for v in s.min..=s.max {
                let got = ctl.plainParamToNormalized(info.id, v as f64);
                let old = (v - s.min) as f32 / (s.max - s.min) as f32;
                assert_eq!(got, old as f64, "{title} {v}");
            }
        }
        assert_eq!((seen, continuous), (params::COUNT, 86));
        assert_eq!(component.terminate(), kResultOk);
        drop(ctl);
        drop(component);
        (vt.base.release)(f as *mut _);
    }
}
