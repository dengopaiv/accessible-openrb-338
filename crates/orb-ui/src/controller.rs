//! The one controller of the panel (task-266, docs/ui_spec.md section 22): all the input of
//! the main window's panel, run on whatever layout the current skin gives (`skin::Layout`, in
//! the skin's units), with the original's interaction (docs/ui_spec.md section 2): the hit
//! tests and gestures of every control kind, knob and fader drags (Shift: fine), the fader's
//! jump, the repeat and wind buttons, the drum machines' step strips and instrument selection,
//! the rows' focus, the logo, the cursor and the computer keyboard. A skin has no input code.

use std::time::Duration;

use egui::{Key, Pos2, Response, Sense, Ui};

use crate::a11y::{announce, drive, nav, step303};
use crate::app::{App, View, Win};
use crate::behavior;
use crate::keys::{self, Action, SrAction};
use crate::numpad;
use crate::skin::{Ctl, Layout};

/// Per-control gesture state, kept in egui's memory.
#[derive(Clone, Default)]
struct Gesture {
    down: bool,
    p0: i32,
    y0: f32,
    fine: bool,
    t0: f64,
    taken: u32,
    new_value: u8,
    last_step: i32,
}

/// A computer key that is down: it acts as a mouse press on its button.
pub(crate) struct Hold {
    /// The physical key (`keys::position`).
    key: Key,
    /// The layout id of the button shown pressed.
    id: Option<String>,
    /// A repeat or wind button: its id, when the key went down, the steps taken.
    arrow: Option<(&'static str, f64, u32)>,
    /// What the announcer was told after the key's press (an arrow held longer announces its
    /// final value at the release; task-189).
    seen: Option<announce::KeyState>,
}

/// Where the pointer is, in skin units.
fn pointer(ui: &Ui, view: &View) -> Option<Pos2> {
    ui.input(|i| i.pointer.latest_pos())
        .map(|p| view.to_panel(p))
}

fn press_origin(ui: &Ui, view: &View) -> Option<Pos2> {
    ui.input(|i| i.pointer.press_origin().or(i.pointer.latest_pos()))
        .map(|p| view.to_panel(p))
}

/// A control of the layout: its row and its place in the layout (`n`). The place is part of
/// its egui ids, so that a skin may show one control twice; the kind is too: the 303's accent
/// knob and accent flag button share the id `tb303_N.accent`.
#[derive(Clone, Copy)]
struct Placed<'a> {
    c: &'a Ctl,
    n: usize,
    /// The first place of its kind and id in the layout: its hit area has the id the fork's
    /// screen reader tree and focus order know it by ([`crate::a11y::hit_id`]); a second
    /// place of the same control is keyed by its place too.
    first: bool,
}

/// The egui id of a placed control's hit area.
fn hit_id(p: Placed) -> egui::Id {
    if p.first {
        crate::a11y::hit_id(p.c)
    } else {
        egui::Id::unique(("hit", &p.c.kind, &p.c.id, p.n))
    }
}

fn gesture(ui: &Ui, p: Placed) -> (egui::Id, Gesture) {
    let id = egui::Id::unique(("gesture", &p.c.kind, &p.c.id, p.n));
    (id, ui.data(|d| d.get_temp(id)).unwrap_or_default())
}

fn hit(ui: &mut Ui, p: Placed, view: &View) -> Response {
    let c = p.c;
    ui.interact(
        view.rect(c.x, c.y, c.w, c.h),
        hit_id(p),
        Sense::click_and_drag(),
    )
}

/// True on the frame the mouse goes down on the control (the action runs at mouse down).
fn first_press(ui: &Ui, p: Placed, r: &Response) -> bool {
    let (id, mut g) = gesture(ui, p);
    let down = r.is_pointer_button_down_on();
    let first = down && !g.down;
    g.down = down;
    ui.data_mut(|d| d.insert_temp(id, g));
    first
}

impl App {
    /// The computer keyboard (`keys`): a key press acts like a mouse press on its button, a
    /// held repeat button keeps repeating, and the button shows pressed while the key is down.
    /// Auto-repeat events and keys with Ctrl or Cmd (the menu's shortcuts) are ignored, as the
    /// original's handlers do. The keys act on the model's controls by id, whatever the skin
    /// shows. The fork's screen reader mode (m-7) has its own key map and keys before it.
    pub(crate) fn keyboard(&mut self, ui: &Ui) {
        let (events, now, focused) = ui.input(|i| (i.events.clone(), i.time, i.focused));
        if !focused {
            self.holds.clear();
        }
        let screen_reader = self.screen_reader_mode();
        // Screen reader mode, the focus on the panel or nowhere: Tab, Shift+Tab and the arrows
        // do not move egui's focus (it moves in the panel's order below, and the arrows change
        // values). Called before any widget of this frame says it takes the focus, which is
        // where egui would move it. A window's focus keeps egui's Tab.
        let on_panel = ui
            .memory(|m| m.focused())
            .is_none_or(|f| nav::section_of(&self.nav, f).is_some());
        // In a window only the arrows' moves are cancelled (they would leave the window for
        // the panel by position); Tab stays egui's.
        let arrow = || {
            events.iter().any(|e| {
                matches!(e, egui::Event::Key { key, pressed: true, .. }
                    if matches!(key, Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft
                        | Key::ArrowRight))
            })
        };
        let tab = || {
            events.iter().any(|e| {
                matches!(
                    e,
                    egui::Event::Key {
                        key: Key::Tab,
                        pressed: true,
                        ..
                    }
                )
            })
        };
        let panel_control = ui
            .memory(|m| m.focused())
            .is_some_and(|f| nav::section_of(&self.nav, f).is_some());
        let window_has_keys = !panel_control && Win::ALL.iter().any(|&w| self.win_open(w));
        if screen_reader && (on_panel || (arrow() && !tab())) {
            ui.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
        }
        self.key_focus = ui.memory(|m| m.focused());
        // Numpad `*` has no egui key: it is Record, as Insert is (a press, and the release that
        // lets go of the button), numpad.rs (task-225). A window that has the keys keeps it.
        for pressed in numpad::take_multiply() {
            if !pressed {
                self.release(Key::Insert);
            } else if focused && !(screen_reader && window_has_keys) {
                let a = Action::Press("transport.record".into());
                self.act(Key::Insert, a, now);
            }
        }
        for ev in events {
            let egui::Event::Key {
                key,
                physical_key,
                pressed,
                repeat,
                modifiers,
            } = ev
            else {
                continue;
            };
            let k = keys::position(key, physical_key);
            // whether the numpad's key (the hook of numpad.rs; always false off Windows)
            let numpad = numpad::claim(k, physical_key.is_some(), pressed);
            if !pressed {
                self.release(k);
                continue;
            }
            // Screen reader mode, the focus on a 303's step editor: Up, Down, Home and End move
            // its edit step, auto-repeat included; Left and Right do nothing (task-190).
            if !numpad && !modifiers.any() && screen_reader && self.step_editor_key(ui, key, now) {
                continue;
            }
            // Screen reader mode, the focus on a drum grid: the arrows, Home, End and Space
            // (with Ctrl and Alt) are the grid's (task-191), before the value keys and the key
            // map.
            if screen_reader && self.grid_key(ui, key, modifiers, repeat) {
                continue;
            }
            // Screen reader mode: Up, Down, Page Up, Page Down, Home and End change the focused
            // slider, spin button or pattern selector, auto-repeat included, and do not reach
            // the original's keys (task-187).
            if !modifiers.any()
                && screen_reader
                && let Some(step) = drive::Step::of_key(key)
                && let Some(t) = self.focused_target(ui)
            {
                let speech = self.speech();
                drive::step(&mut self.panel, &t, step, speech, now);
                continue;
            }
            if screen_reader {
                // The key map of the mode (task-188, decision A11Y-D6). With the focus in a
                // window (Keyboard Shortcuts, Info, About, Mods > Info), the panel's keys wait,
                // as they do for the menus (TASK-202): Enter, Space, the arrows and the letters
                // are the window's; the mode's function keys still work (F2 to F4 are
                // task-189's, F10 the menu bar's, Escape the window's). Also with the focus
                // nowhere while a window is open (before the window takes it, or after a click
                // on its text).
                // F2, F3 and F4 read the pattern, the transport, the levels (task-189), with a
                // window open too (F2 then reads the row focus's pattern).
                if !repeat && !modifiers.any() && self.readout_key(ui, key) {
                    continue;
                }
                if window_has_keys
                    && !matches!(
                        k,
                        Key::F1
                            | Key::F2
                            | Key::F3
                            | Key::F4
                            | Key::F5
                            | Key::F6
                            | Key::F9
                            | Key::F11
                            | Key::F12
                    )
                {
                    continue;
                }
                let ctrl = modifiers.command || modifiers.ctrl;
                // Enter follows the section of the focused control, not the row focus (the
                // user's decision, task-207): in a 303's section that 303's Step (Program
                // Synth mode), in any other section Play. With no panel control focused, the
                // row focus decides, as with the mode off.
                let section = ui
                    .memory(|m| m.focused())
                    .and_then(|f| nav::section_of(&self.nav, f));
                // The numpad's Enter is Play whatever has the focus (task-225).
                let enter = k == Key::Enter && !ctrl && !numpad;
                let mapped = match (enter, section) {
                    (true, Some(s)) => match nav::row_of_section(s) {
                        Some(r) if r < 2 => keys::map_screen_reader(
                            k,
                            modifiers.shift,
                            ctrl,
                            false,
                            self.key_mode,
                            r,
                        ),
                        _ => Some(SrAction::Panel(Action::Press("transport.play".into()))),
                    },
                    _ => keys::map_screen_reader(
                        k,
                        modifiers.shift,
                        ctrl,
                        numpad,
                        self.key_mode,
                        self.panel.focus,
                    ),
                };
                let Some(a) = mapped else {
                    continue;
                };
                match a {
                    // moves of the focus repeat as in any program
                    SrAction::Control(back) if on_panel => {
                        let f = ui.memory(|m| m.focused());
                        self.move_nav(ui, nav::next_control(&self.nav, f, back));
                    }
                    SrAction::Control(_) => {}
                    SrAction::Section(back) => {
                        let f = ui.memory(|m| m.focused());
                        self.move_nav(ui, nav::next_section(&self.nav, f, back));
                    }
                    // the original ignores auto-repeat for its keys; that stays
                    _ if repeat => {}
                    SrAction::Activate => self.activate(ui, now),
                    SrAction::Help => self.show_key_help(),
                    SrAction::Panel(a) => self.act(k, a, now),
                }
                continue;
            }
            if repeat || modifiers.command || modifiers.ctrl {
                continue;
            }
            if let Some(a) = keys::map(k, modifiers.shift, numpad, self.key_mode, self.panel.focus)
            {
                self.act(k, a, now);
            }
        }
        // The held repeat buttons: the held-button schedule of the mouse.
        let mut holds = std::mem::take(&mut self.holds);
        for h in &mut holds {
            if let Some((id, t0, taken)) = &mut h.arrow {
                let total = 1 + behavior::repeat_extra_steps(((now - *t0) * 1000.0) as u32);
                while *taken < total {
                    self.panel.arrow(id);
                    *taken += 1;
                }
                ui.ctx().request_repaint_after(Duration::from_millis(16));
            }
        }
        self.holds = holds;
    }

    /// The release of the key `k`: its held button is let go. A repeat or wind button that
    /// stepped more than once announces where it ended (task-189).
    fn release(&mut self, k: Key) {
        let (released, held): (Vec<Hold>, Vec<Hold>) = std::mem::take(&mut self.holds)
            .into_iter()
            .partition(|h| h.key == k);
        self.holds = held;
        for h in released {
            if let (Some(seen), Some((id, _, taken))) = (h.seen, h.arrow)
                && taken > 1
            {
                let after = seen.with_arrow_from(id, &announce::KeyState::of(&self.panel));
                if let Some(text) = announce::key_effect(&seen, &after, self.key_focus) {
                    self.announce(text);
                }
            }
        }
    }

    /// What a key of the original's map does (`keys::map`). What it changed is announced in
    /// screen reader mode when the focus is not on the control that shows it (task-189).
    fn act(&mut self, k: Key, action: Action, now: f64) {
        // (a press without the release before it, e.g. after the window lost the focus)
        self.holds.retain(|h| h.key != k);
        let before = announce::KeyState::of(&self.panel);
        let before303 = step303::Before::of(&self.panel);
        let pressed = match &action {
            Action::Press(id) => Some(id.clone()),
            _ => None,
        };
        let mut hold = Hold {
            key: k,
            id: None,
            arrow: None,
            seen: None,
        };
        match action {
            Action::PlayStop => {
                let id = if self.panel.transport.playing {
                    "transport.stop"
                } else {
                    "transport.play"
                };
                self.panel.press(id, now);
                hold.id = Some(id.into());
            }
            Action::Press(id) => {
                self.panel.press(&id, now);
                hold.id = Some(id);
            }
            Action::Arrow(id) => {
                self.panel.arrow(id);
                hold.id = Some(id.into());
                hold.arrow = Some((id, now, 1));
            }
            Action::Locate(end) => self.panel.locate_loop(end),
            // Up from row 1 goes to row 4 and Down from row 4 to row 1 (`sub_42D1D3`).
            Action::Focus(d) => {
                self.panel.focus = (self.panel.focus as i32 + d).rem_euclid(4) as usize
            }
            Action::Tap303(rest) => self.panel.tap_303(self.panel.focus, rest),
            Action::TapDrum(i) => self.panel.tap_drum(self.panel.focus == 2, i),
        }
        self.announce_key_effect(&before);
        if let Some(id) = pressed {
            self.announce_303(&before303, &id, self.key_focus);
        }
        if hold.arrow.is_some() {
            hold.seen = Some(announce::KeyState::of(&self.panel));
        }
        self.holds.push(hold);
    }

    /// Whether a computer key that is down holds the button `id` pressed.
    pub fn key_held(&self, id: &str) -> bool {
        self.holds.iter().any(|h| h.id.as_deref() == Some(id))
    }

    /// A click anywhere inside a row gives the row the focus (`sub_42ADBB`); a click on the
    /// logo plays the mod's startup sound.
    pub(crate) fn focus_click(&mut self, ui: &Ui, view: &View, layout: &Layout) {
        let pressed = ui.input(|i| {
            i.pointer
                .button_pressed(egui::PointerButton::Primary)
                .then(|| i.pointer.press_origin())
                .flatten()
        });
        if let Some(p) = pressed {
            // A press on a window, a menu or any other layer over the panel is not the
            // panel's (task-235): a window's close button over the logo played the startup
            // sound, and a click on an Edit menu item moved the focus to the row under it.
            if ui.ctx().layer_id_at(p).is_some_and(|l| l != ui.layer_id()) {
                return;
            }
            let p = view.to_panel(p);
            // a press on the panel takes the focus from the pattern editors (task-290)
            if p.x >= 0.0 && p.y >= 0.0 && p.x < layout.size.x && p.y < layout.size.y {
                self.editor_focus = None;
            }
            if let Some(r) = layout.row_at(p) {
                self.panel.focus = r;
            }
            // the logo (`sub_42A799`) plays the mod's startup sound, whatever the preference
            // (it sends msg 58 without testing it, 0x42A7D3..0x42A7F1)
            if layout.on_logo(p)
                && let Some(link) = &mut self.link
            {
                link.startup_sound();
            }
        }
    }

    /// The pointer's input on the panel's controls, step strips and instrument hit areas of
    /// `layout` at `view`, in the layout's order (which is the order egui hit-tests them in).
    /// Returns, per control of the layout, whether it shows pressed (the mouse or a computer
    /// key holds it). The plugin's hidden controls take nothing.
    ///
    /// The fork's screen reader mode (m-7): each section's controls are in a child `Ui` whose
    /// node is a Group named as the section (task-188), in the layout's order as before; the
    /// drum machines' grids follow in their sections (task-191).
    pub(crate) fn panel_input(&mut self, ui: &mut Ui, view: &View, layout: &Layout) -> Vec<bool> {
        let now = ui.input(|i| i.time);
        let mut groups: Vec<Ui> = if self.screen_reader_mode() {
            let r = ui.max_rect();
            (0..nav::SECTIONS.len())
                .map(|s| ui.new_child(nav::group(s, r)))
                .collect()
        } else {
            Vec::new()
        };
        let mut pressed = vec![false; layout.controls.len()];
        let mut placed = std::collections::HashSet::new();
        for (n, c) in layout.controls.iter().enumerate() {
            let first = placed.insert((c.kind.as_str(), c.id.as_str()));
            if self.hidden(&c.id) {
                continue;
            }
            let target = match nav::section_index(&c.section).and_then(|s| groups.get_mut(s)) {
                Some(g) => g,
                None => &mut *ui,
            };
            pressed[n] = self.control(target, view, Placed { c, n, first }, now);
        }
        for (is808, s) in [(true, 3), (false, 4)] {
            if let Some(g) = groups.get_mut(s) {
                self.drum_grid(g, view, &layout.controls, is808);
            }
        }
        drop(groups);
        self.drum_strips(ui, view, layout);
        self.instrument_selection(ui, view, layout);
        pressed
    }

    /// One control's input; whether it shows pressed. In the fork, also the control's node
    /// for a screen reader and its requests (m-7, `a11y`).
    fn control(&mut self, ui: &mut Ui, view: &View, p: Placed, now: f64) -> bool {
        let c = p.c;
        match c.kind.as_str() {
            // a display takes no click; for a screen reader it is a spin button, a 303's step
            // editor or a read-only value (task-187, task-190)
            "display" => {
                let rect = view.rect(c.x, c.y, c.w, c.h);
                if drive::is_spin(&c.id) {
                    self.spin_button(ui, c, rect, now);
                } else if let Some(k) = step303::editor_of_layout(&c.id)
                    && self.screen_reader_mode()
                {
                    // the 303's step editor (task-190): focusable, no click of its own
                    ui.interact(rect, hit_id(p), Sense::FOCUSABLE);
                    step303::describe(ui.ctx(), &self.panel, k);
                } else {
                    if self.screen_reader_mode() {
                        // a hover-only widget, so that the label's node is in the section's
                        // group (the parent of a node is the Ui of its widget)
                        ui.interact(rect, hit_id(p), Sense::hover());
                    }
                    crate::a11y::describe_display(ui.ctx(), rect, &self.panel, c);
                }
                false
            }
            // display only; the 808's mod window takes no input of its own (its press gives the
            // row the focus, as any press in a row does)
            "area" | "steps" | "led" | "meter" | "modview" => false,
            "knob" => {
                self.knob(ui, view, p);
                false
            }
            "fader" => {
                self.fader(ui, view, p);
                false
            }
            kind => {
                if kind == "grid" && c.name() == "pattern1" {
                    // the row's selector, before its buttons: the Tab order is the panel's
                    self.pattern_selector(ui, view, c, now);
                }
                let joined = drive::joined(c);
                let r = if joined {
                    // part of a spin button or a pattern selector: the mouse works as with
                    // `Sense::click_and_drag`, but no focus and no node (task-187)
                    ui.interact(
                        view.rect(c.x, c.y, c.w, c.h),
                        hit_id(p),
                        Sense::CLICK | Sense::DRAG,
                    )
                } else {
                    hit(ui, p, view)
                };
                let down = r.is_pointer_button_down_on() || self.key_held(&c.id);
                match kind {
                    "repeat" | "wind" => self.repeat(ui, p, &r),
                    "switch" | "toggle" | "radio" | "trigger" | "grid" => {
                        if first_press(ui, p, &r) {
                            self.panel.press(&c.id, now);
                        }
                        // a plugin's pattern drag-out (task-203); nothing in the standalone app
                        if kind == "grid" {
                            self.drag_out(ui, c, &r);
                        }
                    }
                    // A click anywhere on a two-position switch flips the position it shows.
                    "hswitch" | "vswitch" if first_press(ui, p, &r) => {
                        let v = 1 - self.panel.switch_position(&c.id);
                        self.panel.set_switch(&c.id, v);
                    }
                    _ => {}
                }
                if !joined {
                    // A screen reader's Click (its default action) is one mouse down. Only the
                    // AccessKit request: egui's own click of a focused widget on Space or Enter
                    // stays without effect, as before (task-187).
                    let ak_click = ui.input(|i| {
                        i.has_accesskit_action_request(r.id, egui::accesskit::Action::Click)
                    });
                    if ak_click {
                        let before303 = step303::Before::of(&self.panel);
                        self.mouse_down(c, now);
                        self.announce_303(&before303, &c.id, Some(r.id));
                    }
                    crate::a11y::describe(ui.ctx(), r.id, &self.panel, c);
                }
                down
            }
        }
    }

    /// Repeat and wind buttons: one step at mouse down, then the held-button schedule.
    fn repeat(&mut self, ui: &Ui, p: Placed, r: &Response) {
        let c = p.c;
        let (id, mut g) = gesture(ui, p);
        let down = r.is_pointer_button_down_on();
        let time = ui.input(|i| i.time);
        if down && !g.down {
            g.t0 = time;
            g.taken = 0;
        }
        g.down = down;
        if down {
            let t = ((time - g.t0) * 1000.0) as u32;
            let total = 1 + behavior::repeat_extra_steps(t);
            while g.taken < total {
                self.panel.arrow(&c.id);
                g.taken += 1;
            }
            ui.ctx().request_repaint_after(Duration::from_millis(16));
        }
        ui.data_mut(|d| d.insert_temp(id, g));
    }

    /// A knob: a vertical drag over the skin's `span` for the full range (5 times with Shift
    /// held at mouse down), no jump; a double click resets it.
    fn knob(&mut self, ui: &mut Ui, view: &View, placed: Placed) {
        let c = placed.c;
        let r = hit(ui, placed, view);
        let (id, mut g) = gesture(ui, placed);
        let down = r.is_pointer_button_down_on();
        if r.hovered() || down {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        }
        if down {
            self.panel.held = Some(c.id.clone());
            if !g.down {
                // Mouse down grabs the knob where it is (no jump); Shift now means fine.
                g.p0 = self.panel.knob(&c.id);
                g.y0 = press_origin(ui, view).map_or(0.0, |p| p.y);
                g.fine = ui.input(|i| i.modifiers.shift);
            }
            if let Some(p) = pointer(ui, view) {
                let v = behavior::knob_drag(g.p0, g.y0, p.y, c.knob_span(), g.fine);
                self.panel.set_knob(&c.id, v);
            }
        }
        g.down = down;
        ui.data_mut(|d| d.insert_temp(id, g));
        // A double click sets the knob to its value in a new empty song (docs/ui_spec.md
        // section 2); the demo panel has no link and no such song.
        if r.double_clicked()
            && let Some(v) = self.link.as_ref().and_then(|l| l.knob_default(&c.id))
        {
            self.panel
                .set_knob(&c.id, behavior::knob_from_engine(v as i32));
        }
        // a screen reader's request in a frame where the mouse drags this knob is dropped: the
        // drag sets the position from its own start on every frame and would undo it anyway
        if !down && let Some(t) = drive::target(c) {
            self.drive_value(ui, &r, &t, ui.input(|i| i.time));
        }
        crate::a11y::describe(ui.ctx(), r.id, &self.panel, c);
    }

    /// A fader: a press off the handle jumps the handle's center to the pointer, then the
    /// handle follows the drag (the skin's track and handle).
    fn fader(&mut self, ui: &mut Ui, view: &View, placed: Placed) {
        let c = placed.c;
        let r = hit(ui, placed, view);
        let (id, mut g) = gesture(ui, placed);
        let f = c.fader();
        let down = r.is_pointer_button_down_on();
        if down {
            self.panel.held = Some(c.id.clone());
            if !g.down {
                let y = press_origin(ui, view).map_or(0.0, |p| p.y) - c.y;
                let p = self.panel.fader(&c.id);
                if !behavior::fader_on_handle(p, y, &f) {
                    self.panel.set_fader(&c.id, behavior::fader_jump(y, &f));
                }
                g.p0 = self.panel.fader(&c.id);
                g.y0 = y + c.y;
            }
            if let Some(p) = pointer(ui, view) {
                self.panel
                    .set_fader(&c.id, behavior::fader_drag(g.p0, g.y0, p.y, &f));
            }
        }
        g.down = down;
        ui.data_mut(|d| d.insert_temp(id, g));
        // as for the knob: a request during a drag of this fader is dropped
        if !down && let Some(t) = drive::target(c) {
            self.drive_value(ui, &r, &t, ui.input(|i| i.time));
        }
        crate::a11y::describe(ui.ctx(), r.id, &self.panel, c);
    }

    /// The step strips of the two drum machines: press sets the step to the opposite (808)
    /// or the cycled value (909) and a drag paints every crossed step with that value.
    fn drum_strips(&mut self, ui: &mut Ui, view: &View, layout: &Layout) {
        for (n, s) in layout.strips.iter().enumerate() {
            let is808 = s.section == "tr808";
            let rect = view.rect(s.x, s.y, s.w, s.h);
            // keyed by the place in the layout too, so that a skin may show a strip twice; no
            // FOCUSABLE: the strip is not a Tab stop (the fork's task-186; the grids are
            // readable through `App::drum_grid`, task-191), the mouse works as with
            // `Sense::click_and_drag`
            let r = ui.interact(
                rect,
                egui::Id::unique(("hit", "strip", &s.id, n)),
                Sense::CLICK | Sense::DRAG,
            );
            let gid = egui::Id::unique(("gesture", "strip", &s.id, n));
            let mut g: Gesture = ui.data(|d| d.get_temp(gid)).unwrap_or_default();
            let down = r.is_pointer_button_down_on();
            if down {
                let Some(p) = (if g.down {
                    pointer(ui, view)
                } else {
                    press_origin(ui, view)
                }) else {
                    continue;
                };
                if let Some(step) = behavior::strip_step(s.x, s.cell(), p.x) {
                    let step = step as i32;
                    if !g.down {
                        g.new_value = if is808 {
                            u8::from(!self.panel.steps808[self.panel.inst808][step as usize])
                        } else {
                            // Cmd and Option on macOS, Ctrl and Alt elsewhere (task-281)
                            let m = ui.input(|i| behavior::step_mods(i.modifiers));
                            self.panel.step909_new_value(step as usize, m)
                        };
                        g.last_step = step;
                    }
                    // Every step crossed since the last frame.
                    let (a, b) = (g.last_step.min(step), g.last_step.max(step));
                    for s in a..=b {
                        if is808 {
                            self.panel.set_step808(s as usize, g.new_value != 0);
                        } else {
                            self.panel.set_step909(s as usize, g.new_value);
                        }
                    }
                    g.last_step = step;
                }
            }
            g.down = down;
            ui.data_mut(|d| d.insert_temp(gid, g));
        }
    }

    /// The drum machines' legends and name rows select the instrument; dragging keeps
    /// selecting, over all of the machine's hit areas.
    fn instrument_selection(&mut self, ui: &mut Ui, view: &View, layout: &Layout) {
        for (n, a) in layout.selects.iter().enumerate() {
            let rect = view.rect(a.x, a.y, a.w, a.h);
            // no FOCUSABLE, as the step strips (the fork's task-186)
            let r = ui.interact(
                rect,
                egui::Id::unique(("hit", "select", &a.id, n)),
                Sense::CLICK | Sense::DRAG,
            );
            if !r.is_pointer_button_down_on() {
                continue;
            }
            let Some(p) = pointer(ui, view) else {
                continue;
            };
            if let Some(i) = layout.select_hit(&a.section, p) {
                self.panel.select_instrument(a.section == "tr808", i);
            }
        }
    }
}
