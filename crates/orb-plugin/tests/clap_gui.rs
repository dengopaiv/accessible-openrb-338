//! The CLAP editor in a minimal host (task-201; the crash of the user's Reaper check of
//! 2026-11-16, `gui.create` panicking inside nice-plug's extern "C" function): the plugin
//! library is loaded as a host loads it, an instance made, and the GUI created and destroyed
//! through the CLAP API on the main thread (this test has no harness: `main` is the process's
//! main thread, as a host's GUI thread is). Two copies of the library are loaded at once, as
//! Reaper loads the VST3 and the CLAP bundle: with raw-window-metal's fixed class name the
//! second copy's `gui.create` aborted the process (`ORB_TEST_CLAP_LIBRARY` points the test at a
//! bundle's binary, e.g. an old one). macOS only: the editor needs AppKit.

#[cfg(target_os = "macos")]
mod common;

#[cfg(not(target_os = "macos"))]
fn main() {
    println!("clap_gui: skipped, the CLAP editor test needs macOS (AppKit)");
}

#[cfg(target_os = "macos")]
fn main() {
    mac::run();
}

#[cfg(target_os = "macos")]
mod mac {
    use std::ffi::{CString, c_char, c_void};
    use std::path::PathBuf;
    use std::ptr::{null, null_mut};

    use clap_sys::audio_buffer::clap_audio_buffer;
    use clap_sys::entry::clap_plugin_entry;
    use clap_sys::events::{clap_event_header, clap_input_events, clap_output_events};
    use clap_sys::ext::gui::{CLAP_EXT_GUI, CLAP_WINDOW_API_COCOA, clap_host_gui, clap_plugin_gui};
    use clap_sys::factory::plugin_factory::{CLAP_PLUGIN_FACTORY_ID, clap_plugin_factory};
    use clap_sys::host::clap_host;
    use clap_sys::plugin::clap_plugin;
    use clap_sys::process::{CLAP_PROCESS_ERROR, clap_process};
    use clap_sys::version::CLAP_VERSION;

    unsafe extern "C" fn resize_hints_changed(_: *const clap_host) {}
    unsafe extern "C" fn request_resize(_: *const clap_host, _: u32, _: u32) -> bool {
        true
    }
    unsafe extern "C" fn request_show(_: *const clap_host) -> bool {
        true
    }
    unsafe extern "C" fn closed(_: *const clap_host, _: bool) {}

    static HOST_GUI: clap_host_gui = clap_host_gui {
        resize_hints_changed: Some(resize_hints_changed),
        request_resize: Some(request_resize),
        request_show: Some(request_show),
        request_hide: Some(request_show),
        closed: Some(closed),
    };

    unsafe extern "C" fn host_extension(_: *const clap_host, id: *const c_char) -> *const c_void {
        if unsafe { std::ffi::CStr::from_ptr(id) } == CLAP_EXT_GUI {
            &HOST_GUI as *const clap_host_gui as *const c_void
        } else {
            null()
        }
    }
    unsafe extern "C" fn host_request(_: *const clap_host) {}

    unsafe extern "C" fn no_events(_: *const clap_input_events) -> u32 {
        0
    }
    unsafe extern "C" fn no_event(_: *const clap_input_events, _: u32) -> *const clap_event_header {
        null()
    }
    unsafe extern "C" fn drop_event(
        _: *const clap_output_events,
        _: *const clap_event_header,
    ) -> bool {
        true
    }

    /// `blocks` process calls of 512 frames, without events.
    unsafe fn process(pl: &clap_plugin, plugin: *const clap_plugin, blocks: usize) {
        const BUF: usize = 512;
        let (mut l, mut r) = (vec![0f32; BUF], vec![0f32; BUF]);
        let in_events = clap_input_events {
            ctx: null_mut(),
            size: Some(no_events),
            get: Some(no_event),
        };
        let out_events = clap_output_events {
            ctx: null_mut(),
            try_push: Some(drop_event),
        };
        for b in 0..blocks {
            let mut chans = [l.as_mut_ptr(), r.as_mut_ptr()];
            let mut out = clap_audio_buffer {
                data32: chans.as_mut_ptr(),
                data64: null_mut(),
                channel_count: 2,
                latency: 0,
                constant_mask: 0,
            };
            let p = clap_process {
                steady_time: (b * BUF) as i64,
                frames_count: BUF as u32,
                transport: null(),
                audio_inputs: null(),
                audio_outputs: &mut out,
                audio_inputs_count: 0,
                audio_outputs_count: 1,
                in_events: &in_events,
                out_events: &out_events,
            };
            assert_ne!(
                unsafe { (pl.process.unwrap())(plugin, &p) },
                CLAP_PROCESS_ERROR
            );
        }
    }

    /// The plugin library (`cargo build -p orb-plugin --lib`, as tests/exports.rs builds it).
    fn library() -> PathBuf {
        // a bundle's binary instead (debugging a host's crash)
        if let Some(p) = std::env::var_os("ORB_TEST_CLAP_LIBRARY") {
            return p.into();
        }
        let root = super::common::root();
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
            .join("liborb_plugin.dylib")
    }

    pub fn run() {
        let path = library();
        // Two copies of the library in one process, as a host that has loaded the VST3 and
        // the CLAP bundle (the same binary in two files): each makes an editor.
        let dir = tempfile::tempdir().unwrap();
        let copy = dir.path().join("copy.dylib");
        std::fs::copy(&path, &copy).unwrap();
        let first = open_editor(&path);
        let second = open_editor(&copy);
        drop((first, second));
        the_corner_request_is_posted_to_the_main_queue();
        println!("clap_gui: ok");
    }

    /// Host callbacks that count the requests.
    struct Count(std::rc::Rc<std::cell::Cell<u32>>);

    impl nice_plug::editor::HostCallbacks for Count {
        fn request_resize(
            &mut self,
            _: nice_plug::editor::dpi::Size,
            _: f64,
        ) -> Result<(), Box<dyn std::error::Error>> {
            self.0.set(self.0.get() + 1);
            Ok(())
        }

        fn destroyed(&mut self) {}
    }

    /// The resize corner's request (`HostRoute`) reaches the host only after the current
    /// event, through libdispatch's main queue: not during `request`, and once when the main
    /// run loop runs.
    fn the_corner_request_is_posted_to_the_main_queue() {
        use orb_plugin::editor::HostRoute;
        unsafe extern "C" {
            static kCFRunLoopDefaultMode: *const c_void;
            fn CFRunLoopRunInMode(mode: *const c_void, seconds: f64, once: u8) -> i32;
        }
        let count = std::rc::Rc::new(std::cell::Cell::new(0));
        let (route, proxy) = HostRoute::register(
            Box::new(Count(count.clone())),
            std::sync::Arc::new(std::sync::Mutex::new(1.0)),
        );
        assert_eq!(
            route.request(700.0, 594.0),
            Some(true),
            "posted on the main thread"
        );
        assert_eq!(
            route.request(710.0, 602.0),
            Some(true),
            "the newer size, same post"
        );
        assert_eq!(count.get(), 0, "nothing asked inside the call");
        for _ in 0..20 {
            if count.get() > 0 {
                break;
            }
            // SAFETY: the main thread's run loop, for a moment.
            unsafe { CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.05, 1) };
        }
        assert_eq!(count.get(), 1, "the host was asked once, after the event");
        drop(proxy);
    }

    /// Loads the library at `path`, makes two instances in turn and opens and closes their
    /// editors; returns the library (kept loaded, as a host keeps it).
    fn open_editor(path: &std::path::Path) -> libloading::Library {
        let path = path.to_path_buf();
        let lib = unsafe { libloading::Library::new(&path) }.expect("the plugin library loads");
        let entry: libloading::Symbol<*const clap_plugin_entry> =
            unsafe { lib.get(b"clap_entry\0") }.unwrap();
        let entry = unsafe { &**entry };
        let cpath = CString::new(path.to_string_lossy().as_bytes()).unwrap();
        unsafe {
            assert!((entry.init.unwrap())(cpath.as_ptr()));
            let factory = (entry.get_factory.unwrap())(CLAP_PLUGIN_FACTORY_ID.as_ptr())
                as *const clap_plugin_factory;
            let f = &*factory;
            let d = &*(f.get_plugin_descriptor.unwrap())(factory, 0);
            let host = clap_host {
                clap_version: CLAP_VERSION,
                host_data: null_mut(),
                name: c"orb-plugin gui test".as_ptr(),
                vendor: c"OpenRB-338".as_ptr(),
                url: c"".as_ptr(),
                version: c"0".as_ptr(),
                get_extension: Some(host_extension),
                request_restart: Some(host_request),
                request_process: Some(host_request),
                request_callback: Some(host_request),
            };
            for round in 0..2 {
                let plugin = (f.create_plugin.unwrap())(factory, &host, d.id);
                assert!(!plugin.is_null());
                let pl = &*plugin;
                assert!((pl.init.unwrap())(plugin));
                assert!((pl.activate.unwrap())(plugin, 48_000.0, 1, 512));
                let gui = (pl.get_extension.unwrap())(plugin, CLAP_EXT_GUI.as_ptr())
                    as *const clap_plugin_gui;
                assert!(!gui.is_null(), "the plugin has a GUI");
                let g = &*gui;
                let api = CLAP_WINDOW_API_COCOA.as_ptr();
                assert!((g.is_api_supported.unwrap())(plugin, api, false));
                // what Reaper does: create (embedded), then the size, then set_parent
                assert!(
                    (g.create.unwrap())(plugin, api, false),
                    "gui.create, round {round}"
                );
                let (mut w, mut h) = (0u32, 0u32);
                assert!((g.get_size.unwrap())(plugin, &mut w, &mut h));
                assert_eq!((w, h), (624, 532), "the editor's default size");
                assert!((g.can_resize.unwrap())(plugin));
                let (mut aw, mut ah) = (1000u32, 600u32);
                assert!((g.adjust_size.unwrap())(plugin, &mut aw, &mut ah));
                // the instance plays while its editor is open (the Reaper crash report also
                // had the audio thread in the engine)
                assert!((pl.start_processing.unwrap())(plugin));
                process(pl, plugin, 200);
                (pl.stop_processing.unwrap())(plugin);
                (g.destroy.unwrap())(plugin);
                (pl.deactivate.unwrap())(plugin);
                (pl.destroy.unwrap())(plugin);
            }
            (entry.deinit.unwrap())();
        }
        lib
    }
}
