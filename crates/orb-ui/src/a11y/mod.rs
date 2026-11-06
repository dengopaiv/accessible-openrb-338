//! The accessible layer of the panel (milestone m-7, docs/accessibility.md and
//! docs/accessibility_plan.md): the nodes a screen reader reads through AccessKit.
//!
//! Everything here writes into egui's AccessKit tree with `Context::accesskit_node_builder`,
//! whose closure runs only while AccessKit is on (a screen reader or another UI Automation
//! client has asked for the tree): with it off, a frame costs one lock per control and nothing
//! else.

pub mod names;
pub mod units;

use eframe::egui::{self, Rect, accesskit};

use crate::layout::Ctl;
use crate::model::Panel;
use names::Speech;

/// The key of the speech style in the context's data.
fn speech_id() -> egui::Id {
    egui::Id::unique("orb_a11y_speech")
}

/// Sets how knobs and faders speak their value from this frame on (task-192): the app sets it
/// every frame from screen reader mode and the setting "Speak raw values".
pub fn set_speech(ctx: &egui::Context, speech: Speech) {
    ctx.data_mut(|d| d.insert_temp(speech_id(), speech));
}

/// How knobs and faders speak their value ([`Speech::Plain`] until [`set_speech`]).
pub fn speech(ctx: &egui::Context) -> Speech {
    ctx.data(|d| d.get_temp(speech_id())).unwrap_or_default()
}

/// Writes the role, name, value and state of the control `c` into the node of its hit area
/// `id` (task-186). The node already has the bounds and the Focus and Click actions egui gives a
/// focusable widget.
pub fn describe(ctx: &egui::Context, id: egui::Id, panel: &Panel, c: &Ctl) {
    // read before the node builder, which holds the context's lock
    let speech = speech(ctx);
    ctx.accesskit_node_builder(id, |node| {
        let Some(role) = names::role(&c.kind) else {
            return;
        };
        node.set_role(role);
        if let Some(name) = names::name(c) {
            node.set_label(name);
        }
        if let Some(text) = names::value_text(panel, c, speech) {
            node.set_value(text);
        }
        match c.kind.as_str() {
            "knob" | "fader" => {
                if let Some(v) = panel.value(&c.id) {
                    node.set_numeric_value(f64::from(v));
                }
                node.set_min_numeric_value(0.0);
                node.set_max_numeric_value(127.0);
                node.set_numeric_value_step(1.0);
            }
            "toggle" | "radio" => node.set_toggled(panel.toggle_on(&c.id).into()),
            "grid" => node.set_toggled(panel.grid_lit(c).into()),
            _ => {}
        }
    });
}

/// A display's node (task-186): a label that cannot take the focus, its text the display's name
/// and what it shows ("Tempo 140 BPM"). The id is the one [`hit_id`] gives the display.
/// The display is not a value control: A11Y-3 (task-187) joins it with its up and down
/// buttons into one spin button.
pub fn describe_display(ctx: &egui::Context, rect: Rect, panel: &Panel, c: &Ctl) {
    ctx.accesskit_node_builder(hit_id(c), |node| {
        node.set_role(accesskit::Role::Label);
        let name = names::name(c).unwrap_or_default();
        let shown = names::display_text(&c.id, panel.display(&c.id));
        let text = format!("{name} {shown}");
        // A label's text is its value: UI Automation names a Label from the value
        // (accesskit_windows 0.35.2 `name`, node.rs:404). The label carries the same text for
        // the readers of the label.
        node.set_value(text.clone());
        node.set_label(text);
        node.set_bounds(accesskit::Rect {
            x0: rect.min.x.into(),
            y0: rect.min.y.into(),
            x1: rect.max.x.into(),
            y1: rect.max.y.into(),
        });
    });
}

/// The egui id of a part's node: its hit area's id (`App::hit`). The kind is part of the key:
/// the 303's accent knob and accent button share the layout id `tb303_N.accent`.
pub fn hit_id(c: &Ctl) -> egui::Id {
    hit_id_of(&c.kind, &c.id)
}

/// [`hit_id`] by the layout's kind and id (the tests' helper uses it).
pub fn hit_id_of(kind: &str, layout_id: &str) -> egui::Id {
    egui::Id::unique(("hit", kind, layout_id))
}
