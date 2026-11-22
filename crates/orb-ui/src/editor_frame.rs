//! The frame the pattern editors share (task-280): the 303 editor (`crate::pattern_editor`,
//! task-274) and the drum editors (`crate::drum_editor`, the 808's, and the 909's of task-281).
//! A floating window that is placed as it opens and keeps its place ([`Placing`],
//! [`show_window`]), the header's controls (the bank buttons, the pattern number between its
//! arrows, Copy, Paste and Clear, Length and Shuffle), the step grid's geometry and painting
//! (16 columns that fill the width, the playing step, the dimmed steps beyond the length), the
//! arrow buttons and the mouse on a grid ([`grid_press`]). Each editor adds only its rows, its
//! cells and what a click on them does.

use egui::{Color32, Context, Pos2, Rect, Sense, Stroke, Ui, Vec2, pos2, vec2};

use crate::palette::{AMBER, INK, INK_DIM, SLATE_LINE};

/// The steps of a pattern.
pub const STEPS: usize = 16;

/// The frames in which an opening places the window: egui's first frame of a window measures
/// it, the second still holds it to the screen by the size measured before its content was
/// complete (seen: the place's x cut to 24 on a 624-point window), the third has its size.
pub(crate) const PLACE_FRAMES: u8 = 3;

/// The least width of a step column (the columns widen to fill the window), a row's height,
/// the row labels' column, in points.
pub(crate) const CELL_W: f32 = 22.0;
pub(crate) const CELL_H: f32 = 19.0;
pub(crate) const KEY_W: f32 = 66.0;
/// The size of the arrow buttons under and beside the grid (one size for all), and the gap
/// between the grid and a column of buttons at its right.
pub(crate) const ARROW: Vec2 = vec2(24.0, 20.0);
pub(crate) const GAP: f32 = 6.0;
/// The step numbers' row over the grid.
pub(crate) const HEAD_H: f32 = 14.0;
/// The gap between two grids (a divider line in its middle).
pub(crate) const SPLIT_H: f32 = 9.0;
/// The size of the row labels.
pub(crate) const LABEL_SIZE: f32 = 15.5;

/// The step cell `(step, row)` of a grid at `rect` that holds `p`, `None` outside it.
pub(crate) fn cell_at(rect: Rect, rows: usize, p: Pos2) -> Option<(usize, usize)> {
    if !rect.contains(p) {
        return None;
    }
    let s = ((p.x - rect.left()) / col_w(rect)).floor() as usize;
    let r = ((p.y - rect.top()) / CELL_H).floor() as usize;
    (s < STEPS && r < rows).then_some((s, r))
}

/// The width of a step column of a grid at `rect`: the grid's width over the 16 steps.
pub(crate) fn col_w(rect: Rect) -> f32 {
    rect.width() / STEPS as f32
}

/// The step column of a grid at `rect` under `x`, `None` outside its columns.
pub(crate) fn column_at(rect: Rect, x: f32) -> Option<usize> {
    let s = ((x - rect.left()) / col_w(rect)).floor();
    (s >= 0.0 && (s as usize) < STEPS).then_some(s as usize)
}

/// The cell of step `s`, row `r` of a grid at `rect`.
pub(crate) fn cell_rect(rect: Rect, s: usize, r: usize) -> Rect {
    let w = col_w(rect);
    Rect::from_min_size(
        pos2(rect.left() + s as f32 * w, rect.top() + r as f32 * CELL_H),
        vec2(w, CELL_H),
    )
}

/// How an editor's window is placed and brought to the front, kept with the editor.
#[derive(Clone, Debug, Default)]
pub(crate) struct Placing {
    /// The window comes to the front in the next frame.
    pub(crate) front: bool,
    /// The frames left in which an opening places the window (the first frame of a window
    /// only measures it, and its place is then held to the screen by a wrong size).
    pub(crate) place: u8,
    /// The window's left top at the last frame, and whether it moved since the last save.
    pub(crate) last_min: Option<Pos2>,
    pub(crate) moved: bool,
    /// Where the window was drawn last.
    pub(crate) window: Option<Rect>,
}

impl Placing {
    /// An opening (or a reopening of an open window): to the front, and placed when it was
    /// closed.
    pub(crate) fn open(&mut self, was_open: bool) {
        if !was_open {
            self.place = PLACE_FRAMES;
        }
        self.front = true;
    }
}

/// What a frame of [`show_window`] gives back.
pub(crate) struct Shown {
    /// False when the close button closed the window.
    pub(crate) open: bool,
    /// The window's new place to save (the standalone app's only): set once the user lets go
    /// of the window after moving it.
    pub(crate) save: Option<[i32; 2]>,
}

/// An editor's window.
pub(crate) struct Spec<'a> {
    pub(crate) id: egui::Id,
    pub(crate) title: &'a str,
    /// The standalone app's saved place (`None` in the plugin).
    pub(crate) saved: Option<[i32; 2]>,
    /// The window's size before its first frame.
    pub(crate) guess: Vec2,
    /// Whether a move by the mouse is reported for saving (the standalone app).
    pub(crate) keep: bool,
}

/// Shows an editor's window: draggable, closable, neither resizable nor collapsible. An
/// opening places it at the saved place, else centered by its size.
pub(crate) fn show_window(
    ctx: &Context,
    spec: Spec<'_>,
    p: &mut Placing,
    add: impl FnOnce(&mut Ui),
) -> Shown {
    let Spec {
        id,
        title,
        saved,
        guess,
        keep,
    } = spec;
    let size = p.window.map_or(guess, |r| r.size());
    let place = match saved {
        Some([x, y]) => pos2(x as f32, y as f32),
        None => ctx.content_rect().center() - size / 2.0,
    };
    let mut open = true;
    let mut window = egui::Window::new(title)
        .id(id)
        .open(&mut open)
        .collapsible(false)
        .resizable(false);
    if p.place > 0 {
        p.place -= 1;
        // (`current_pos` sets the first frame's place only: a window moved by its title bar
        // takes the stored place back after it, egui 0.37)
        window = window.current_pos(place);
        if let Some(mut s) = egui::AreaState::load(ctx, id) {
            s.set_left_top_pos(place);
            let layer = egui::LayerId::new(egui::Order::Middle, id);
            ctx.memory_mut(|m| m.areas_mut().set_state(layer, s));
        }
    }
    let resp = window.show(ctx, add);
    let down = ctx.input(|i| i.pointer.any_down());
    if let Some(r) = resp {
        if std::mem::take(&mut p.front) {
            ctx.move_to_top(r.response.layer_id);
        }
        p.window = Some(r.response.rect);
        // a move by the mouse (not the window's own placing as it opens)
        let min = r.response.rect.min;
        if down && p.last_min.is_some_and(|m| m != min) {
            p.moved = true;
        }
        p.last_min = Some(min);
    }
    // the place is saved once the window is let go
    let mut save = None;
    if keep && p.moved && !down {
        p.moved = false;
        save = p.last_min.map(|m| [m.x.round() as i32, m.y.round() as i32]);
    }
    if !open {
        p.last_min = None;
    }
    Shown { open, save }
}

/// A button with a triangle drawn on it, pointing `dir` (a unit vector), at `rect`; `salt`
/// tells the editors' buttons apart. Whether clicked.
pub(crate) fn arrow_button(
    ui: &mut Ui,
    rect: Rect,
    dir: Vec2,
    buttons: &mut Vec<(&'static str, Rect)>,
    salt: &'static str,
    name: &'static str,
) -> bool {
    // (not `ui.put`, which would move the layout's cursor below the button)
    let r = ui.interact(rect, egui::Id::unique((salt, name)), Sense::click());
    buttons.push((name, rect));
    paint_arrow(ui, &r, dir);
    r.clicked()
}

/// [`arrow_button`] laid out in the current row (the pattern number's arrows), `size` big.
fn arrow_in_row(
    ui: &mut Ui,
    size: Vec2,
    dir: Vec2,
    buttons: &mut Vec<(&'static str, Rect)>,
    name: &'static str,
) -> bool {
    let (rect, r) = ui.allocate_exact_size(size, Sense::click());
    buttons.push((name, rect));
    paint_arrow(ui, &r, dir);
    r.clicked()
}

/// A button's frame in the style's look and a triangle pointing `dir`.
fn paint_arrow(ui: &Ui, r: &egui::Response, dir: Vec2) {
    let rect = r.rect;
    let v = ui.style().interact(r);
    ui.painter().rect(
        rect,
        v.corner_radius,
        v.weak_bg_fill,
        v.bg_stroke,
        egui::StrokeKind::Inside,
    );
    let c = rect.center();
    let a = 4.5;
    let side = vec2(-dir.y, dir.x);
    let pts = vec![c + dir * a, c - dir * a + side * a, c - dir * a - side * a];
    let color = if r.hovered() { INK } else { INK_DIM };
    ui.painter()
        .add(egui::Shape::convex_polygon(pts, color, Stroke::NONE));
}

/// The header's compact spacing, so a window fits the smallest panel (624 points wide).
pub(crate) fn compact(ui: &mut Ui) {
    ui.spacing_mut().item_spacing.x = 4.0;
    ui.spacing_mut().button_padding = vec2(5.0, 1.0);
}

/// The bank buttons A B C D of pattern `pattern` (0..=31); the bank clicked, if another.
pub(crate) fn bank_buttons(
    ui: &mut Ui,
    pattern: u8,
    buttons: &mut Vec<(&'static str, Rect)>,
) -> Option<u8> {
    let bank = pattern / 8;
    let mut to = None;
    for b in 0..4u8 {
        let name = ["A", "B", "C", "D"][b as usize];
        let r = ui.selectable_label(bank == b, name);
        buttons.push((name, r.rect));
        if r.clicked() && bank != b {
            to = Some(b);
        }
    }
    to
}

/// What the pattern number asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Number {
    /// The left or right arrow.
    Prev,
    Next,
    /// The number typed or dragged: pattern `n` (0..=7) of the bank.
    Set(u8),
}

/// The pattern number 1-8 of pattern `pattern` as a `DragValue` (drag up or down, or click to
/// type) between a left and a right arrow, each enabled as given.
pub(crate) fn number_field(
    ui: &mut Ui,
    pattern: u8,
    prev: bool,
    next: bool,
    buttons: &mut Vec<(&'static str, Rect)>,
) -> Option<Number> {
    let small = vec2(16.0, ui.spacing().interact_size.y);
    let k0 = pattern % 8;
    let mut out = None;
    let left = ui
        .add_enabled_ui(prev, |ui| {
            arrow_in_row(ui, small, vec2(-1.0, 0.0), buttons, "number_prev")
        })
        .inner;
    if left && prev {
        out = Some(Number::Prev);
    }
    let mut n = k0 + 1;
    let dv = egui::DragValue::new(&mut n).range(1..=8).speed(0.05);
    let (r, changed) = value_field(ui, "number", dv);
    buttons.push(("number", r.rect));
    if changed && n != k0 + 1 {
        out = Some(Number::Set(n.clamp(1, 8) - 1));
    }
    let right = ui
        .add_enabled_ui(next, |ui| {
            arrow_in_row(ui, small, vec2(1.0, 0.0), buttons, "number_next")
        })
        .inner;
    if right && next {
        out = Some(Number::Next);
    }
    out
}

/// A value of the header (the pattern number, Length): drag it up or down (Shift: slower, egui's
/// `DragValue`), or click it, type and press Enter (or click elsewhere) to set it; Escape
/// cancels the typed value (task-280, as the Keyboard Shortcuts help says). Its response and
/// whether its value changed.
///
/// egui 0.37 sets a typed value whenever the field loses the focus without Escape pressed in
/// that frame; after Escape it loses the focus, and the next frame (Escape no longer pressed)
/// sets the text typed. So a change in the frames right after an Escape on the focused field is
/// dropped here.
fn value_field(ui: &mut Ui, salt: &str, dv: egui::DragValue<'_>) -> (egui::Response, bool) {
    let key = ui.scope_id().with(("orb_value_field", salt));
    // whether it had the focus at the end of the last frame (egui drops the focus on Escape as
    // the frame begins), and the frames left in which a change is dropped
    let (focused, cancel): (bool, u8) = ui.data(|d| d.get_temp(key)).unwrap_or_default();
    let mut cancel = cancel.saturating_sub(1);
    if focused && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        cancel = 3;
    }
    // a typed value is set at the end only (Enter), not as each key is typed: one change
    let r = ui.add(dv.update_while_editing(false));
    let changed = r.changed() && cancel == 0;
    let focused = r.has_focus();
    ui.data_mut(|d| d.insert_temp(key, (focused, cancel)));
    (r, changed)
}

/// Copy, Paste (enabled with `paste`) and Clear: the name of the one clicked.
pub(crate) fn edit_buttons(
    ui: &mut Ui,
    paste: bool,
    buttons: &mut Vec<(&'static str, Rect)>,
) -> Option<&'static str> {
    let mut out = None;
    for name in ["Copy", "Paste", "Clear"] {
        let ok = name != "Paste" || paste;
        let r = ui.add_enabled(ok, egui::Button::new(name));
        buttons.push((name, r.rect));
        if r.clicked() {
            out = Some(name);
        }
    }
    out
}

/// What the Length value and the Shuffle toggle ask for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Header {
    /// A new length, and whether its value is being dragged (a drag is one change).
    pub(crate) length: Option<(u8, bool)>,
    /// Whether the Length value is being dragged.
    pub(crate) length_drag: bool,
    pub(crate) shuffle: bool,
}

/// Length 1-16 (a `DragValue`) and the Shuffle toggle of a pattern's `bytes` (byte 1 the
/// length, byte 0 the shuffle flag, as in every pattern of a song).
pub(crate) fn length_shuffle(
    ui: &mut Ui,
    bytes: &[u8],
    buttons: &mut Vec<(&'static str, Rect)>,
) -> Header {
    let mut h = Header::default();
    let mut len = bytes[1].clamp(1, STEPS as u8);
    let dv = egui::DragValue::new(&mut len)
        .range(1..=16)
        .speed(0.1)
        .prefix("Length ");
    let (r, changed) = value_field(ui, "length", dv);
    buttons.push(("Length", r.rect));
    h.length_drag = r.dragged();
    if changed {
        h.length = Some((len.clamp(1, 16), r.dragged()));
    }
    let r = ui.selectable_label(bytes[0] != 0, "Shuffle");
    buttons.push(("Shuffle", r.rect));
    h.shuffle = r.clicked();
    h
}

/// The step numbers 1 to 16 in `head`, over the columns of a grid from `x0`, `cw` wide; the
/// playing step's in amber, those beyond the length `len` faint.
pub(crate) fn step_numbers(
    ui: &Ui,
    head: Rect,
    x0: f32,
    cw: f32,
    len: usize,
    playing: Option<usize>,
) {
    for st in 0..STEPS {
        let c = pos2(x0 + (st as f32 + 0.5) * cw, head.center().y);
        let color = if playing == Some(st) {
            AMBER
        } else if st < len {
            INK_DIM
        } else {
            SLATE_LINE
        };
        ui.painter().text(
            c,
            egui::Align2::CENTER_CENTER,
            (st + 1).to_string(),
            egui::FontId::proportional(10.0),
            color,
        );
    }
}

/// The mouse on a grid in a frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Press {
    /// The button went down on cell `(step, row)`.
    Down(usize, usize),
    /// The button is held: the cell under the mouse, and its step column (whatever the row).
    Held {
        cell: Option<(usize, usize)>,
        column: Option<usize>,
    },
    /// The button went up after a press on the grid.
    Release,
}

/// The press on the grid at `rect` of `rows` rows, interacted with as `id`; `was_down` is the
/// grid's press at the last frame, updated. The cell under the mouse is lit faintly.
pub(crate) fn grid_press(
    ui: &Ui,
    id: egui::Id,
    rect: Rect,
    rows: usize,
    was_down: &mut bool,
) -> Option<Press> {
    let r = ui.interact(rect, id, Sense::click_and_drag());
    hover_cell(ui, &r, rect, rows);
    let down = r.is_pointer_button_down_on();
    let first = !*was_down;
    *was_down = down;
    if down {
        let p = pointer_pos(ui, first);
        let cell = p.and_then(|p| cell_at(rect, rows, p));
        if first {
            cell.map(|(s, row)| Press::Down(s, row))
        } else {
            Some(Press::Held {
                cell,
                column: p.and_then(|p| column_at(rect, p.x)),
            })
        }
    } else if !first {
        Some(Press::Release)
    } else {
        None
    }
}

/// The pointer: where the press began on its first frame, else where it is.
fn pointer_pos(ui: &Ui, first: bool) -> Option<Pos2> {
    ui.input(|i| {
        if first {
            i.pointer.press_origin().or(i.pointer.latest_pos())
        } else {
            i.pointer.latest_pos()
        }
    })
}

/// The cell under the mouse, lit faintly.
fn hover_cell(ui: &Ui, r: &egui::Response, rect: Rect, rows: usize) {
    if let Some((s, row)) = r.hover_pos().and_then(|p| cell_at(rect, rows, p)) {
        ui.painter()
            .rect_filled(cell_rect(rect, s, row), 0.0, Color32::from_white_alpha(18));
    }
}

/// The step columns' lines (a stronger one every four steps), the playing step's column and
/// the dimmed steps beyond the length.
pub(crate) fn paint_columns(
    painter: &egui::Painter,
    rect: Rect,
    len: usize,
    playing: Option<usize>,
) {
    let cw = col_w(rect);
    for st in 0..=STEPS {
        let x = rect.left() + st as f32 * cw;
        let color = if st % 4 == 0 {
            SLATE_LINE
        } else {
            SLATE_LINE.gamma_multiply(0.4)
        };
        painter.line_segment(
            [pos2(x, rect.top()), pos2(x, rect.bottom())],
            Stroke::new(1.0, color),
        );
    }
    if len < STEPS {
        let dim = Rect::from_min_max(pos2(rect.left() + len as f32 * cw, rect.top()), rect.max);
        painter.rect_filled(dim, 0.0, Color32::from_black_alpha(110));
    }
    if let Some(st) = playing {
        let col = Rect::from_min_size(
            pos2(rect.left() + st as f32 * cw, rect.top()),
            vec2(cw, rect.height()),
        );
        painter.rect_filled(col, 0.0, Color32::from_white_alpha(30));
        painter.rect_stroke(
            col,
            0.0,
            Stroke::new(1.0, AMBER.gamma_multiply(0.6)),
            egui::StrokeKind::Inside,
        );
    }
}

/// `c`, dimmed for a step beyond the length.
pub(crate) fn dimmed(c: Color32, dim: bool) -> Color32 {
    if dim { c.gamma_multiply(0.4) } else { c }
}

/// A divider line across `rect`'s middle, from its left to `right`.
pub(crate) fn divider(painter: &egui::Painter, rect: Rect, right: f32) {
    let y = rect.center().y;
    painter.line_segment(
        [pos2(rect.left(), y), pos2(right, y)],
        Stroke::new(1.0, SLATE_LINE),
    );
}

/// `bytes`, a pattern whose steps are `step_len` bytes from byte 2, with the steps rotated by
/// one step, right (later) or left, wrapping within the pattern's length (byte 1, 1..=16); the
/// steps beyond the length stay.
pub(crate) fn shifted(bytes: &[u8], step_len: usize, right: bool) -> Vec<u8> {
    let n = bytes[1].clamp(1, STEPS as u8) as usize;
    let at = |s: usize| 2 + step_len * s;
    let mut b = bytes.to_vec();
    for s in 0..n {
        let from = if right { (s + n - 1) % n } else { (s + 1) % n };
        b[at(s)..at(s) + step_len].copy_from_slice(&bytes[at(from)..at(from) + step_len]);
    }
    b
}
