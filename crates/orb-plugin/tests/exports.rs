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
    CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_TRANSPORT, CLAP_TRANSPORT_IS_PLAYING, clap_event_header,
    clap_event_transport, clap_input_events, clap_output_events,
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

#[test]
fn the_clap_entry_has_the_identity_and_plays_a_restored_song() {
    const FRAMES: usize = 30_000;
    const BUF: usize = 512;
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
