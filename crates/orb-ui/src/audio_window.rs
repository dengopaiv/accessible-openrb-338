//! Options > Audio Output... (task-191): the window that chooses the driver, the device, the
//! sample rate and the buffer size of the sound, and shows what is in use.
//!
//! The four settings are [`AudioChoice`] ([`crate::settings`]); a change applies at once
//! ([`App::audio_choose`]): the stream reopens through [`orb_player::control::Controller`], the
//! song keeps playing after a short gap, and the choice is saved.

use std::time::{Duration, Instant};

use egui::{self, Context, Key, Modifiers, RichText};
use orb_player::control::Event;
use orb_player::devices::{self, Caps, DeviceInfo};
use orb_player::output::InUse;

use crate::a11y::window as win;
use crate::app::{App, Win, focus_into_window};
use crate::layout::DIALOG_MARGIN;
use crate::palette::INK_DIM;
use crate::settings::AudioChoice;

/// The label of the system default device.
pub const SYSTEM_DEFAULT: &str = "System default";
/// The label of the device's default rate.
pub const DEVICE_DEFAULT: &str = "Device default";

/// What the window knows of the drivers and devices: the drivers when it opens; the devices
/// when it opens, when the driver changes and when the Device list opens; the selected device's
/// rates and sizes also when the Sample rate or Buffer size list opens; never on every frame (a
/// driver may take a while to answer).
#[derive(Default)]
pub struct AudioUi {
    drivers: Vec<String>,
    devices: Vec<DeviceInfo>,
    /// The device's rates and sizes (`None`: the driver does not say).
    caps: Option<Caps>,
    /// Why the lists are short, shown under the settings.
    problem: Option<String>,
    /// The In use numbers as last sampled and when ([`AudioUi::sample`]).
    shown: Option<(Instant, InUse)>,
    /// What the controller runs when that is not the saved choice (a fallback from a lost
    /// device, task-245): the window shows it as its chosen values. Never saved: the saved
    /// choice changes only when the user picks in the window.
    in_use_choice: Option<AudioChoice>,
    /// The lists are to be read again for [`AudioUi::in_use_choice`] as soon as no list is open
    /// (a list is not replaced under the user).
    refresh_pending: bool,
    /// A test seam, set only by `App::audio_open_for_test`: the lists are fixed and never
    /// read from the drivers. The real window never sets it.
    lists_fixed: bool,
    /// The highlight of the open list in screen reader mode.
    nav: ListNav,
}

/// How often the In use numbers (output latency, underruns, the granted size) are sampled: the
/// window's own repaint period. A repaint caused by input shows the sampled copy again.
pub const SAMPLE_EVERY: Duration = Duration::from_millis(500);

/// Whether two readings are of one stream setup (all but the measured numbers).
fn same_setup(a: &InUse, b: &InUse) -> bool {
    let blank = |u: &InUse| InUse {
        granted: None,
        output_latency_ns: None,
        underruns: 0,
        ..u.clone()
    };
    blank(a) == blank(b)
}

/// Whether `driver`'s buffer is set in the driver's own control panel and not asked for by
/// the program (ASIO, task-193).
pub fn buffer_is_the_drivers(driver: &str) -> bool {
    devices::is_asio(Some(driver))
}

/// The driver a choice names: its own, else the platform's first.
fn driver_of<'a>(drivers: &'a [String], choice: &'a AudioChoice) -> &'a str {
    choice
        .driver
        .as_deref()
        .or_else(|| drivers.first().map(String::as_str))
        .unwrap_or("")
}

impl AudioUi {
    /// The In use numbers to draw at `now`, given the stream's `live` reading: the copy sampled
    /// at most [`SAMPLE_EVERY`] ago, so the numbers change at a steady rate and not with every
    /// repaint (the mouse moving). A new sample is taken when none is held, when the period has
    /// passed or when the stream's setup changed (a reopen shows at once).
    pub fn sample(&mut self, now: Instant, live: Option<InUse>) -> Option<InUse> {
        let Some(live) = live else {
            self.shown = None;
            return None;
        };
        let keep = matches!(&self.shown, Some((at, old))
            if now.saturating_duration_since(*at) < SAMPLE_EVERY && same_setup(old, &live));
        if !keep {
            self.shown = Some((now, live));
        }
        self.shown.as_ref().map(|(_, u)| u.clone())
    }

    /// Reads the drivers, the driver's devices and the device's capabilities for `choice`.
    pub fn refresh(&mut self, choice: &AudioChoice) {
        self.drivers = devices::drivers();
        self.refresh_devices(choice);
    }

    /// Reads the driver's devices and the selected device's capabilities again.
    pub fn refresh_devices(&mut self, choice: &AudioChoice) {
        self.problem = None;
        self.devices = match devices::list_devices(choice.driver.as_deref()) {
            Ok(d) => d,
            Err(e) => {
                self.problem = Some(format!("The devices cannot be listed: {e}"));
                Vec::new()
            }
        };
        self.refresh_caps(choice);
    }

    /// Reads the selected device's rates and buffer sizes again.
    pub fn refresh_caps(&mut self, choice: &AudioChoice) {
        self.caps = match devices::caps(choice.driver.as_deref(), choice.device.as_deref()) {
            Ok(c) => Some(c),
            Err(e) => {
                self.problem = Some(format!("The device's settings cannot be read: {e}"));
                None
            }
        };
    }

    /// The rates to offer (after "Device default").
    fn rates(&self) -> &[u32] {
        self.caps.as_ref().map_or(&[], |c| &c.rates)
    }

    /// The size a saved `want` really gets on the selected device (clamped into its range), the
    /// one the Buffer size list shows as selected.
    fn requested_buffer(&self, want: u32) -> u32 {
        self.caps.as_ref().map_or(want, |c| c.requested(want))
    }

    /// The buffer sizes to offer.
    fn buffers(&self) -> Vec<u32> {
        match &self.caps {
            Some(c) => c.buffers.clone(),
            None => devices::default_buffer_sizes(),
        }
    }
}

/// The choice after the user picked driver `name` in the list of `drivers`: the platform's
/// first driver is stored as none, and the device, the rate and the size start over.
pub fn after_driver(drivers: &[String], name: &str) -> AudioChoice {
    AudioChoice {
        driver: if drivers.first().map(String::as_str) == Some(name) {
            None
        } else {
            Some(name.to_string())
        },
        ..AudioChoice::default()
    }
}

/// The choice after the user picked a device (`None`: System default): the rate stays when the
/// device offers it, the size moves to the nearest the device offers.
pub fn after_device(
    old: &AudioChoice,
    device: Option<&DeviceInfo>,
    caps: Option<&Caps>,
) -> AudioChoice {
    let mut c = AudioChoice {
        device: device.map(|d| d.id.clone()),
        device_name: device.map(|d| d.name.clone()),
        ..old.clone()
    };
    if let Some(caps) = caps {
        c.rate = c.rate.filter(|r| caps.rates.contains(r));
        c.buffer = devices::nearest_buffer(&caps.buffers, c.buffer);
    }
    c
}

/// A buffer size with its time at `rate` Hz, e.g. `2048 frames (46.4 ms)`.
pub fn frames_text(frames: u32, rate: u32) -> String {
    format!(
        "{frames} frames ({:.1} ms)",
        devices::latency_ms(frames, rate)
    )
}

/// Screen reader mode: gives the combo box `r` the name `name` (what it chooses); the selected
/// text is its value, and its description says how to open it (task-248). The label beside it in
/// the grid is not what a screen reader reads.
fn name_combo(ctx: &Context, on: bool, r: &egui::Response, name: &str) {
    if on {
        ctx.accesskit_node_builder(r.id, |b| {
            b.set_label(name);
            b.set_description(OPEN_HINT);
        });
    }
}

/// How a combo box opens, read after its value in screen reader mode.
pub const OPEN_HINT: &str = "Alt+Down opens the list";

/// Takes a press of `key` with exactly the modifiers `mods` (egui's `consume_key` also matches
/// an extra Alt or Shift, so Alt+Up would act as Up). True when there was one.
fn consume_exact(i: &mut egui::InputState, mods: Modifiers, key: Key) -> bool {
    let mut found = false;
    i.events.retain(|e| match e {
        egui::Event::Key {
            key: k,
            pressed: true,
            modifiers,
            ..
        } if *k == key && modifiers.matches_exact(mods) => {
            found = true;
            false
        }
        _ => true,
    });
    found
}

/// What one of the window's lists offers: the text shown on its box, its entries and the one
/// chosen now (`None`: the choice is not among the entries).
struct ComboSpec<'a> {
    salt: &'a str,
    name: &'a str,
    text: String,
    labels: Vec<String>,
    selected: Option<usize>,
}

/// What [`audio_combo`] found: the choice made by a click or a key, and whether a key opened
/// the list (a click is the response's, as upstream's window reads it).
struct ComboOut {
    chosen: Option<AudioChoice>,
    opened: bool,
}

/// Screen reader mode, where the highlight of the open list is: the list's open box, the
/// highlighted entry, and the box that takes the focus back when the list closes. The open
/// list's entries are real focus targets, so a screen reader reads the highlight.
#[derive(Default)]
pub struct ListNav {
    open: Option<egui::Id>,
    cursor: usize,
    refocus: Option<egui::Id>,
    /// The box that had the focus at the end of the last frame (by its list's id): the focus of
    /// the response is known only once the box is drawn, and a list opens only before that.
    focused: Option<egui::Id>,
}

/// What a combo box of a Windows dialog does with the keyboard, in screen reader mode (task-248):
/// Alt+Down opens the list (Space and Enter do already); on a closed box Up and Down choose the
/// previous and next entry, Home and End the first and last, at once and without wrapping;
/// in the open list Up, Down, Home and End move the highlight, Enter, Space or Alt+Up choose it,
/// and Escape closes the list and keeps the choice; the focus is on the box afterwards. Without the
/// mode this is egui's `ComboBox` and its list with the clicks alone.
fn audio_combo(
    ui: &mut egui::Ui,
    sr: bool,
    nav: &mut ListNav,
    spec: &ComboSpec,
    resolve: &dyn Fn(usize) -> Option<AudioChoice>,
) -> ComboOut {
    let ctx = ui.ctx().clone();
    let id = ui.make_persistent_id(egui::IdSalt::new(spec.salt));
    let last = spec.labels.len().saturating_sub(1);
    let mut out = ComboOut {
        chosen: None,
        opened: false,
    };
    let was_open = egui::ComboBox::is_open(&ctx, id);
    if sr
        && nav.focused == Some(id)
        && !was_open
        && ctx.input_mut(|i| consume_exact(i, Modifiers::ALT, Key::ArrowDown))
    {
        egui::Popup::open_id(&ctx, id.with("popup"));
        out.opened = true;
    }
    let r = egui::ComboBox::from_id_salt(spec.salt)
        .selected_text(spec.text.clone())
        .width(280.0)
        .show_ui(ui, |ui| {
            let (up, down, home, end, alt_up, escape) = if sr {
                ui.ctx().input_mut(|i| {
                    (
                        consume_exact(i, Modifiers::NONE, Key::ArrowUp),
                        consume_exact(i, Modifiers::NONE, Key::ArrowDown),
                        consume_exact(i, Modifiers::NONE, Key::Home),
                        consume_exact(i, Modifiers::NONE, Key::End),
                        consume_exact(i, Modifiers::ALT, Key::ArrowUp),
                        i.key_pressed(Key::Escape),
                    )
                })
            } else {
                Default::default()
            };
            // the list just opened: the highlight starts on the chosen entry
            let mut moved = sr && nav.open != Some(id);
            if moved {
                nav.open = Some(id);
                nav.cursor = spec.selected.unwrap_or(0);
            }
            let was = nav.cursor;
            if home {
                nav.cursor = 0;
            } else if end {
                nav.cursor = last;
            } else if up {
                nav.cursor = nav.cursor.saturating_sub(1);
            } else if down {
                nav.cursor = (nav.cursor + 1).min(last);
            }
            moved |= nav.cursor != was;
            let mut clicked = None;
            for (i, label) in spec.labels.iter().enumerate() {
                let item = ui.selectable_label(spec.selected == Some(i), label);
                if item.clicked() {
                    clicked = Some(i);
                }
                if sr {
                    if moved {
                        if i == nav.cursor {
                            item.request_focus();
                        }
                    } else if item.has_focus() {
                        // Tab or a click moved the focus: the highlight follows
                        nav.cursor = i;
                    }
                    if item.has_focus() {
                        // egui keeps the arrows and Escape off its own focus moves. It takes
                        // the filter of a widget that had the focus last frame only, so a
                        // key in the first frame after Tab is unguarded; tried in a test
                        // (audio_output_device_and_driver_follow_the_arrows: Tab and Down in
                        // consecutive frames) and nothing went wrong, and a person does not
                        // type that fast
                        let filter = egui::EventFilter {
                            vertical_arrows: true,
                            escape: true,
                            ..Default::default()
                        };
                        ui.memory_mut(|m| m.set_focus_lock_filter(item.id, filter));
                    }
                }
            }
            if alt_up {
                clicked = Some(nav.cursor);
            }
            if sr && clicked.is_some() {
                // a key's choice does not close the list as a click does
                ui.close();
            }
            if sr && (escape || clicked.is_some()) {
                nav.refocus = Some(id);
            }
            clicked
        });
    if sr {
        if !egui::ComboBox::is_open(&ctx, id) && nav.open == Some(id) {
            nav.open = None;
        }
        if nav.refocus == Some(id) {
            nav.refocus = None;
            r.response.request_focus();
        }
        if r.response.has_focus() {
            nav.focused = Some(id);
            let filter = egui::EventFilter {
                vertical_arrows: true,
                escape: true,
                ..Default::default()
            };
            ctx.memory_mut(|m| m.set_focus_lock_filter(r.response.id, filter));
        } else if nav.focused == Some(id) {
            nav.focused = None;
        }
    }
    if sr && r.response.has_focus() && !was_open && !egui::ComboBox::is_open(&ctx, id) {
        let (up, down, home, end) = ctx.input_mut(|i| {
            (
                consume_exact(i, Modifiers::NONE, Key::ArrowUp),
                consume_exact(i, Modifiers::NONE, Key::ArrowDown),
                consume_exact(i, Modifiers::NONE, Key::Home),
                consume_exact(i, Modifiers::NONE, Key::End),
            )
        });
        // no entry chosen: Down goes to the first, Up to the last
        let to = match spec.selected {
            _ if home => Some(0),
            _ if end => Some(last),
            Some(s) if up => Some(s.saturating_sub(1)),
            Some(s) if down => Some((s + 1).min(last)),
            None if up => Some(last),
            None if down => Some(0),
            _ => None,
        };
        if let Some(i) = to.filter(|&i| Some(i) != spec.selected && i < spec.labels.len()) {
            out.chosen = resolve(i);
        }
    }
    name_combo(&ctx, sr, &r.response, spec.name);
    if let Some(i) = r.inner.flatten()
        && spec.selected != Some(i)
    {
        out.chosen = resolve(i);
    }
    out.opened |= r.response.clicked();
    out
}

/// One row of the In use list: the label, and the text as a line that takes the focus in screen
/// reader mode and is read as "label: text" (`win::line`). Without the mode two plain labels.
fn in_use_row(ui: &mut egui::Ui, on: bool, label: &str, text: impl Into<String>) {
    let text = text.into();
    ui.label(label);
    let spoken = format!("{label}: {text}");
    win::line(ui, on, RichText::new(text), false, Some(&spoken));
    ui.end_row();
}

/// What the user is told after the stream was reopened on `new` while `old` was the choice:
/// each of the four settings that differ, as "Driver: WASAPI", "Device: System default",
/// "Sample rate: 48000 Hz" and "Buffer size: 512 frames (10.7 ms)"; `None` when none differs.
/// Said once per change, never for the periodic In use refresh.
pub fn change_text(old: &AudioChoice, new: &AudioChoice, drivers: &[String]) -> Option<String> {
    let mut v: Vec<String> = Vec::new();
    if old.driver != new.driver {
        v.push(format!("Driver: {}", driver_of(drivers, new)));
    }
    if old.device != new.device {
        let d = match (&new.device, &new.device_name) {
            (None, _) => SYSTEM_DEFAULT,
            (Some(_), Some(n)) => n,
            (Some(id), None) => id,
        };
        v.push(format!("Device: {d}"));
    }
    if old.rate != new.rate {
        v.push(match new.rate {
            None => format!("Sample rate: {DEVICE_DEFAULT}"),
            Some(r) => format!("Sample rate: {r} Hz"),
        });
    }
    if old.buffer != new.buffer && !buffer_is_the_drivers(driver_of(drivers, new)) {
        let rate = new.rate.unwrap_or(orb_player::RATE);
        v.push(format!("Buffer size: {}", frames_text(new.buffer, rate)));
    }
    (!v.is_empty()).then(|| v.join(". "))
}

/// The rows of the In use grid: what the stream is on, or why there is no sound.
fn in_use_rows(ui: &mut egui::Ui, sr: bool, in_use: &Option<InUse>, running: bool) {
    match in_use {
        Some(u) => {
            in_use_row(ui, sr, "Device", format!("{}: {}", u.driver, u.device));
            in_use_row(
                ui,
                sr,
                "Sample rate",
                if u.resampled {
                    format!("{} Hz, resampled from 44100 Hz", u.rate)
                } else {
                    format!("{} Hz, not resampled", u.rate)
                },
            );
            in_use_row(
                ui,
                sr,
                "Format",
                format!("{} channels, {}", u.channels, u.format),
            );
            // only the first callback's size: what was asked for is the
            // Buffer size list's selection
            in_use_row(
                ui,
                sr,
                "Buffer",
                match u.granted {
                    Some(g) => frames_text(g, u.rate),
                    None => "no callback yet".to_string(),
                },
            );
            // what the host measured: playback time minus call time
            if let Some(ns) = u.output_latency_ns {
                in_use_row(
                    ui,
                    sr,
                    "Output latency",
                    format!("{:.1} ms (measured by the driver)", ns as f64 / 1e6),
                );
            }
            in_use_row(ui, sr, "Underruns", u.underruns.to_string());
        }
        None => in_use_row(
            ui,
            sr,
            "Output",
            if running {
                "no sound: the stream did not open"
            } else {
                "no audio output"
            },
        ),
    }
}

/// The announcement of a change and the controller's notes of the same apply: one text, the
/// change first (`None` when there is neither).
pub fn join_said(change: Option<String>, notes: &[String]) -> Option<String> {
    let all: Vec<&str> = change
        .iter()
        .map(String::as_str)
        .chain(notes.iter().map(String::as_str))
        .collect();
    (!all.is_empty()).then(|| all.join(". "))
}

/// The id of the window of an open (`App::audio_serial`).
pub fn audio_window_id(serial: u32) -> egui::Id {
    egui::Id::unique(("orb_audio_output_window", serial))
}

impl App {
    /// Options > Audio Output...
    pub fn show_audio(&mut self) {
        if !self.audio_open {
            // a new id: no stored place, so the window opens at its default, the center
            self.audio_serial += 1;
            let choice = self.audio_choice();
            self.audio_ui.refresh(&choice);
            self.audio_ui.refresh_pending = false;
        } else {
            self.raise_window(self.audio_id());
        }
        self.audio_open = true;
    }

    /// The start of the program: the controller and the saved settings are taken, in this order
    /// (the window compares what runs with the saved choice), so that a saved device that was
    /// missing at the start shows as the system default in Options > Audio Output.
    pub fn adopt_audio_and_settings(
        &mut self,
        controller: Option<orb_player::control::Controller>,
        settings: crate::settings::Settings,
    ) {
        self.audio = controller;
        self.settings = settings;
        self.audio_follow_controller();
    }

    /// The id of the window now (for tests).
    pub fn audio_id(&self) -> egui::Id {
        audio_window_id(self.audio_serial)
    }

    /// Opens the window like [`App::show_audio`] but with the given lists, and never asks the
    /// drivers: no device is read (for tests of how the window is read).
    #[doc(hidden)]
    pub fn audio_open_for_test(
        &mut self,
        drivers: Vec<String>,
        devices: Vec<DeviceInfo>,
        caps: Option<Caps>,
    ) {
        self.audio_serial += 1;
        self.audio_ui.drivers = drivers;
        self.audio_ui.devices = devices;
        self.audio_ui.caps = caps;
        self.audio_ui.lists_fixed = true;
        self.audio_open = true;
    }

    /// Whether the lists wait for an open list to close (for tests).
    #[doc(hidden)]
    pub fn audio_refresh_pending_for_test(&self) -> bool {
        self.audio_ui.refresh_pending
    }

    /// Whether the window is open.
    pub fn audio_is_open(&self) -> bool {
        self.audio_open
    }

    /// Applies `choice` at once: the stream reopens on it (with the fallbacks of
    /// [`orb_player::control::Controller::open_chain`]), what the stream ended up on is saved,
    /// and the window's lists follow.
    pub fn audio_choose(&mut self, choice: AudioChoice) {
        // the test seam (`lists_fixed`) never opens a stream and never asks a driver
        let fixed = self.audio_ui.lists_fixed;
        let Some(c) = &mut self.audio else {
            return;
        };
        let (events, now) = if fixed {
            (Vec::new(), choice)
        } else {
            let events = c.apply(choice.request());
            (events, AudioChoice::from_request(c.request()))
        };
        // what the stream ended up on, once (task-234); the controller's notes follow
        let said = change_text(&self.audio_choice(), &now, &self.audio_ui.drivers);
        self.settings.set_audio(now.clone());
        self.audio_ui.in_use_choice = None;
        self.audio_ui.refresh_pending = false;
        if let Err(e) = self.settings.save() {
            self.fail(e);
        }
        self.audio_events_after(said, events);
        if !fixed {
            self.audio_ui.refresh(&now);
        }
    }

    /// Tells the user what the controller reports: a note in the status line, an error in the
    /// box.
    ///
    /// The controller may have fallen back (a lost device opens the system default): the
    /// window's choice then follows what it runs now, so the Device (and a changed rate or
    /// size) shows what is in use. It is said by the controller's own note only, and not saved:
    /// the saved choice stays the user's until they pick in the window. The controller never
    /// goes back to the saved device by itself (its retry opens the system default only), so
    /// the window shows System default until the user picks, or until the next start finds the
    /// saved device again.
    pub fn audio_events(&mut self, events: Vec<Event>) {
        if events.is_empty() {
            return;
        }
        self.audio_events_after(None, events);
        self.audio_follow_controller();
    }

    /// The choice the window shows: what the controller runs after a fallback, else the saved
    /// choice.
    pub fn audio_choice(&self) -> AudioChoice {
        self.audio_ui
            .in_use_choice
            .clone()
            .unwrap_or_else(|| self.settings.audio.clone())
    }

    /// Makes the window show what the controller runs, silently, when that is not the saved
    /// choice (a fallback). The saved choice is not touched, so no save writes the fallback to
    /// the file. The lists are read again when no list is open. Called at the start (through
    /// [`App::adopt_audio_and_settings`]) and after the controller's events.
    pub fn audio_follow_controller(&mut self) {
        let Some(c) = &self.audio else {
            return;
        };
        let now = AudioChoice::from_request(c.request());
        let before = self.audio_choice();
        self.audio_ui.in_use_choice = (now != self.settings.audio).then_some(now.clone());
        if now != before && self.audio_open {
            self.audio_ui.refresh_pending = true;
        }
    }

    /// [`App::audio_events`] after a change said as `change`: the notes go to the status line
    /// and, with the change first, are announced as one utterance (a second message would
    /// replace the first).
    fn audio_events_after(&mut self, change: Option<String>, events: Vec<Event>) {
        let mut notes = Vec::new();
        for e in events {
            match e {
                Event::Note(n) => {
                    self.status = Some(n.clone());
                    notes.push(n);
                }
                Event::Error(e) => self.fail(e),
            }
        }
        if let Some(t) = join_said(change, &notes) {
            self.announce(t);
        }
    }

    /// Draws the window when it is open. Escape and the close button close it.
    pub fn audio_ui(&mut self, ctx: &Context) {
        if !self.audio_open {
            return;
        }
        let sr = self.screen_reader_mode();
        // a fallback changed the chosen values: the lists follow once no list is open
        if self.audio_ui.refresh_pending && !egui::Popup::is_any_open(ctx) {
            self.audio_ui.refresh_pending = false;
            if !self.audio_ui.lists_fixed {
                let c = self.audio_choice();
                self.audio_ui.refresh_devices(&c);
            }
        }
        // in screen reader mode Escape closes an open list first (the keyboard's way out of a
        // combo box), the window on the next press
        let list_open = sr && egui::Popup::is_any_open(ctx);
        if !list_open && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            self.audio_open = false;
            return;
        }
        let due = self.window_focus_due(Win::Audio);
        let id = audio_window_id(self.audio_serial);
        let mut open = true;
        let mut chosen: Option<AudioChoice> = None;
        // which lists to read again after this frame: the Device list opened, or a list of the
        // device's rates or sizes did
        let mut refresh_devices = false;
        let mut refresh_caps = false;
        // the rate the latencies are of: the stream's, else the one asked for
        let live = self.audio.as_ref().and_then(|c| c.in_use());
        let in_use = self.audio_ui.sample(Instant::now(), live);
        let latency_rate = in_use.as_ref().map(|u| u.rate);
        let choice = self.audio_choice();
        let mut nav = std::mem::take(&mut self.audio_ui.nav);
        let ui_state = &self.audio_ui;
        let running = self.audio.is_some();
        let screen = ctx.content_rect();
        let shown_buffer = ui_state.requested_buffer(choice.buffer);
        let rate = latency_rate.or(choice.rate).unwrap_or(orb_player::RATE);
        egui::Window::new("Audio Output")
            .id(audio_window_id(self.audio_serial))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(screen.center())
            // inside the window with the dialogs' margin whatever the device names and the
            // "In use" rows need (task-297): the rest scrolls
            .max_width(screen.width() - 2.0 * DIALOG_MARGIN)
            .show(ctx, |ui| {
                focus_into_window(ui, due);
                win::window(ui, sr, id, "Audio Output");
                egui::ScrollArea::vertical()
                    .auto_shrink([true, true])
                    .max_height((screen.height() - 2.0 * DIALOG_MARGIN - 50.0).max(60.0))
                    .show(ui, |ui| {
                        egui::Grid::new("audio_settings")
                            .num_columns(2)
                            .spacing([12.0, 8.0])
                            .show(ui, |ui| {
                                // Driver
                                let driver = driver_of(&ui_state.drivers, &choice).to_string();
                                ui.label("Driver");
                                let spec = ComboSpec {
                                    salt: "audio_driver",
                                    name: "Driver",
                                    text: driver.clone(),
                                    labels: ui_state.drivers.clone(),
                                    selected: ui_state.drivers.iter().position(|d| *d == driver),
                                };
                                let out = audio_combo(ui, sr, &mut nav, &spec, &|i| {
                                    Some(after_driver(&ui_state.drivers, &ui_state.drivers[i]))
                                });
                                chosen = out.chosen.or(chosen.take());
                                ui.end_row();
                                // Device
                                ui.label("Device");
                                let missing = choice.device.as_ref().is_some_and(|id| {
                                    !ui_state.devices.iter().any(|d| d.id == *id)
                                });
                                let shown = match (&choice.device, &choice.device_name) {
                                    (None, _) => SYSTEM_DEFAULT.to_string(),
                                    (Some(_), Some(n)) if missing => format!("{n} (not available)"),
                                    (Some(id), n) => ui_state
                                        .devices
                                        .iter()
                                        .find(|d| d.id == *id)
                                        .map(|d| d.name.clone())
                                        .or_else(|| n.clone())
                                        .unwrap_or_else(|| id.clone()),
                                };
                                // entry 0 is the system default, then the devices
                                let mut labels = vec![SYSTEM_DEFAULT.to_string()];
                                labels.extend(ui_state.devices.iter().map(|d| d.name.clone()));
                                let selected = match &choice.device {
                                    None => Some(0),
                                    Some(id) => ui_state
                                        .devices
                                        .iter()
                                        .position(|d| d.id == *id)
                                        .map(|p| p + 1),
                                };
                                let spec = ComboSpec {
                                    salt: "audio_device",
                                    name: "Device",
                                    text: shown,
                                    labels,
                                    selected,
                                };
                                let out = audio_combo(ui, sr, &mut nav, &spec, &|i| {
                                    let dev = i.checked_sub(1).map(|p| &ui_state.devices[p]);
                                    // the test seam never asks a driver
                                    let caps = if ui_state.lists_fixed {
                                        ui_state.caps.clone()
                                    } else {
                                        devices::caps(
                                            choice.driver.as_deref(),
                                            dev.map(|d| d.id.as_str()),
                                        )
                                        .ok()
                                    };
                                    Some(after_device(&choice, dev, caps.as_ref()))
                                });
                                chosen = out.chosen.or(chosen.take());
                                refresh_devices |= out.opened;
                                ui.end_row();
                                // Sample rate
                                ui.label("Sample rate");
                                let rate_text = match choice.rate {
                                    None => DEVICE_DEFAULT.to_string(),
                                    Some(r) => format!("{r} Hz"),
                                };
                                // entry 0 is the device's default, then its rates
                                let mut labels = vec![DEVICE_DEFAULT.to_string()];
                                labels.extend(ui_state.rates().iter().map(|r| format!("{r} Hz")));
                                let selected = match choice.rate {
                                    None => Some(0),
                                    Some(r) => {
                                        ui_state.rates().iter().position(|x| *x == r).map(|p| p + 1)
                                    }
                                };
                                let spec = ComboSpec {
                                    salt: "audio_rate",
                                    name: "Sample rate",
                                    text: rate_text,
                                    labels,
                                    selected,
                                };
                                let out = audio_combo(ui, sr, &mut nav, &spec, &|i| {
                                    Some(AudioChoice {
                                        rate: i.checked_sub(1).map(|p| ui_state.rates()[p]),
                                        ..choice.clone()
                                    })
                                });
                                chosen = out.chosen.or(chosen.take());
                                refresh_caps |= out.opened;
                                ui.end_row();
                                // Buffer size
                                ui.label("Buffer size");
                                if buffer_is_the_drivers(&driver) {
                                    // task-193: ASIO sets the buffer in its own panel
                                    win::line(
                                        ui,
                                        sr,
                                        RichText::new("Set in the driver's own control panel"),
                                        false,
                                        Some("Buffer size: set in the driver's own control panel"),
                                    );
                                } else {
                                    let sizes = ui_state.buffers();
                                    let spec = ComboSpec {
                                        salt: "audio_buffer",
                                        name: "Buffer size",
                                        text: frames_text(shown_buffer, rate),
                                        labels: sizes
                                            .iter()
                                            .map(|&n| frames_text(n, rate))
                                            .collect(),
                                        selected: sizes.iter().position(|&n| n == shown_buffer),
                                    };
                                    let out = audio_combo(ui, sr, &mut nav, &spec, &|i| {
                                        Some(AudioChoice {
                                            buffer: sizes[i],
                                            ..choice.clone()
                                        })
                                    });
                                    chosen = out.chosen.or(chosen.take());
                                    refresh_caps |= out.opened;
                                }
                                ui.end_row();
                            });
                        if let Some(p) = &ui_state.problem {
                            ui.add_space(4.0);
                            win::line(ui, sr, RichText::new(p).color(INK_DIM), true, None);
                        }
                        ui.add_space(6.0);
                        ui.separator();
                        ui.label(RichText::new("In use").strong());
                        ui.add_space(2.0);
                        egui::Grid::new("audio_in_use")
                            .num_columns(2)
                            .spacing([12.0, 4.0])
                            .show(ui, |ui| in_use_rows(ui, sr, &in_use, running));
                        // the underruns count up while the window is open
                        ctx.request_repaint_after(Duration::from_millis(500));
                    });
            });
        self.audio_ui.nav = nav;
        if !open {
            self.audio_open = false;
        }
        // (the test seam's lists are fixed: no driver is asked)
        if self.audio_ui.lists_fixed {
            refresh_devices = false;
            refresh_caps = false;
        }
        if refresh_devices {
            let c = self.audio_choice();
            self.audio_ui.refresh_devices(&c);
        } else if refresh_caps {
            let c = self.audio_choice();
            self.audio_ui.refresh_caps(&c);
        }
        if let Some(c) = chosen {
            self.audio_choose(c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn caps() -> Caps {
        Caps {
            rates: vec![44100, 48000],
            buffers: vec![256, 512, 1024],
            range: devices::SupportedBufferSize::Range {
                min: 256,
                max: 1024,
            },
        }
    }

    fn dev(id: &str) -> DeviceInfo {
        DeviceInfo {
            id: id.into(),
            name: format!("Name {id}"),
        }
    }

    #[test]
    fn the_first_driver_is_stored_as_none() {
        let drivers = vec!["WASAPI".to_string(), "ASIO".to_string()];
        assert_eq!(after_driver(&drivers, "WASAPI").driver, None);
        let c = after_driver(&drivers, "ASIO");
        assert_eq!(c.driver.as_deref(), Some("ASIO"));
        // a new driver starts over
        assert_eq!(
            AudioChoice {
                driver: c.driver.clone(),
                ..AudioChoice::default()
            },
            c
        );
        assert_eq!(driver_of(&drivers, &AudioChoice::default()), "WASAPI");
    }

    #[test]
    fn a_device_keeps_the_rate_and_moves_the_size_to_what_it_offers() {
        let old = AudioChoice {
            rate: Some(48000),
            buffer: 2048,
            ..AudioChoice::default()
        };
        let c = after_device(&old, Some(&dev("a")), Some(&caps()));
        assert_eq!(c.device.as_deref(), Some("a"));
        assert_eq!(c.device_name.as_deref(), Some("Name a"));
        assert_eq!(c.rate, Some(48000));
        assert_eq!(c.buffer, 1024);
        // a rate the device does not offer is dropped
        let old = AudioChoice {
            rate: Some(96000),
            ..old
        };
        assert_eq!(after_device(&old, None, Some(&caps())).rate, None);
        // System default clears the device
        let named = after_device(&old, Some(&dev("a")), None);
        let back = after_device(&named, None, None);
        assert_eq!((back.device, back.device_name), (None, None));
    }

    #[test]
    fn a_size_shows_its_time() {
        assert_eq!(frames_text(2048, 44100), "2048 frames (46.4 ms)");
        assert_eq!(frames_text(512, 48000), "512 frames (10.7 ms)");
    }

    fn reading(latency_ms: u64, underruns: u64) -> InUse {
        InUse {
            driver: "WASAPI".into(),
            device: "Speakers".into(),
            follows_default: true,
            rate: 44100,
            resampled: false,
            channels: 2,
            format: "f32".into(),
            requested: Some(2048),
            granted: Some(2048),
            output_latency_ns: Some(latency_ms * 1_000_000),
            underruns,
        }
    }

    #[test]
    fn the_numbers_are_sampled_on_a_timer_and_not_on_every_repaint() {
        let t0 = Instant::now();
        let ms = Duration::from_millis;
        let mut ui = AudioUi::default();
        // the first reading is taken at once
        assert_eq!(ui.sample(t0, Some(reading(40, 0))), Some(reading(40, 0)));
        // repaints before the period show the sampled copy, whatever the live numbers are
        for k in [1, 50, 120, 499] {
            assert_eq!(
                ui.sample(t0 + ms(k), Some(reading(99, 7))),
                Some(reading(40, 0))
            );
        }
        // the period passed: a new sample, which is then held for the next period
        assert_eq!(
            ui.sample(t0 + ms(500), Some(reading(60, 1))),
            Some(reading(60, 1))
        );
        assert_eq!(
            ui.sample(t0 + ms(700), Some(reading(70, 2))),
            Some(reading(60, 1))
        );
        assert_eq!(
            ui.sample(t0 + ms(1000), Some(reading(70, 2))),
            Some(reading(70, 2))
        );
    }

    #[test]
    fn a_new_stream_shows_at_once_and_no_stream_shows_nothing() {
        let t0 = Instant::now();
        let ms = Duration::from_millis;
        let mut ui = AudioUi::default();
        ui.sample(t0, Some(reading(40, 0)));
        // the stream was reopened on another device inside the period
        let mut other = reading(10, 0);
        other.device = "Headphones".into();
        assert_eq!(ui.sample(t0 + ms(5), Some(other.clone())), Some(other));
        assert_eq!(ui.sample(t0 + ms(6), None), None);
        assert_eq!(
            ui.sample(t0 + ms(7), Some(reading(1, 0))),
            Some(reading(1, 0))
        );
    }

    #[test]
    fn only_asio_sets_its_own_buffer() {
        assert!(buffer_is_the_drivers("ASIO"));
        for d in ["CoreAudio", "WASAPI", "ALSA"] {
            assert!(!buffer_is_the_drivers(d));
        }
    }
}

#[cfg(test)]
mod a11y_tests {
    use super::*;

    fn reading() -> InUse {
        InUse {
            driver: "WASAPI".into(),
            device: "Speakers".into(),
            follows_default: true,
            rate: 48000,
            resampled: true,
            channels: 2,
            format: "f32".into(),
            requested: Some(2048),
            granted: Some(2048),
            output_latency_ns: Some(21_300_000),
            underruns: 3,
        }
    }

    /// The In use rows in screen reader mode are lines that take the focus, read as
    /// "label: text"; without the mode they are plain labels that cannot be focused.
    #[test]
    fn the_in_use_rows_are_lines_a_screen_reader_reads() {
        for sr in [true, false] {
            let ctx = Context::default();
            ctx.enable_accesskit();
            let mut out = Default::default();
            // the grid's first frame is a sizing pass that registers no widgets
            for t in [0.0, 0.1, 0.2] {
                let input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(600.0, 400.0),
                    )),
                    time: Some(t),
                    focused: true,
                    ..Default::default()
                };
                out = ctx.run_ui(input, |ui| {
                    egui::Grid::new("g").show(ui, |ui| {
                        in_use_rows(ui, sr, &Some(reading()), true);
                    });
                });
            }
            let update = out.platform_output.accesskit_update.take().unwrap();
            let values: Vec<String> = update
                .nodes
                .iter()
                .filter_map(|(_, n)| n.value().map(str::to_string))
                .collect();
            let want = [
                "Device: WASAPI: Speakers",
                "Sample rate: 48000 Hz, resampled from 44100 Hz",
                "Format: 2 channels, f32",
                "Buffer: 2048 frames (42.7 ms)",
                "Output latency: 21.3 ms (measured by the driver)",
                "Underruns: 3",
            ];
            if sr {
                for w in want {
                    assert!(values.iter().any(|v| v == w), "{w}: {values:?}");
                }
            } else {
                // only egui's own text of each label, no "label: text" line
                for w in want {
                    assert!(!values.iter().any(|v| v == w), "{w}: {values:?}");
                }
            }
        }
    }

    fn dev_info(id: &str) -> DeviceInfo {
        DeviceInfo {
            id: id.into(),
            name: format!("Name {id}"),
        }
    }

    #[test]
    fn a_change_and_the_notes_are_one_text_change_first() {
        let note = "Audio device X is not available; using System default".to_string();
        assert_eq!(join_said(None, &[]), None);
        assert_eq!(
            join_said(Some("Device: X".into()), &[]).as_deref(),
            Some("Device: X")
        );
        assert_eq!(
            join_said(None, std::slice::from_ref(&note)).as_deref(),
            Some(note.as_str())
        );
        assert_eq!(
            join_said(Some("Device: X".into()), &[note.clone(), "More".into()]).as_deref(),
            Some("Device: X. Audio device X is not available; using System default. More")
        );
    }

    #[test]
    fn a_change_is_said_with_what_changed() {
        let drivers = vec!["WASAPI".to_string(), "ASIO".to_string()];
        let old = AudioChoice::default();
        assert_eq!(change_text(&old, &old, &drivers), None);
        let asio = after_driver(&drivers, "ASIO");
        assert_eq!(
            change_text(&old, &asio, &drivers).as_deref(),
            Some("Driver: ASIO")
        );
        let dev = after_device(&old, Some(&dev_info("a")), None);
        assert_eq!(
            change_text(&old, &dev, &drivers).as_deref(),
            Some("Device: Name a")
        );
        assert_eq!(
            change_text(&dev, &old, &drivers).as_deref(),
            Some("Device: System default")
        );
        let rate = AudioChoice {
            rate: Some(48000),
            ..old.clone()
        };
        assert_eq!(
            change_text(&old, &rate, &drivers).as_deref(),
            Some("Sample rate: 48000 Hz")
        );
        assert_eq!(
            change_text(&rate, &old, &drivers).as_deref(),
            Some("Sample rate: Device default")
        );
        let buf = AudioChoice {
            buffer: 512,
            ..old.clone()
        };
        assert_eq!(
            change_text(&old, &buf, &drivers).as_deref(),
            Some("Buffer size: 512 frames (11.6 ms)")
        );
        // ASIO's buffer is the driver's own: not said as a choice
        let asio_buf = AudioChoice {
            buffer: 512,
            ..asio.clone()
        };
        assert_eq!(change_text(&asio, &asio_buf, &drivers), None);
    }
}
