//! Shared helper of the accessibility tests (task-183, docs/accessibility.md section 6.2):
//! runs frames headless with AccessKit enabled, keeps the last `TreeUpdate`, finds nodes, reads
//! them and sends the requests a screen reader sends. No window and no GPU.
//!
//! Use it from a test file with `#[path = "common/a11y.rs"] mod a11y;`.

#![allow(dead_code)]

use eframe::egui::accesskit::{
    Action, ActionData, ActionRequest, Node, NodeId, Role, Toggled, TreeId, TreeUpdate,
};
use eframe::egui::{self, Event, Id, Key, Modifiers, Pos2, RawInput, Rect, Vec2};
use orb_ui::app::App;
use orb_ui::layout::{PANEL_H, PANEL_W};
use orb_ui::model::Panel;

/// The egui id of a panel control's hit area (`App::hit`): the kind is part of the key. The
/// app's own function, so a change of the key is seen by the tests.
pub fn hit_id(kind: &str, layout_id: &str) -> Id {
    orb_ui::a11y::hit_id_of(kind, layout_id)
}

/// The accessibility tree of one frame.
pub struct Tree {
    pub update: TreeUpdate,
}

impl Tree {
    pub fn len(&self) -> usize {
        self.update.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.update.nodes.is_empty()
    }

    pub fn nodes(&self) -> impl Iterator<Item = (NodeId, &Node)> {
        self.update.nodes.iter().map(|(id, n)| (*id, n))
    }

    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes().find(|(i, _)| *i == id).map(|(_, n)| n)
    }

    /// The node of an egui id (the node id is derived from it).
    pub fn by_id(&self, id: Id) -> Option<&Node> {
        self.node(id.accesskit_id())
    }

    /// The node of a panel control, by the layout's kind and id.
    pub fn by_layout(&self, kind: &str, layout_id: &str) -> Option<&Node> {
        self.by_id(hit_id(kind, layout_id))
    }

    /// Nodes whose name (AccessKit label) is exactly `name`.
    pub fn by_name<'a>(&'a self, name: &'a str) -> impl Iterator<Item = (NodeId, &'a Node)> {
        self.nodes().filter(move |(_, n)| n.label() == Some(name))
    }

    pub fn by_role(&self, role: Role) -> impl Iterator<Item = (NodeId, &Node)> {
        self.nodes().filter(move |(_, n)| n.role() == role)
    }

    pub fn named(&self) -> Vec<String> {
        self.nodes()
            .filter_map(|(_, n)| n.label().map(str::to_string))
            .collect()
    }

    /// The node that has the keyboard focus, if it is in the tree.
    pub fn focused(&self) -> Option<&Node> {
        self.node(self.update.focus)
    }

    pub fn focused_id(&self) -> NodeId {
        self.update.focus
    }
}

pub fn name(n: &Node) -> Option<&str> {
    n.label()
}

pub fn role(n: &Node) -> Role {
    n.role()
}

pub fn value(n: &Node) -> Option<&str> {
    n.value()
}

pub fn numeric_value(n: &Node) -> Option<f64> {
    n.numeric_value()
}

pub fn toggled(n: &Node) -> Option<Toggled> {
    n.toggled()
}

pub fn supports(n: &Node, action: Action) -> bool {
    n.supports_action(action)
}

/// A request as a screen reader sends it.
pub fn request(target: Id, action: Action, data: Option<ActionData>) -> Event {
    Event::AccessKitActionRequest(ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: target.accesskit_id(),
        data,
    })
}

pub fn set_value_request(target: Id, v: f64) -> Event {
    request(target, Action::SetValue, Some(ActionData::NumericValue(v)))
}

fn key_event(key: Key, pressed: bool) -> Event {
    Event::Key {
        key,
        physical_key: Some(key),
        pressed,
        repeat: false,
        modifiers: Modifiers::NONE,
    }
}

/// An egui context with AccessKit on, running frames of any UI closure; the base of `AppTree`
/// and of tests with a small widget.
pub struct Frames {
    pub ctx: egui::Context,
    pub time: f64,
    pub size: Vec2,
    pub tree: Option<Tree>,
}

impl Frames {
    pub fn new(size: Vec2) -> Self {
        let ctx = egui::Context::default();
        egui_extras::install_image_loaders(&ctx);
        ctx.enable_accesskit();
        Self {
            ctx,
            time: 0.0,
            size,
            tree: None,
        }
    }

    /// One frame with the given events; the tree it produced is kept and returned.
    pub fn run(&mut self, events: Vec<Event>, ui_fn: impl FnMut(&mut egui::Ui)) -> &Tree {
        self.time += 0.016;
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, self.size)),
            time: Some(self.time),
            events,
            focused: true,
            ..Default::default()
        };
        let mut out = self.ctx.run_ui(input, ui_fn);
        out.textures_delta.clear();
        let update = out
            .platform_output
            .accesskit_update
            .expect("an accesskit update");
        self.tree.insert(Tree { update })
    }
}

/// The panel's `App` run headless, with the tree of the last frame.
pub struct AppTree {
    pub frames: Frames,
    pub app: App,
}

impl AppTree {
    /// The app with its menu bar shown (as in the window), settled for a few frames: the first
    /// frames load the SVG parts.
    pub fn new() -> Self {
        let mut app = App::new(Panel::new(), None);
        app.menu = true;
        let mut h = Self {
            frames: Frames::new(Vec2::new(PANEL_W, PANEL_H)),
            app,
        };
        for _ in 0..4 {
            h.frame(vec![]);
        }
        h
    }

    pub fn frame(&mut self, events: Vec<Event>) -> &Tree {
        let app = &mut self.app;
        self.frames.run(events, |ui| app.show(ui))
    }

    pub fn tree(&self) -> &Tree {
        self.frames.tree.as_ref().expect("a frame was run")
    }

    /// Presses and releases a key in two frames; returns the tree after the release.
    pub fn key(&mut self, key: Key) -> &Tree {
        self.frame(vec![key_event(key, true)]);
        self.frame(vec![key_event(key, false)])
    }

    /// Sends a screen reader request and runs a second frame so the app has acted on it.
    pub fn send(&mut self, target: Id, action: Action, data: Option<ActionData>) -> &Tree {
        self.frame(vec![request(target, action, data)]);
        self.frame(vec![])
    }
}
