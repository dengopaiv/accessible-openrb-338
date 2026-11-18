//! The CLAP editor in a minimal host (task-201; the crash of the user's Reaper check of
//! 2026-11-16, `gui.create` panicking inside nice-plug's extern "C" function): the plugin
//! library is loaded as a host loads it, an instance made, and the GUI created and destroyed
//! through the CLAP API on the main thread (this test has no harness: `main` is the process's
//! main thread, as a host's GUI thread is). Two copies of the library are loaded at once, as
//! Reaper loads the VST3 and the CLAP bundle: with raw-window-metal's fixed class name the
//! second copy's `gui.create` aborted the process (`ORB_TEST_CLAP_LIBRARY` points the test at a
//! bundle's binary, e.g. an old one). The VST3 view is resized as Ableton Live resizes it
//! (task-234). macOS only: the editor needs AppKit.

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
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    use clap_sys::audio_buffer::clap_audio_buffer;
    use clap_sys::entry::clap_plugin_entry;
    use clap_sys::events::{clap_event_header, clap_input_events, clap_output_events};
    use clap_sys::ext::gui::{
        CLAP_EXT_GUI, CLAP_WINDOW_API_COCOA, clap_host_gui, clap_plugin_gui, clap_window,
        clap_window_handle,
    };
    use clap_sys::factory::plugin_factory::{CLAP_PLUGIN_FACTORY_ID, clap_plugin_factory};
    use clap_sys::host::clap_host;
    use clap_sys::plugin::clap_plugin;
    use clap_sys::process::{CLAP_PROCESS_ERROR, clap_process};
    use clap_sys::version::CLAP_VERSION;

    /// The host refuses `gui.request_resize` (task-224: as Reaper may before the editor is
    /// parented).
    static REFUSE: AtomicBool = AtomicBool::new(false);
    /// The `gui.request_resize` calls.
    static REQUESTS: AtomicU32 = AtomicU32::new(0);

    unsafe extern "C" fn resize_hints_changed(_: *const clap_host) {}
    unsafe extern "C" fn request_resize(_: *const clap_host, _: u32, _: u32) -> bool {
        // a resize loop would recurse here until the stack overflows; stop it with a message
        let n = REQUESTS.fetch_add(1, Ordering::Relaxed) + 1;
        // the 101st call aborts
        if n > 100 {
            eprintln!("clap_gui: gui.request_resize called {n} times, a resize loop");
            std::process::abort();
        }
        !REFUSE.load(Ordering::Relaxed)
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
        the_host_sizes_the_editor(&path);
        a_live_corner_drag_follows_the_cursor(&path);
        println!("clap_gui: ok");
    }

    /// The cursor's positions of a corner drag (the proposed size, points, relative to the
    /// window's top left as AppKit's live resize proposes it), off the rule's diagonal: wider
    /// than the rule's shape while growing (to 1000 x 700, where the rule's height is 839),
    /// then back down to 400 x 450 (taller than the rule's 349).
    fn corner_drag() -> Vec<(i32, i32)> {
        let mut path = Vec::new();
        let (mut x, mut y) = (624, 532);
        while x < 1000 {
            x += 8;
            y = (y + 4).min(700);
            path.push((x, y));
        }
        while x > 400 {
            x -= 8;
            y = (y - 3).max(450);
            path.push((x, y));
        }
        path
    }

    /// Ableton Live's resizing of the VST3 view (task-234; Live 12 Lite on macOS loads only
    /// the VST3): Live hosts the view in a window of its own and the view is resizable
    /// (`canResize`). While the user drags the window's corner Live asks `checkSizeConstraint`
    /// with the cursor's size, sizes its window to the answer and calls `onSize` (the user's
    /// recording): the view answers with the size asked (the panel is letterboxed in it), so
    /// the sizes follow the cursor and an answer asked again comes back unchanged. The panel
    /// corner's sizes (`corner_size` along the same drag; the test computes them and makes the
    /// host's calls itself, with no `IPlugFrame`: the request path `resizeView` is not
    /// exercised, a test cannot drag the egui corner of the loaded library) pass the same round
    /// trip unchanged.
    fn a_live_corner_drag_follows_the_cursor(path: &std::path::Path) {
        use objc2::MainThreadOnly;
        use objc2::rc::Retained;
        use objc2_app_kit::{NSBackingStoreType, NSWindow, NSWindowStyleMask};
        use objc2_foundation::{NSPoint, NSRect, NSSize};
        use orb_ui::embed::corner_size;
        use vst3::ComPtr;
        use vst3::Steinberg::Vst::{IComponent, IEditController, IEditControllerTrait};
        use vst3::Steinberg::{
            IPlugFrame, IPlugFrameTrait, IPlugView, IPlugViewTrait, IPluginBaseTrait,
            IPluginFactory, IPluginFactory2, IPluginFactory2_iid, PClassInfo2, ViewRect,
            kPlatformTypeNSView, kResultOk,
        };

        unsafe extern "C" {
            static kCFRunLoopDefaultMode: *const c_void;
            fn CFRunLoopRunInMode(mode: *const c_void, seconds: f64, once: u8) -> i32;
        }
        /// The host's `IPlugFrame`: the sizes of the view's `resizeView` calls.
        struct Frame(std::sync::Arc<std::sync::Mutex<Vec<(i32, i32)>>>);
        impl vst3::Class for Frame {
            type Interfaces = (IPlugFrame,);
        }
        impl IPlugFrameTrait for Frame {
            unsafe fn resizeView(&self, _view: *mut IPlugView, size: *mut ViewRect) -> i32 {
                let s = unsafe { &*size };
                self.0
                    .lock()
                    .unwrap()
                    .push((s.right - s.left, s.bottom - s.top));
                kResultOk
            }
        }
        let frame_calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let mtm = objc2::MainThreadMarker::new().expect("the main thread");
        let lib = unsafe { libloading::Library::new(path) }.expect("the plugin library loads");
        let get: libloading::Symbol<unsafe extern "system" fn() -> *mut IPluginFactory> =
            unsafe { lib.get(b"GetPluginFactory\0") }.unwrap();
        // Live's plugin window and its content view, the view's parent
        let window = unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                NSWindow::alloc(mtm),
                NSRect::new(NSPoint::new(100.0, 100.0), NSSize::new(624.0, 532.0)),
                NSWindowStyleMask::Titled | NSWindowStyleMask::Resizable,
                NSBackingStoreType::Buffered,
                false,
            )
        };
        unsafe { window.setReleasedWhenClosed(false) };
        let content = window.contentView().expect("the window's view");
        let rect = |w: i32, h: i32| ViewRect {
            left: 0,
            top: 0,
            right: w,
            bottom: h,
        };
        let mut sizes = Vec::new();
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
            let mut obj: *mut c_void = null_mut();
            let iid = vst3::Steinberg::Vst::IComponent_iid;
            assert_eq!(
                (vt.createInstance)(f, ci.cid.as_ptr(), iid.as_ptr(), &mut obj),
                kResultOk
            );
            let component = ComPtr::<IComponent>::from_raw(obj as *mut IComponent).unwrap();
            assert_eq!(component.initialize(null_mut()), kResultOk);
            let controller = component
                .cast::<IEditController>()
                .expect("the component is its controller");
            let view = ComPtr::<IPlugView>::from_raw(controller.createView(c"editor".as_ptr()))
                .expect("the plugin has a view");
            assert_eq!(view.isPlatformTypeSupported(kPlatformTypeNSView), kResultOk);
            // the host may resize the view to any size (task-234)
            assert_eq!(view.canResize(), kResultOk, "canResize");
            let mut r = rect(0, 0);
            assert_eq!(view.getSize(&mut r), kResultOk);
            assert_eq!((r.right, r.bottom), (656, 532), "the view's default size");
            // the host's frame: records the view's resizeView calls
            let frame = vst3::ComWrapper::new(Frame(frame_calls.clone()));
            let frame_ptr = frame.to_com_ptr::<IPlugFrame>().unwrap();
            assert_eq!(view.setFrame(frame_ptr.as_ptr()), kResultOk);
            let parent = Retained::as_ptr(&content) as *mut c_void;
            assert_eq!(view.attached(parent, kPlatformTypeNSView), kResultOk);
            window.orderFront(None);
            assert_eq!(view.canResize(), kResultOk, "canResize after attached");
            // The sizes the panel's corner asks for on the same drag (`corner_size`, sizes of
            // the rule), given by the test to the host's calls a host makes after a request:
            // the check, its window, the view.
            let mut grip = Vec::new();
            for (x, y) in corner_drag() {
                let s = corner_size(egui::Vec2::new(x as f32, y as f32));
                let (w, h) = (s.x as i32, s.y as i32);
                let mut r = rect(w, h);
                assert_eq!(view.checkSizeConstraint(&mut r), kResultOk);
                assert_eq!((r.right, r.bottom), (w, h), "the host's check of {w}x{h}");
                window.setContentSize(NSSize::new(w as f64, h as f64));
                assert_eq!(view.onSize(&mut r), kResultOk);
                let mut g = rect(0, 0);
                assert_eq!(view.getSize(&mut g), kResultOk);
                assert_eq!(
                    (g.right, g.bottom),
                    (w, h),
                    "getSize after onSize({w}, {h})"
                );
                grip.push(((x, y), (w, h)));
            }
            sizes.push(grip);
            // Live's drag of its window's corner: the cursor's size to checkSizeConstraint,
            // the answer to its window and onSize. The answer is the cursor's size, and the
            // view asks nothing of the host while the host drags (task-234: the user saw a
            // VST3 host-frame drag stick at one size): no resizeView, with the main run loop
            // run after each event (where the corner's requests would be flushed) and a
            // backing change sent to the editor's view (baseview's other request path).
            let mut drag = Vec::new();
            for (x, y) in corner_drag() {
                let mut r = rect(x, y);
                assert_eq!(view.checkSizeConstraint(&mut r), kResultOk);
                let (w, h) = (r.right, r.bottom);
                assert_eq!((w, h), (x, y), "checkSizeConstraint gives the host's size");
                // Live's window takes the answer; the view is told
                window.setContentSize(NSSize::new(w as f64, h as f64));
                assert_eq!(view.onSize(&mut r), kResultOk);
                let mut g = rect(0, 0);
                assert_eq!(view.getSize(&mut g), kResultOk);
                assert_eq!(
                    (g.right, g.bottom),
                    (w, h),
                    "getSize after onSize({w}, {h})"
                );
                // asked again, the view gives it back
                let mut again = rect(w, h);
                assert_eq!(view.checkSizeConstraint(&mut again), kResultOk);
                assert_eq!(
                    (again.right, again.bottom),
                    (w, h),
                    "checkSizeConstraint of its own answer"
                );
                for v in content.subviews().iter() {
                    let _: () = objc2::msg_send![&*v, viewDidChangeBackingProperties];
                }
                // SAFETY: the main thread's run loop, for a moment.
                CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.01, 1);
                drag.push(((x, y), (w, h)));
            }
            sizes.push(drag);
            assert_eq!(
                *frame_calls.lock().unwrap(),
                Vec::<(i32, i32)>::new(),
                "resizeView during the host's drag"
            );
            assert_eq!(view.removed(), kResultOk);
            drop(view);
            drop(controller);
            assert_eq!(component.terminate(), kResultOk);
            drop(component);
            ((*(*f2).vtbl).base.base.release)(f2 as *mut _);
            (vt.base.release)(f as *mut _);
        }
        window.orderOut(None);
        window.close();
        // the widths follow the cursor: up while it grows, down while it shrinks, never back
        for sizes in &sizes {
            let turn = sizes.iter().position(|s| s.0.0 == 1000).unwrap();
            for (i, pair) in sizes.windows(2).enumerate() {
                let ((c0, s0), (c1, s1)) = (pair[0], pair[1]);
                let ok = if i < turn { s1.0 >= s0.0 } else { s1.0 <= s0.0 };
                assert!(
                    ok,
                    "the width went back: cursor {c0:?} -> {s0:?}, cursor {c1:?} -> {s1:?}\n\
                     all: {sizes:?}"
                );
            }
        }
        drop(lib);
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

    /// The host sizes the editor before and after `gui.set_parent`, refusing the plugin's
    /// `gui.request_resize` (task-224, the Reaper crash switching the FX window from the
    /// parameter list to the plugin UI). On macOS `gui.create` gives baseview no parent, so it
    /// makes a window of its own and observes its resizes until `set_parent`; the host's
    /// `set_size` there was echoed back to the host as a request, refused, reverted, and so on
    /// until the stack overflowed. Then in a host window: sizes, a scale, the view's backing
    /// properties notification (with no change: a test cannot change the display's scale), the
    /// host's window resized, hidden, shown, and the editor closed and opened again.
    fn the_host_sizes_the_editor(path: &std::path::Path) {
        use objc2::rc::Retained;
        use objc2::{MainThreadMarker, MainThreadOnly};
        use objc2_app_kit::{NSBackingStoreType, NSWindow, NSWindowStyleMask};
        use objc2_foundation::{NSPoint, NSRect, NSSize};

        let mtm = MainThreadMarker::new().expect("the main thread");
        let lib = unsafe { libloading::Library::new(path) }.expect("the plugin library loads");
        let entry: libloading::Symbol<*const clap_plugin_entry> =
            unsafe { lib.get(b"clap_entry\0") }.unwrap();
        let entry = unsafe { &**entry };
        let cpath = CString::new(path.to_string_lossy().as_bytes()).unwrap();
        // the host's window and the view it gives the editor
        let window = unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                NSWindow::alloc(mtm),
                NSRect::new(NSPoint::new(100.0, 100.0), NSSize::new(624.0, 532.0)),
                NSWindowStyleMask::Titled | NSWindowStyleMask::Resizable,
                NSBackingStoreType::Buffered,
                false,
            )
        };
        unsafe { window.setReleasedWhenClosed(false) };
        let content = window.contentView().expect("the window's view");
        let parent = clap_window {
            api: CLAP_WINDOW_API_COCOA.as_ptr(),
            specific: clap_window_handle {
                cocoa: Retained::as_ptr(&content) as *mut c_void,
            },
        };
        REFUSE.store(true, Ordering::Relaxed);
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
            let plugin = (f.create_plugin.unwrap())(factory, &host, d.id);
            let pl = &*plugin;
            assert!((pl.init.unwrap())(plugin));
            let g = &*((pl.get_extension.unwrap())(plugin, CLAP_EXT_GUI.as_ptr())
                as *const clap_plugin_gui);
            let api = CLAP_WINDOW_API_COCOA.as_ptr();
            // the host's size, adjusted, set; `false` from set_size is a failure too
            let size = |w: u32, h: u32| {
                let (mut aw, mut ah) = (w, h);
                assert!((g.adjust_size.unwrap())(plugin, &mut aw, &mut ah));
                assert!(
                    (g.set_size.unwrap())(plugin, aw, ah),
                    "gui.set_size({aw}, {ah})"
                );
                let (mut gw, mut gh) = (0u32, 0u32);
                assert!((g.get_size.unwrap())(plugin, &mut gw, &mut gh));
                assert_eq!((gw, gh), (aw, ah), "the editor's size after set_size");
            };
            for round in 0..2 {
                REQUESTS.store(0, Ordering::Relaxed);
                assert!((g.create.unwrap())(plugin, api, false), "round {round}");
                // the FX window's size before the editor is parented (the parameter list's)
                size(900, 700);
                size(500, 300);
                assert!((g.set_scale.unwrap())(plugin, 2.0));
                size(700, 600);
                assert!((g.set_parent.unwrap())(plugin, &parent));
                assert!((g.show.unwrap())(plugin));
                window.orderFront(None);
                size(800, 650);
                // Bitwig's drag of its window's corner (task-234, the user's check of
                // 2026-11-17: the CLAP's scale jumped as Live's VST3 did): gui.adjust_size with
                // the cursor's size on each mouse event, then gui.set_size with the answer. The
                // answers are the cursor's sizes and, asked again, come back unchanged. (REAPER
                // asks the same way but keeps its window at the size it asked for: the answer
                // being that size, the editor is the window's size and the panel is
                // letterboxed in it, uncropped.)
                if round == 0 {
                    let mut sizes = Vec::new();
                    for (x, y) in corner_drag() {
                        let (mut aw, mut ah) = (x as u32, y as u32);
                        assert!((g.adjust_size.unwrap())(plugin, &mut aw, &mut ah));
                        assert_eq!((aw, ah), (x as u32, y as u32), "the host's size");
                        assert!((g.set_size.unwrap())(plugin, aw, ah));
                        let (mut gw, mut gh) = (0u32, 0u32);
                        assert!((g.get_size.unwrap())(plugin, &mut gw, &mut gh));
                        assert_eq!((gw, gh), (aw, ah), "get_size after set_size");
                        let (mut bw, mut bh) = (aw, ah);
                        assert!((g.adjust_size.unwrap())(plugin, &mut bw, &mut bh));
                        assert_eq!((bw, bh), (aw, ah), "gui.adjust_size of its own answer");
                        sizes.push(((x, y), aw));
                    }
                    let turn = sizes.iter().position(|s| s.0.0 == 1000).unwrap();
                    for (i, p) in sizes.windows(2).enumerate() {
                        let ok = if i < turn {
                            p[1].1 >= p[0].1
                        } else {
                            p[1].1 <= p[0].1
                        };
                        assert!(ok, "CLAP: the width went back at {i}: {sizes:?}");
                    }
                    size(800, 650);
                }
                // the editor's view (the host view's only subview) gets a backing change
                let views = content.subviews();
                assert_eq!(views.len(), 1, "the editor's view in the host's");
                let view = views.objectAtIndex(0);
                let () = objc2::msg_send![&*view, viewDidChangeBackingProperties];
                // The host's window resized (float/dock), then the host's size for the editor.
                // The window's resize alone reaches no baseview code (the editor's view is not
                // the window's delegate and does not autoresize), so it asks nothing by itself;
                // it checks only that the parented editor survives it and the set_size after.
                window.setContentSize(NSSize::new(1000.0, 800.0));
                size(1000, 800);
                assert!((g.hide.unwrap())(plugin));
                assert!((g.show.unwrap())(plugin));
                (g.destroy.unwrap())(plugin);
                window.orderOut(None);
                assert_eq!(
                    REQUESTS.load(Ordering::Relaxed),
                    0,
                    "the host is not asked for the sizes it set, round {round}"
                );
            }
            (pl.destroy.unwrap())(plugin);
            (entry.deinit.unwrap())();
        }
        REFUSE.store(false, Ordering::Relaxed);
        window.close();
        drop(lib);
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
                // the panel at 1:1, its margins (2 x 16 across) and the menu bar
                assert_eq!((w, h), (656, 532), "the editor's default size");
                // the host may resize the editor to any size (task-234): gui.adjust_size gives
                // the host's size back, at least the minimum (the panel is letterboxed)
                assert!((g.can_resize.unwrap())(plugin), "gui.can_resize");
                let mut hints: clap_sys::ext::gui::clap_gui_resize_hints = std::mem::zeroed();
                assert!((g.get_resize_hints.unwrap())(plugin, &mut hints));
                assert!(hints.can_resize_horizontally && hints.can_resize_vertically);
                assert!(!hints.preserve_aspect_ratio);
                for ((w, h), want) in [
                    ((1000u32, 540u32), (1000u32, 540u32)),
                    ((721, 532), (721, 532)),
                    ((630, 1042), (630, 1042)),
                    ((10, 10), (312, 277)),
                ] {
                    let (mut aw, mut ah) = (w, h);
                    assert!((g.adjust_size.unwrap())(plugin, &mut aw, &mut ah));
                    assert_eq!((aw, ah), want, "gui.adjust_size({w}, {h})");
                }
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
