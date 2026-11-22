//! The one controller of the panel (task-266, docs/ui_spec.md section 22): all the input of
//! the main window's panel, run on whatever layout the current skin gives (`skin::Layout`, in
//! the skin's units), with the original's interaction (docs/ui_spec.md section 2): the hit
//! tests and gestures of every control kind, knob and fader drags (Shift: fine), the fader's
//! jump, the repeat and wind buttons, the drum machines' step strips and instrument selection,
//! the rows' focus, the logo, the cursor and the computer keyboard. A skin has no input code.

use std::time::Duration;

use egui::{Key, Pos2, Response, Sense, Ui};

use crate::app::{App, View};
use crate::behavior;
use crate::keys::{self, Action};
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
}

fn gesture(ui: &Ui, p: Placed) -> (egui::Id, Gesture) {
    let id = egui::Id::unique(("gesture", &p.c.kind, &p.c.id, p.n));
    (id, ui.data(|d| d.get_temp(id)).unwrap_or_default())
}

fn hit(ui: &mut Ui, p: Placed, view: &View) -> Response {
    let c = p.c;
    ui.interact(
        view.rect(c.x, c.y, c.w, c.h),
        egui::Id::unique(("hit", &c.kind, &c.id, p.n)),
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
    /// shows.
    pub(crate) fn keyboard(&mut self, ui: &Ui) {
        let (events, now, focused) = ui.input(|i| (i.events.clone(), i.time, i.focused));
        if !focused {
            self.holds.clear();
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
            if !pressed {
                self.holds.retain(|h| h.key != k);
                continue;
            }
            if repeat || modifiers.command || modifiers.ctrl {
                continue;
            }
            if let Some(a) = keys::map(k, modifiers.shift, self.key_mode, self.panel.focus) {
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

    fn act(&mut self, k: Key, action: Action, now: f64) {
        // (a press without the release before it, e.g. after the window lost the focus)
        self.holds.retain(|h| h.key != k);
        let mut hold = Hold {
            key: k,
            id: None,
            arrow: None,
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
    pub(crate) fn panel_input(&mut self, ui: &mut Ui, view: &View, layout: &Layout) -> Vec<bool> {
        let now = ui.input(|i| i.time);
        let mut pressed = vec![false; layout.controls.len()];
        for (n, c) in layout.controls.iter().enumerate() {
            if !self.hidden(&c.id) {
                pressed[n] = self.control(ui, view, Placed { c, n }, now);
            }
        }
        self.drum_strips(ui, view, layout);
        self.instrument_selection(ui, view, layout);
        pressed
    }

    /// One control's input; whether it shows pressed.
    fn control(&mut self, ui: &mut Ui, view: &View, p: Placed, now: f64) -> bool {
        let c = p.c;
        match c.kind.as_str() {
            // display only; the 808's mod window takes no input of its own (its press gives the
            // row the focus, as any press in a row does)
            "area" | "steps" | "display" | "led" | "meter" | "modview" => false,
            "knob" => {
                self.knob(ui, view, p);
                false
            }
            "fader" => {
                self.fader(ui, view, p);
                false
            }
            kind => {
                let r = hit(ui, p, view);
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
    }

    /// The step strips of the two drum machines: press sets the step to the opposite (808)
    /// or the cycled value (909) and a drag paints every crossed step with that value.
    fn drum_strips(&mut self, ui: &mut Ui, view: &View, layout: &Layout) {
        for (n, s) in layout.strips.iter().enumerate() {
            let is808 = s.section == "tr808";
            let rect = view.rect(s.x, s.y, s.w, s.h);
            // keyed by the place in the layout too, so that a skin may show a strip twice
            let r = ui.interact(
                rect,
                egui::Id::unique(("hit", "strip", &s.id, n)),
                Sense::click_and_drag(),
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
            let r = ui.interact(
                rect,
                egui::Id::unique(("hit", "select", &a.id, n)),
                Sense::click_and_drag(),
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
