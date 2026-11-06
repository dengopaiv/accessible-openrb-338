//! Windows and dialogs for a screen reader (task-193, A11Y-8; docs/ui_spec.md section 20.4):
//! the role and the name of a window or a modal, and text lines that take the focus so that
//! they can be read one by one. Everything here acts only with `on` (screen reader mode).

use eframe::egui::{self, Label, Response, RichText, Sense, Ui, UiKind, accesskit};

/// Gives the modal whose contents `ui` is in the role `role` and the name `name`;
/// `description` (the text of an alert) is what the screen reader reads with it. egui's node
/// of a modal is a Dialog without a name.
pub fn dialog(ui: &Ui, on: bool, role: accesskit::Role, name: &str, description: Option<&str>) {
    if !on {
        return;
    }
    let frame = ui
        .stack()
        .iter()
        .find(|f| f.kind() == Some(UiKind::Modal))
        .map(|f| f.unique_id);
    if let Some(id) = frame {
        set(ui.ctx(), id, role, name, description);
    }
}

/// Makes the window `window` (the id given to `egui::Window::id`) a Dialog named by its title:
/// egui's node of a window is a Window, which a screen reader does not announce as a dialog.
/// Called from the window's contents, after egui has made the node.
pub fn window(ui: &Ui, on: bool, window: egui::Id, name: &str) {
    if on {
        let node = window.with("move");
        set(ui.ctx(), node, accesskit::Role::Dialog, name, None);
    }
}

fn set(ctx: &egui::Context, id: egui::Id, role: accesskit::Role, name: &str, text: Option<&str>) {
    ctx.accesskit_node_builder(id, |b| {
        b.set_role(role);
        b.set_label(name);
        if let Some(d) = text {
            b.set_description(d);
        }
    });
}

/// A line of text of a window. With `on` it takes the focus (Tab and Shift+Tab walk the lines)
/// and a screen reader reads `spoken`, else the text; it scrolls into view when it gets the
/// focus. Without `on` it is a plain label.
pub fn line(ui: &mut Ui, on: bool, text: RichText, wrap: bool, spoken: Option<&str>) -> Response {
    let mut label = Label::new(text);
    if wrap {
        label = label.wrap();
    }
    if !on {
        return ui.add(label);
    }
    let r = ui.add(label.sense(Sense::FOCUSABLE));
    if let Some(s) = spoken {
        ui.ctx().accesskit_node_builder(r.id, |b| b.set_value(s));
    }
    if r.gained_focus() {
        r.scroll_to_me(None);
    }
    r
}
