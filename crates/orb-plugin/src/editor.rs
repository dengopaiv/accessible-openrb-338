//! The editor (task-201, `docs/plugin.md` section 10): orb-ui's panel in the host's plugin
//! window, through nice-plug's egui adapter (egui-baseview on wgpu).
//!
//! - [`OrbEditor`] is nice-plug's `Editor`: its size is the state's (`editor_size`), the host may
//!   resize it ([`resize_hint`]) to sizes of the rule height = panel height at the width + the
//!   unscaled menu bar ([`OrbHandle`]'s `adjust_size`), and each open spawns a new egui window with
//!   a new [`orb_ui::app::App`].
//! - [`PluginPort`] is the panel's player ([`orb_ui::link::Port`]): a control's message sets its
//!   host parameter (so the host records and saves it; during the preview it goes to the preview
//!   set instead), a pattern edit goes to [`OrbParams::edit`], Play and Stop enter and leave the
//!   preview (task-217), the rest of the transport is dropped.
//! - [`EditorEmbed`] is the plugin's side of the panel ([`orb_ui::embed::Embed`]): the song, File >
//!   Open and New, the mod.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::error::Error;
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use nice_plug::context::gui::GuiContext;
use nice_plug::editor::dpi::{LogicalSize, NativeSize, Size};
use nice_plug::editor::{EditorHandle, HostCallbacks, HostMethods, SpawnedEditor};
use nice_plug::prelude::*;
use nice_plug_egui::baseview::{Error as BaseviewError, HandlerError, Window, WindowSize};
use nice_plug_egui::{
    EguiEditorHandle, EguiEditorState, EguiNiceSettings, Frame, KeyCapture, NiceEguiApp,
    RepaintNotifier, create_egui_editor,
};
use orb_engine::frame::{Engine, LoadRefused};
use orb_engine::samples::SampleSet;
use orb_engine::song::Song;
use orb_player::{Command, Snapshot, Tracks};
use orb_ui::app::App;
use orb_ui::embed::{Embed, MENU_H, PluginMode, SongView, window_height};
use orb_ui::layout::{PANEL_H, PANEL_W};
use orb_ui::link::{Link, Port};
use orb_ui::settings::Settings;

use crate::instance::{self, Kit, Mode, OrbParams, ROWS};
use crate::params::{self, SPECS, Target};

/// The editor's size before the state has one, in logical points: the panel at its native
/// size and the menu bar above it.
pub const DEFAULT_SIZE: (u32, u32) = (PANEL_W as u32, PANEL_H as u32 + MENU_H as u32);

/// The engine frames after which the panel takes the engine's values again after it set a
/// parameter: the host passes the change to `process` in its next buffer, so this is longer
/// than any buffer a host uses (8192 frames, 186 ms).
pub const FOLLOW_FRAMES: u64 = 8192;

/// How the host may resize the editor: both ways, no smaller than half the panel (the
/// standalone window's minimum) and its menu bar. No aspect ratio: the menu bar does not scale
/// with the panel, so the window's shape is a rule, which [`OrbHandle::adjust_size`] applies
/// (CLAP `gui.adjust_size`, VST3 `IPlugView::checkSizeConstraint`).
pub fn resize_hint() -> ResizeHint {
    ResizeHint::RESIZABLE.with_min_max_logical_size(
        Some(LogicalSize {
            width: PANEL_W / 2.0,
            height: PANEL_H / 2.0 + MENU_H,
        }),
        None,
    )
}

/// The window size (logical points) that keeps the rule height = [`window_height`] of the
/// width, closest to a size `new` the host proposes while the window is `current`: the side
/// the host changed more (relatively) decides; at least half the panel.
pub fn adjust_logical(new: (f64, f64), current: (f64, f64)) -> (f64, f64) {
    let (pw, ph, m) = (PANEL_W as f64, PANEL_H as f64, MENU_H as f64);
    let dw = (new.0 - current.0).abs() / current.0.max(1.0);
    let dh = (new.1 - current.1).abs() / current.1.max(1.0);
    let w = if dh > dw {
        (new.1 - m) * pw / ph
    } else {
        new.0
    };
    let w = w.max(pw / 2.0).round();
    (w, window_height(w as f32) as f64)
}

/// The editor window's handle: egui's, with the size rule of [`adjust_logical`] for the host's
/// resizing.
pub struct OrbHandle {
    inner: EguiEditorHandle,
}

impl EditorHandle for OrbHandle {
    type Window = Window;
    type Error = BaseviewError;

    fn run_until_closed(window: Self::Window) -> Result<(), Self::Error> {
        guard_err("run", || EguiEditorHandle::run_until_closed(window))
    }

    fn set_parent(
        &self,
        parent: ParentWindowHandle,
        window: &Self::Window,
    ) -> Result<(), Self::Error> {
        guard_err("set_parent", || self.inner.set_parent(parent, window))
    }

    fn show(&self, window: &Self::Window) -> Result<(), Self::Error> {
        guard_err("show", || self.inner.show(window))
    }

    fn hide(&self, window: &Self::Window) -> Result<(), Self::Error> {
        guard_err("hide", || self.inner.hide(window))
    }

    fn set_size(
        &self,
        new_size: NativeSize<u32>,
        window: &Self::Window,
    ) -> Result<(), Self::Error> {
        guard_err("set_size", || self.inner.set_size(new_size, window))
    }

    fn host_main_thread_callback(&self, window: &Self::Window) {
        let _ = guard("host_main_thread_callback", || {
            self.inner.host_main_thread_callback(window)
        });
    }

    /// The closest size of the rule (CLAP `gui.adjust_size`, VST3 `checkSizeConstraint`).
    fn adjust_size(
        &self,
        new_size: NativeSize<u32>,
        window: &Self::Window,
    ) -> Option<NativeSize<u32>> {
        // a panic accepts the host's size as it is (`Some` of it: CLAP `adjust_size` true with
        // the size unchanged, VST3 `checkSizeConstraint` kResultOk, nice-plug 0.4.2
        // wrapper/vst3/view.rs 758-772; `None` would refuse it)
        guard("adjust_size", || {
            let cur = window.size();
            let scale = cur.scale_factor;
            let l = new_size.cast::<f64>().to_logical(scale);
            let (w, h) =
                adjust_logical((l.width, l.height), (cur.logical.width, cur.logical.height));
            Some(NativeSize::from_size(
                Size::Logical(LogicalSize::new(w, h)),
                scale,
            ))
        })
        .unwrap_or(Some(new_size))
    }

    fn set_fallback_scale_factor(
        &self,
        scale_factor: f64,
        window: &Self::Window,
    ) -> Result<(), Self::Error> {
        guard_err("set_fallback_scale_factor", || {
            self.inner.set_fallback_scale_factor(scale_factor, window)
        })
    }

    fn on_virtual_key_from_host(
        &self,
        key_code: VirtualKeyCode,
        is_down: bool,
        modifiers: Modifiers,
    ) -> bool {
        guard("on_virtual_key_from_host", || {
            self.inner
                .on_virtual_key_from_host(key_code, is_down, modifiers)
        })
        .unwrap_or(false)
    }

    fn state_changed(&self) {
        let _ = guard("state_changed", || self.inner.state_changed());
    }

    fn param_value_changed(&self, id: &str, normalized_value: f32) {
        let _ = guard("param_value_changed", || {
            self.inner.param_value_changed(id, normalized_value)
        });
    }

    fn param_modulation_changed(&self, id: &str, modulation_offset: f32) {
        let _ = guard("param_modulation_changed", || {
            self.inner.param_modulation_changed(id, modulation_offset)
        });
    }
}

/// Runs `f`, an editor entry point the host calls through nice-plug's `extern "C"` functions:
/// a panic there cannot unwind and aborts the host (the user's Reaper check, task-201). A
/// panic becomes an error, logged; the host gets no editor, or the call does nothing.
fn guard<T>(what: &str, f: impl FnOnce() -> T) -> Result<T, String> {
    std::panic::catch_unwind(AssertUnwindSafe(f)).map_err(|p| {
        let msg = p
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| p.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "a panic".into());
        let e = format!("the editor's {what} panicked: {msg}");
        nice_plug::nice_error!("{e}");
        e
    })
}

/// [`guard`] for a call that returns a window error.
fn guard_err<T>(
    what: &str,
    f: impl FnOnce() -> Result<T, BaseviewError>,
) -> Result<T, BaseviewError> {
    guard(what, f).unwrap_or_else(|e| Err(BaseviewError::from(HandlerError::from_boxed(e.into()))))
}

/// The plugin's editor.
pub struct OrbEditor {
    params: Arc<OrbParams>,
    /// The window's scale factor last seen (logical to physical), for [`Editor::size`].
    scale: Arc<Mutex<f64>>,
}

impl OrbEditor {
    pub fn new(params: Arc<OrbParams>) -> Self {
        OrbEditor {
            params,
            scale: Arc::new(Mutex::new(1.0)),
        }
    }

    /// The size the state keeps, or [`DEFAULT_SIZE`] (logical points).
    pub fn logical_size(&self) -> (u32, u32) {
        self.params.editor_size().unwrap_or(DEFAULT_SIZE)
    }
}

impl Editor for OrbEditor {
    type Handle = OrbHandle;

    fn spawn(
        &self,
        parent: Option<ParentWindowHandle>,
        wait_for_parent: bool,
        fallback_scale_factor: Option<f64>,
        gui_context: GuiContext,
        host: Option<HostMethods>,
    ) -> Result<SpawnedEditor<Self::Handle>, Box<dyn Error>> {
        guard("spawn", || {
            self.open(
                parent,
                wait_for_parent,
                fallback_scale_factor,
                gui_context,
                host,
            )
        })
        .unwrap_or_else(|e| Err(e.into()))
    }

    fn size(&self) -> NativeSize<u32> {
        // a panic gives the default size
        guard("size", || {
            let (w, h) = self.logical_size();
            let scale = *self.scale.lock().unwrap_or_else(|e| e.into_inner());
            NativeSize::from_size(Size::Logical(LogicalSize::new(w as f64, h as f64)), scale)
        })
        .unwrap_or_else(|_| NativeSize::new(DEFAULT_SIZE.0, DEFAULT_SIZE.1))
    }

    fn resize_hint(&self) -> ResizeHint {
        guard("resize_hint", resize_hint).unwrap_or_default()
    }
}

impl OrbEditor {
    /// [`Editor::spawn`], under [`guard`].
    fn open(
        &self,
        parent: Option<ParentWindowHandle>,
        wait_for_parent: bool,
        fallback_scale_factor: Option<f64>,
        gui_context: GuiContext,
        host: Option<HostMethods>,
    ) -> Result<SpawnedEditor<OrbHandle>, Box<dyn Error>> {
        let (w, h) = self.logical_size();
        let state =
            EguiEditorState::from_size(Size::Logical(LogicalSize::new(w as f64, h as f64)), 1.0);
        let settings = EguiNiceSettings::new()
            .with_tile(crate::NAME)
            .with_resize_hint(resize_hint());
        // the host's callbacks through a proxy, so that the resize corner reaches them
        let (route, host) = match host {
            Some(h) => {
                let (route, proxy) = HostRoute::register(h.callbacks, self.scale.clone());
                let h = HostMethods {
                    callbacks: proxy,
                    main_thread_caller: h.main_thread_caller,
                };
                (Some(route), Some(h))
            }
            None => (None, None),
        };
        let notifier = RepaintNotifier::new();
        let app = EditorApp {
            params: self.params.clone(),
            scale: self.scale.clone(),
            route,
            notifier: notifier.clone(),
            clock: None,
            app: None,
        };
        let editor = create_egui_editor(state, notifier, settings, app)
            .ok_or("the egui editor could not be made")?;
        let s = editor.spawn(
            parent,
            wait_for_parent,
            fallback_scale_factor,
            gui_context,
            host,
        )?;
        Ok(SpawnedEditor {
            handle: OrbHandle { inner: s.handle },
            window: s.window,
        })
    }
}

/// The egui app of one open editor window.
struct EditorApp {
    params: Arc<OrbParams>,
    scale: Arc<Mutex<f64>>,
    route: Option<HostRoute>,
    /// egui-baseview's realtime-safe repaint flag, polled by its window timer (1/64 s).
    notifier: RepaintNotifier,
    /// egui's repaint requests, turned into `notifier` at their time ([`RepaintClock`]).
    clock: Option<RepaintClock>,
    app: Option<App>,
}

impl NiceEguiApp for EditorApp {
    fn build(
        &mut self,
        egui_ctx: egui::Context,
        gui_context: GuiContext,
        frame: &mut Frame,
    ) -> Result<(), HandlerError> {
        frame.set_key_capture(KeyCapture::IgnoreAll);
        // egui's repaint requests go through the clock, not egui-baseview's waker (RepaintClock)
        let clock = RepaintClock::start({
            let n = self.notifier.clone();
            move || n.request_repaint()
        });
        let requests = clock.handle();
        egui_ctx.set_request_repaint_callback(move |info| requests.request_after(info.delay));
        self.clock = Some(clock);
        // a panic leaves the window empty (`guard`)
        self.app = guard("build", || {
            egui_extras::install_image_loaders(&egui_ctx);
            // the system fonts behind egui's own, as eframe adds them in the standalone app
            egui_ctx.add_font_provider(Arc::new(
                egui_system_font_provider::SystemFontProvider::new(),
            ));
            egui_ctx.set_theme(egui::ThemePreference::Dark);
            debug_assert!(
                has_fonts(),
                "egui has no fonts: its `default_fonts` feature is off (every text would be boxes)"
            );
            let mut app = editor_app_routed(
                self.params.clone(),
                gui_context,
                Settings::load(),
                self.route.clone(),
            );
            app.enable_logo_anim();
            app
        })
        .ok();
        Ok(())
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut Frame) {
        if let Some(app) = &mut self.app
            && guard("frame", || app.show(ui)).is_err()
        {
            // the panel is gone until the editor is opened again
            self.app = None;
        }
        // The editor takes no keys (task-201): every key also goes back to the host, except
        // while a text field (the file browser's) has the focus.
        frame.set_key_capture(if ui.ctx().text_edit_focused() {
            KeyCapture::CaptureAll
        } else {
            KeyCapture::IgnoreAll
        });
    }

    /// The host or the corner resized the window: the size goes into the state.
    fn resized(&mut self, size: WindowSize) {
        // a panic leaves the state's size as it was
        let _ = guard("resized", || {
            *self.scale.lock().unwrap_or_else(|e| e.into_inner()) = size.scale_factor;
            self.params.set_editor_size((
                size.logical.width.round() as u32,
                size.logical.height.round() as u32,
            ));
        });
    }

    fn editor_closed(&mut self) {
        // the clock's thread ends (dropping it stops it)
        self.clock = None;
        // dropping the panel; a panic in its drop is logged
        let app = self.app.take();
        let _ = guard("editor_closed", move || drop(app));
    }
}

/// Whether this build's egui has fonts of its own (its `default_fonts` feature): without them
/// the editor's texts are boxes. `tests/fonts.rs` checks the feature for the plugin's normal
/// (non-dev) dependencies, which a test or an example build cannot show.
pub fn has_fonts() -> bool {
    !egui::FontDefinitions::default().font_data.is_empty()
}

/// The panel of the plugin `params` as an editor: a link to its [`PluginPort`] and its
/// [`EditorEmbed`]; host parameters are set through `gui`, the file browser's folders and the
/// Mods folder come from `settings` (the standalone app's settings file).
pub fn editor_app(params: Arc<OrbParams>, gui: GuiContext, settings: Settings) -> App {
    editor_app_routed(params, gui, settings, None)
}

/// [`editor_app`] with a way to the host's resize request for the resize corner (`None`: the
/// window resizes itself).
pub fn editor_app_routed(
    params: Arc<OrbParams>,
    gui: GuiContext,
    settings: Settings,
    route: Option<HostRoute>,
) -> App {
    let port = PluginPort::new(params.clone(), gui.clone());
    let link = Link::with_port(
        Box::new(port),
        instance::tables(),
        Some(rbm::standard::builtin_samples()),
    );
    let mut embed = EditorEmbed::new(params, gui);
    embed.route = route;
    App::embedded(Box::new(embed), link, settings)
}

/// When the editor window draws next: egui's repaint requests, turned into a wake-up at their
/// time on a thread of the editor's own.
///
/// egui-baseview 0.9 hands egui's requests to baseview's waker (`window.rs` 382-384), and
/// baseview 0.4.2 on macOS loses a request made while the window draws: a zero delay sets the
/// display link running (`platform/macos/waker.rs` 20-26, `wrappers/appkit/main_thread.rs`
/// 73-77 runs it at once on the main thread), and the end of the frame pauses it again
/// (`platform/macos/view.rs` 280-296, `set_next_frame_needed(redraw_requested)`). egui makes
/// such a request at the start of the frame after any immediate one (`begin_pass_repaint_logic`
/// with an outstanding repaint, egui 0.37 `context.rs` 113-125), and the delayed requests of
/// that frame then skip the callback (`context.rs` 164-172, only a shorter delay calls it), so
/// the editor stopped drawing until the mouse moved (the user's Reaper check, 2026-11-16).
/// The clock replaces the callback (`EditorApp::build`): each request moves its due time, and
/// at the due time `wake` sets nice-plug's `RepaintNotifier`, which egui-baseview's window timer
/// polls every 1/64 s outside the frame and turns into `request_redraw`. No request, no wake-up:
/// the thread sleeps, so a still panel costs nothing; while the engine plays the panel asks
/// at its meter rate (orb-ui's repaint rule), and the plugin's LEDs every 50 ms (`embed::POLL`).
pub struct RepaintClock {
    shared: Arc<ClockShared>,
    thread: Option<std::thread::JoinHandle<()>>,
}

/// The clock's state and its condition variable.
struct ClockShared {
    state: Mutex<ClockState>,
    changed: std::sync::Condvar,
}

#[derive(Default)]
struct ClockState {
    /// The earliest requested repaint.
    due: Option<std::time::Instant>,
    stop: bool,
}

/// What egui's repaint callback holds: requests to the clock.
#[derive(Clone)]
pub struct RepaintRequests(Arc<ClockShared>);

impl RepaintRequests {
    /// A repaint is wanted after `delay` (`Duration::MAX`: never).
    pub fn request_after(&self, delay: std::time::Duration) {
        let Some(at) = std::time::Instant::now().checked_add(delay) else {
            return;
        };
        let mut s = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        if s.due.is_none_or(|d| at < d) {
            s.due = Some(at);
            self.0.changed.notify_one();
        }
    }
}

impl RepaintClock {
    /// Starts the clock's thread, which calls `wake` at each due time.
    pub fn start(wake: impl Fn() + Send + 'static) -> Self {
        let shared = Arc::new(ClockShared {
            state: Mutex::new(ClockState::default()),
            changed: std::sync::Condvar::new(),
        });
        let s = shared.clone();
        let thread = std::thread::Builder::new()
            .name("orb editor repaint".into())
            .spawn(move || {
                let _ = guard("repaint clock", || clock_loop(&s, &wake));
            })
            .ok();
        RepaintClock { shared, thread }
    }

    /// The requests' side, for egui's callback.
    pub fn handle(&self) -> RepaintRequests {
        RepaintRequests(self.shared.clone())
    }
}

impl Drop for RepaintClock {
    fn drop(&mut self) {
        {
            let mut s = self.shared.state.lock().unwrap_or_else(|e| e.into_inner());
            s.stop = true;
            self.shared.changed.notify_one();
        }
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

fn clock_loop(shared: &ClockShared, wake: &dyn Fn()) {
    let mut s = shared.state.lock().unwrap_or_else(|e| e.into_inner());
    loop {
        if s.stop {
            return;
        }
        match s.due {
            None => s = shared.changed.wait(s).unwrap_or_else(|e| e.into_inner()),
            Some(at) => {
                let now = std::time::Instant::now();
                if now >= at {
                    s.due = None;
                    drop(s);
                    wake();
                    s = shared.state.lock().unwrap_or_else(|e| e.into_inner());
                } else {
                    s = shared
                        .changed
                        .wait_timeout(s, at - now)
                        .unwrap_or_else(|e| e.into_inner())
                        .0;
                }
            }
        }
    }
}

/// The host's callbacks of an editor window, shared by the window (egui-baseview holds the
/// [`HostProxy`]) and the resize corner, on the window's thread.
type Callbacks = Rc<RefCell<Box<dyn HostCallbacks>>>;

thread_local! {
    /// The host callbacks of the editor windows of this thread, by [`HostRoute`] id.
    static ROUTES: RefCell<BTreeMap<u64, Rc<RouteState>>> =
        const { RefCell::new(BTreeMap::new()) };
}

static NEXT_ROUTE: AtomicU64 = AtomicU64::new(1);

/// The way from an editor's resize corner to the host's resize request: nice-plug hands the
/// host's callbacks (`HostMethods`) to `Editor::spawn` for baseview, whose own `resize` does not
/// call them (baseview 0.4.2 `Window::resize`). [`HostRoute::register`] puts a [`HostProxy`]
/// in between that keeps them reachable, by id, from the thread that spawned the window (the
/// main thread on macOS and Windows, where the window runs; on X11 the window has a thread of
/// its own, the route is not found there and the window resizes itself).
///
/// The request is not made inside the egui frame: egui-baseview 0.9 holds its window state
/// borrowed for the whole frame (`window.rs` 584-592), and a host that answers with
/// `set_size` at once (Reaper; CLAP `request_resize` and VST3 `resizeView` may) resizes the
/// window, whose `resized` borrows that state again (`window.rs` 736) and panics. The corner
/// only records the size ([`HostRoute::request`]); a callback posted to the window's thread
/// ([`post`]: libdispatch's main queue on macOS, a thread timer on Windows) makes the request
/// after the frame ([`HostRoute::flush`]).
#[derive(Clone)]
pub struct HostRoute {
    id: u64,
    /// The window's scale factor last seen.
    scale: Arc<Mutex<f64>>,
}

/// A route's state on the window's thread.
struct RouteState {
    callbacks: Callbacks,
    /// The size the corner wants and its scale factor, not yet asked for.
    pending: std::cell::Cell<Option<(f64, f64, f64)>>,
    /// A flush is posted.
    posted: std::cell::Cell<bool>,
    /// Posts the flush of a route to this thread ([`post`]; tests post nothing and flush).
    post: fn(u64) -> bool,
}

/// The host callbacks as egui-baseview gets them: the shared [`Callbacks`]; dropped with the
/// window, it forgets the route. It holds an `Rc`, so it is not `Send` and is dropped on the
/// thread that registered it, where the route lives.
pub struct HostProxy {
    id: u64,
    inner: Callbacks,
}

impl HostCallbacks for HostProxy {
    fn request_resize(&mut self, new_size: Size, scale_factor: f64) -> Result<(), Box<dyn Error>> {
        match self.inner.try_borrow_mut() {
            Ok(mut c) => c.request_resize(new_size, scale_factor),
            Err(_) => Err("the host's callbacks are in use".into()),
        }
    }

    fn destroyed(&mut self) {
        if let Ok(mut c) = self.inner.try_borrow_mut() {
            c.destroyed();
        }
    }
}

impl Drop for HostProxy {
    fn drop(&mut self) {
        let id = self.id;
        let _ = ROUTES.try_with(|r| r.borrow_mut().remove(&id));
        cancel_posts(id);
    }
}

impl HostRoute {
    /// Registers `callbacks` on this thread: the route for the corner and the proxy for the
    /// window.
    pub fn register(
        callbacks: Box<dyn HostCallbacks>,
        scale: Arc<Mutex<f64>>,
    ) -> (HostRoute, Box<dyn HostCallbacks>) {
        Self::register_with(callbacks, scale, post)
    }

    /// [`HostRoute::register`] with a function of its own that posts the flush (tests: one
    /// that posts nothing, then [`HostRoute::flush`] by hand).
    pub fn register_with(
        callbacks: Box<dyn HostCallbacks>,
        scale: Arc<Mutex<f64>>,
        post: fn(u64) -> bool,
    ) -> (HostRoute, Box<dyn HostCallbacks>) {
        let id = NEXT_ROUTE.fetch_add(1, Ordering::Relaxed);
        let inner: Callbacks = Rc::new(RefCell::new(callbacks));
        let state = Rc::new(RouteState {
            callbacks: inner.clone(),
            pending: Default::default(),
            posted: Default::default(),
            post,
        });
        ROUTES.with(|r| r.borrow_mut().insert(id, state));
        (HostRoute { id, scale }, Box::new(HostProxy { id, inner }))
    }

    /// The corner wants a window of `w` x `h` logical points: recorded, and asked for after
    /// the frame. `Some(true)` when it will be; `None` when the callbacks are not reachable
    /// from this thread or nothing can be posted to it (the window then resizes itself).
    pub fn request(&self, w: f32, h: f32) -> Option<bool> {
        let s = ROUTES.with(|r| r.borrow().get(&self.id).cloned())?;
        let scale = *self.scale.lock().unwrap_or_else(|e| e.into_inner());
        s.pending.set(Some((w as f64, h as f64, scale)));
        if !s.posted.get() {
            if !(s.post)(self.id) {
                s.pending.set(None);
                return None;
            }
            s.posted.set(true);
        }
        Some(true)
    }

    /// Asks the host for the size the corner recorded, if any (the posted callback; tests).
    /// Returns whether the host accepted it.
    pub fn flush(&self) -> Option<bool> {
        flush_route(self.id)
    }
}

/// [`HostRoute::flush`] of the route `id` of this thread.
fn flush_route(id: u64) -> Option<bool> {
    let s = ROUTES.with(|r| r.borrow().get(&id).cloned())?;
    s.posted.set(false);
    let (w, h, scale) = s.pending.take()?;
    let size = Size::Logical(LogicalSize::new(w, h));
    let ok = match s.callbacks.try_borrow_mut() {
        Ok(mut c) => c.request_resize(size, scale).is_ok(),
        Err(_) => false,
    };
    Some(ok)
}

/// Runs [`flush_route`] of route `id` on this thread once the current event (the egui frame)
/// is over: libdispatch's main queue (the window's thread on macOS). `false`: not posted.
#[cfg(target_os = "macos")]
fn post(id: u64) -> bool {
    use std::ffi::c_void;
    unsafe extern "C" {
        static _dispatch_main_q: c_void;
        fn dispatch_async_f(
            queue: *const c_void,
            context: *mut c_void,
            work: extern "C" fn(*mut c_void),
        );
    }
    extern "C" fn run(context: *mut c_void) {
        let id = context as usize as u64;
        let _ = guard("resize request", || flush_route(id));
    }
    if !is_main_thread() {
        return false;
    }
    // SAFETY: `_dispatch_main_q` is the main queue (`dispatch_get_main_queue()`); the context
    // is a plain number.
    unsafe { dispatch_async_f(&raw const _dispatch_main_q, id as usize as *mut c_void, run) };
    true
}

/// Whether this is the process's main thread (macOS), where the main queue runs.
#[cfg(target_os = "macos")]
fn is_main_thread() -> bool {
    unsafe extern "C" {
        fn pthread_main_np() -> i32;
    }
    // SAFETY: no arguments, no state.
    unsafe { pthread_main_np() != 0 }
}

/// [`post`] on Windows: a thread timer of 0 ms (`SetTimer` without a window), whose
/// `WM_TIMER` the host's message loop dispatches after the current message.
#[cfg(windows)]
type TimerProc = unsafe extern "system" fn(*mut std::ffi::c_void, u32, usize, u32);

#[cfg(windows)]
#[link(name = "user32")]
unsafe extern "system" {
    fn SetTimer(hwnd: *mut std::ffi::c_void, id: usize, ms: u32, proc_: Option<TimerProc>)
    -> usize;
    fn KillTimer(hwnd: *mut std::ffi::c_void, id: usize) -> i32;
}

#[cfg(windows)]
thread_local! {
    /// The route of each pending timer of this thread ([`post`]).
    static TIMERS: RefCell<BTreeMap<usize, u64>> = const { RefCell::new(BTreeMap::new()) };
}

/// Cancels the posted flushes of route `id` (its window closes): Windows kills its timers.
/// macOS cannot take back a block on the main queue; it finds no route and does nothing,
/// running this image's code once more (docs/plugin.md, the residual risk).
#[cfg(windows)]
fn cancel_posts(id: u64) {
    let timers: Vec<usize> = TIMERS
        .try_with(|t| {
            let mut t = t.borrow_mut();
            let mine: Vec<usize> = t.iter().filter(|e| *e.1 == id).map(|e| *e.0).collect();
            for k in &mine {
                t.remove(k);
            }
            mine
        })
        .unwrap_or_default();
    for timer in timers {
        // SAFETY: a timer of this thread, without a window.
        unsafe { KillTimer(std::ptr::null_mut(), timer) };
    }
}

/// [`cancel_posts`] where nothing can be cancelled.
#[cfg(not(windows))]
fn cancel_posts(_id: u64) {}

#[cfg(windows)]
fn post(id: u64) -> bool {
    use std::ffi::c_void;
    unsafe extern "system" fn tick(_: *mut c_void, _: u32, timer: usize, _: u32) {
        // SAFETY: a timer of this thread, without a window.
        unsafe { KillTimer(std::ptr::null_mut(), timer) };
        if let Some(id) = TIMERS.with(|t| t.borrow_mut().remove(&timer)) {
            let _ = guard("resize request", || flush_route(id));
        }
    }
    // SAFETY: a thread timer with a callback of the right signature.
    let timer = unsafe { SetTimer(std::ptr::null_mut(), 0, 0, Some(tick)) };
    if timer == 0 {
        return false;
    }
    TIMERS.with(|t| t.borrow_mut().insert(timer, id));
    true
}

/// [`post`] elsewhere: nothing to post to (on X11 the route is not found anyway).
#[cfg(not(any(target_os = "macos", windows)))]
fn post(_id: u64) -> bool {
    false
}

/// Sets parameter `i` of [`SPECS`] to `v` through the host, as one gesture; nothing when it
/// holds `v`.
fn set_param(params: &OrbParams, gui: &GuiContext, i: usize, v: i32) {
    let p = params.param_at(i);
    if p.value() == v {
        return;
    }
    let s = gui.param_setter();
    s.begin_set_parameter(p);
    s.set_parameter(p, v);
    s.end_set_parameter(p);
}

/// The panel's player in the plugin.
pub struct PluginPort {
    params: Arc<OrbParams>,
    gui: GuiContext,
    snapshot: Snapshot,
    tracks: Tracks,
}

impl PluginPort {
    pub fn new(params: Arc<OrbParams>, gui: GuiContext) -> Self {
        PluginPort {
            params,
            gui,
            snapshot: Snapshot::default(),
            tracks: Tracks::default(),
        }
    }
}

impl Port for PluginPort {
    fn send(&mut self, c: Command) -> Result<(), Command> {
        let p = &self.params;
        // a failure (a full queue) drops the change: the panel then follows the engine
        let _ = match c {
            // orb's Play/Stop enter and leave the preview (task-217)
            Command::Play => {
                p.set_preview(true);
                Ok(())
            }
            Command::Stop => {
                p.set_preview(false);
                Ok(())
            }
            Command::PatternField { .. }
            | Command::PatternHeader { .. }
            | Command::Pattern { .. }
            | Command::StepField { .. }
            | Command::Step { .. } => p.edit(c),
            Command::Vintage(on) => p.set_vintage(on),
            Command::Preview(_) | Command::PreviewStop => p.preview(c),
            c => match params::of_command(&c) {
                // in the preview a control goes to the preview set, not to the host
                Some((i, v)) if p.previewing() => p.set_preview_value(i, v),
                Some((i, v)) => {
                    // a pattern button on the pattern the parameter holds: a host note has
                    // selected another since; the panel's message selects it again
                    if let Target::Ctrl(d, 1) = SPECS[i].target
                        && let Some(row) = ROWS.iter().position(|&r| r == d)
                        && p.param_at(i).value() == v
                    {
                        p.select_pattern(row, v as u8)
                    } else {
                        set_param(p, &self.gui, i, v);
                        Ok(())
                    }
                }
                // the song mode, record, loop, locate and tempo: the plugin has none
                None => Ok(()),
            },
        };
        Ok(())
    }

    fn snapshot(&mut self) -> &Snapshot {
        // before the first activation the engine has run no call: no values to take
        self.snapshot = self.params.snapshot().unwrap_or_default();
        &self.snapshot
    }

    fn tracks(&mut self) -> &Tracks {
        &self.tracks
    }

    fn records(&self) -> bool {
        false
    }

    fn collect_garbage(&mut self) {
        self.params.collect_garbage();
    }

    fn build(
        &self,
        _song: &Song,
        _samples: Option<Arc<SampleSet>>,
    ) -> Result<Box<Engine>, LoadRefused> {
        unreachable!("the plugin's editor loads songs through Embed::load_song")
    }

    fn build_new(&self, _samples: Option<Arc<SampleSet>>) -> Box<Engine> {
        unreachable!("the plugin's editor makes new songs through Embed::load_song")
    }

    /// The plugin took the mod already (`Embed::set_mod`).
    fn set_mod(&mut self, _samples: Arc<SampleSet>) -> Result<(), Command> {
        Ok(())
    }

    fn follow_frames(&self) -> u64 {
        FOLLOW_FRAMES
    }
}

/// The mod files the editors of this process chose, by the SHA-256 of their bytes: an editor
/// opened again names the file (the Mods menu checks it). Not saved: a restored state's mod
/// is a copy in the temporary folder ([`EditorEmbed::kit_file`]).
fn mod_files() -> std::sync::MutexGuard<'static, BTreeMap<String, PathBuf>> {
    static FILES: Mutex<BTreeMap<String, PathBuf>> = Mutex::new(BTreeMap::new());
    FILES.lock().unwrap_or_else(|e| e.into_inner())
}

/// The plugin's side of the panel.
pub struct EditorEmbed {
    params: Arc<OrbParams>,
    gui: GuiContext,
    /// The parameter values of the last load, until the panel has shown them (the host may
    /// apply the editor's changes later).
    pending: Option<Vec<(String, i32)>>,
    /// The way to the host's resize request.
    route: Option<HostRoute>,
}

impl EditorEmbed {
    pub fn new(params: Arc<OrbParams>, gui: GuiContext) -> Self {
        EditorEmbed {
            params,
            gui,
            pending: None,
            route: None,
        }
    }

    /// A file holding the instance's mod: the one the editor chose, else a copy of the state's
    /// bytes in the temporary folder (the 808's mod window and Mods > Info read the file).
    fn kit_file(&self) -> Option<(String, PathBuf)> {
        let Kit::Mod {
            name,
            sha256,
            bytes,
        } = self.params.kit()
        else {
            return None;
        };
        if let Some(p) = mod_files().get(&sha256).filter(|p| p.is_file()) {
            return Some((name, p.clone()));
        }
        let dir = std::env::temp_dir().join("OpenRB-338").join("mods");
        let path = dir.join(format!("{sha256}.rbm"));
        if !path.is_file() {
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(&path, bytes.as_slice());
        }
        Some((name, path))
    }
}

impl Embed for EditorEmbed {
    fn revision(&self) -> u64 {
        self.params.revision()
    }

    fn view(&mut self) -> SongView {
        // the preview set's values while the preview runs
        let mut values = self.params.panel_values();
        for (id, v) in self.pending.take().unwrap_or_default() {
            if let Some(i) = self.params.ids().iter().position(|x| *x == id) {
                values[i] = v;
            }
        }
        let (snapshot, patterns) = self.params.view(&values);
        SongView {
            song: self.params.song(),
            kit: self.kit_file(),
            patterns,
            snapshot,
        }
    }

    fn load_song(&mut self, file: Option<Vec<u8>>) -> Result<(), String> {
        let open = file.is_some();
        let changes = match file {
            Some(b) => self.params.load_song(b)?,
            None => self.params.new_song()?,
        };
        for (id, v) in &changes {
            if let Some(i) = self.params.ids().iter().position(|x| x == id) {
                set_param(&self.params, &self.gui, i, *v);
            }
        }
        // File > Open always switches to pattern mode (task-185)
        if open {
            self.params.set_mode(Mode::Pattern);
        }
        self.pending = Some(changes);
        Ok(())
    }

    fn set_mod(&mut self, file: Option<&Path>) -> Result<(), String> {
        let Some(path) = file else {
            return self.params.set_mod(None);
        };
        let bytes =
            std::fs::read(path).map_err(|e| format!("{}: cannot read: {e}", path.display()))?;
        let sha = instance::sha256_hex(&bytes);
        self.params.set_mod(Some(bytes))?;
        mod_files().insert(sha, path.to_path_buf());
        Ok(())
    }

    fn take_error(&mut self) -> Option<String> {
        self.params.take_error()
    }

    fn request_size(&mut self, size: egui::Vec2) -> Option<bool> {
        self.route.as_ref()?.request(size.x, size.y)
    }

    fn mode(&self) -> Option<PluginMode> {
        Some(match self.params.mode() {
            Mode::Pattern => PluginMode::Pattern,
            Mode::Note => PluginMode::Note,
        })
    }

    fn previewing(&self) -> Option<bool> {
        Some(self.params.previewing())
    }

    fn set_mode(&mut self, mode: PluginMode) {
        self.params.set_mode(match mode {
            PluginMode::Pattern => Mode::Pattern,
            PluginMode::Note => Mode::Note,
        });
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicU32;
    use std::time::{Duration, Instant};

    use super::*;

    /// Waits up to a second for `n` wake-ups.
    fn wait_for(count: &AtomicU32, n: u32) -> bool {
        let end = Instant::now() + Duration::from_secs(1);
        while Instant::now() < end {
            if count.load(Ordering::SeqCst) >= n {
                return true;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        false
    }

    #[test]
    fn the_repaint_clock_wakes_at_the_requested_time_and_not_otherwise() {
        let count = Arc::new(AtomicU32::new(0));
        let clock = RepaintClock::start({
            let c = count.clone();
            move || {
                c.fetch_add(1, Ordering::SeqCst);
            }
        });
        let r = clock.handle();
        // no request: no wake-up
        std::thread::sleep(Duration::from_millis(40));
        assert_eq!(count.load(Ordering::SeqCst), 0);
        // never (egui's "no repaint")
        r.request_after(Duration::MAX);
        std::thread::sleep(Duration::from_millis(40));
        assert_eq!(count.load(Ordering::SeqCst), 0);
        // a delayed request: not before its time, then once
        let t0 = Instant::now();
        r.request_after(Duration::from_millis(60));
        assert!(wait_for(&count, 1));
        assert!(
            t0.elapsed() >= Duration::from_millis(60),
            "{:?}",
            t0.elapsed()
        );
        std::thread::sleep(Duration::from_millis(40));
        assert_eq!(count.load(Ordering::SeqCst), 1, "one wake-up per request");
        // an immediate request (egui's zero delay, also from inside a frame) wakes at once;
        // a later request does not delay an earlier one
        let t0 = Instant::now();
        r.request_after(Duration::ZERO);
        r.request_after(Duration::from_secs(5));
        assert!(wait_for(&count, 2));
        assert!(t0.elapsed() < Duration::from_millis(500));
        // dropping the clock ends its thread
        drop(clock);
        r.request_after(Duration::ZERO);
        std::thread::sleep(Duration::from_millis(30));
        assert_eq!(count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn a_panic_in_an_entry_point_becomes_an_error() {
        assert_eq!(guard("x", || 3), Ok(3));
        let e = guard("spawn", || -> i32 { panic!("could not create new class") }).unwrap_err();
        assert_eq!(e, "the editor's spawn panicked: could not create new class");
        let e = guard_err("set_parent", || -> Result<(), BaseviewError> {
            panic!("{}", String::from("boom"))
        })
        .unwrap_err();
        assert!(e.to_string().contains("set_parent panicked: boom"), "{e}");
    }
}
