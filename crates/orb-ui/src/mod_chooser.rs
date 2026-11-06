//! The mod chooser (task-195, A11Y-10; docs/ui_spec.md section 20.7): Mods > Choose Mod... in
//! screen reader mode, Ctrl/Cmd+M. A dialog with a filter field and the list of the Mods menu
//! as a tree, so that a folder of 300 mods is not walked with the arrows of a pop-up menu.
//!
//! - **Two focus stops**: the filter (a text field) and the list; Tab and Shift+Tab move between
//!   them, Down in the filter moves to the list. Typing narrows the list by name (ASCII case
//!   ignored, as the menu sorts), live.
//! - **The list** is a Tree: "Standard ReBirth" first, then the menu's entries in the menu's order,
//!   built from the cached list ([`App::mods`]; no folder read). An entry with variants (mods that
//!   share an INFO name) is a collapsed item; Right opens it to its files, Left closes it, or goes
//!   from a file to its entry. Up, Down, Home, End and Page Up / Page Down move the selection; the
//!   tree is one focus stop whose active descendant is the selected item, as the drum grid's
//!   (`a11y::grid`).
//! - **Enter** chooses, as the menu does: the entry's mod (for a merged name the preferred variant,
//!   [`ModEntry::path`]) by `Cmd::UseKit`, a file by `Cmd::ChooseVariant`. Escape closes the dialog
//!   from either stop. **Ctrl/Cmd+I** opens Mod Info for the selected entry without loading it
//!   ([`App::show_mod_info_of`]).
//! - **An item is read** as its name, then "variants: N", "has demo" and "current" where they hold
//!   (a file: its label, "preferred", "has demo", "current").

use std::collections::BTreeSet;

use eframe::egui::{
    self, Align2, Color32, Context, FontId, Id, Key, Modifiers, Rect, RichText, Sense, Stroke, Ui,
    UiBuilder, accesskit, vec2,
};
use orb_cli::mods::{Kit, STANDARD};

use crate::a11y::window as win;
use crate::app::{App, Win};
use crate::menu::{Cmd, ModEntry};
use crate::palette::{AMBER, INK, INK_DIM, SLATE_DARK, SLATE_LINE};
use crate::shortcuts;

/// The height of an item in points.
const ROW_H: f32 = 22.0;
/// The most items a Page Up or Page Down moves.
const PAGE: usize = 10;

/// An item of the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Item {
    /// "Standard ReBirth", the built-in kit.
    Standard,
    /// An entry of the Mods menu: index in the cached list.
    Entry(usize),
    /// A file of an entry with variants: (entry, variant).
    Variant(usize, usize),
}

/// The state of the open dialog.
#[derive(Clone, Debug)]
pub struct Chooser {
    /// What the filter field holds.
    pub filter: String,
    /// The selected item (the tree's active descendant).
    pub cursor: Item,
    /// The entries (indexes of the list) whose variants are shown.
    pub expanded: BTreeSet<usize>,
    /// The item last scrolled into view.
    scrolled: Option<Item>,
    /// Whether the list, not the filter, had the focus last: where it goes back to after
    /// Mod Info (opened from here) closes.
    list_stop: bool,
}

/// What a key asks of the dialog besides the state changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    None,
    /// Enter: choose the item.
    Choose(Item),
    /// Ctrl+I: Mod Info of the item.
    Info(Item),
}

/// A key of the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListKey {
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Left,
    Right,
    Enter,
}

impl ListKey {
    pub const ALL: [(Key, ListKey); 9] = [
        (Key::ArrowUp, ListKey::Up),
        (Key::ArrowDown, ListKey::Down),
        (Key::Home, ListKey::Home),
        (Key::End, ListKey::End),
        (Key::PageUp, ListKey::PageUp),
        (Key::PageDown, ListKey::PageDown),
        (Key::ArrowLeft, ListKey::Left),
        (Key::ArrowRight, ListKey::Right),
        (Key::Enter, ListKey::Enter),
    ];
}

/// Whether the entry name `name` shows with the filter text `filter` (ASCII case ignored,
/// anywhere in the name; an empty filter shows everything).
pub fn name_matches(name: &str, filter: &str) -> bool {
    filter.is_empty()
        || name
            .to_ascii_lowercase()
            .contains(&filter.to_ascii_lowercase())
}

/// The items on show, in order: "Standard ReBirth", the entries of `list` that match `filter`,
/// each followed by its files when it is in `expanded`.
pub fn visible(list: &[ModEntry], filter: &str, expanded: &BTreeSet<usize>) -> Vec<Item> {
    let mut v = Vec::new();
    if name_matches(STANDARD, filter) {
        v.push(Item::Standard);
    }
    for (i, e) in list.iter().enumerate() {
        if !name_matches(&e.name, filter) {
            continue;
        }
        v.push(Item::Entry(i));
        if expanded.contains(&i) {
            v.extend((0..e.variants.len()).map(|j| Item::Variant(i, j)));
        }
    }
    v
}

/// What choosing an item does: the Mods menu's command for it.
pub fn choose_cmd(list: &[ModEntry], item: Item) -> Option<Cmd> {
    Some(match item {
        Item::Standard => Cmd::UseKit(Kit::Standard),
        Item::Entry(i) => Cmd::UseKit(Kit::Mod(list.get(i)?.path.clone())),
        Item::Variant(i, j) => Cmd::ChooseVariant(list.get(i)?.variants.get(j)?.path.clone()),
    })
}

/// The kit Mod Info shows for an item.
pub fn info_kit(list: &[ModEntry], item: Item) -> Option<Kit> {
    Some(match item {
        Item::Standard => Kit::Standard,
        Item::Entry(i) => Kit::Mod(list.get(i)?.path.clone()),
        Item::Variant(i, j) => Kit::Mod(list.get(i)?.variants.get(j)?.path.clone()),
    })
}

impl Chooser {
    /// A closed-filter dialog with the selection on `cursor`.
    pub fn new(cursor: Item) -> Self {
        Self {
            filter: String::new(),
            cursor,
            expanded: BTreeSet::new(),
            scrolled: None,
            list_stop: false,
        }
    }

    /// Keeps the selection on an item on show: else the first one.
    pub fn keep_valid(&mut self, rows: &[Item]) {
        if !rows.contains(&self.cursor)
            && let Some(first) = rows.first()
        {
            self.cursor = *first;
        }
    }

    /// The filter changed: the selection goes to the first item on show.
    pub fn filtered(&mut self, rows: &[Item]) {
        if let Some(first) = rows.first() {
            self.cursor = *first;
        }
    }

    /// A key in the list. `rows` is [`visible`] before the key; the caller computes it again
    /// after. Returns the item Enter chooses.
    pub fn key(&mut self, list: &[ModEntry], rows: &[Item], k: ListKey) -> Option<Item> {
        if rows.is_empty() {
            return None;
        }
        let at = rows.iter().position(|r| *r == self.cursor).unwrap_or(0);
        let last = rows.len() - 1;
        let to = |i: usize| rows[i.min(last)];
        match k {
            ListKey::Up => self.cursor = to(at.saturating_sub(1)),
            ListKey::Down => self.cursor = to(at + 1),
            ListKey::Home => self.cursor = to(0),
            ListKey::End => self.cursor = to(last),
            ListKey::PageUp => self.cursor = to(at.saturating_sub(PAGE)),
            ListKey::PageDown => self.cursor = to(at + PAGE),
            ListKey::Right => {
                if let Item::Entry(i) = self.cursor
                    && list.get(i).is_some_and(|e| !e.variants.is_empty())
                {
                    // closed: open it; open: the first file
                    if !self.expanded.insert(i) {
                        self.cursor = Item::Variant(i, 0);
                    }
                }
            }
            ListKey::Left => match self.cursor {
                Item::Variant(i, _) => self.cursor = Item::Entry(i),
                Item::Entry(i) => {
                    self.expanded.remove(&i);
                }
                Item::Standard => {}
            },
            ListKey::Enter => return Some(self.cursor),
        }
        None
    }
}

/// One line of the list as drawn and described.
#[derive(Clone, Debug)]
pub struct RowView {
    pub item: Item,
    /// What a screen reader reads.
    pub text: String,
    /// 1 for an entry, 2 for a file of one.
    pub level: usize,
    /// Some for an entry with variants.
    pub expanded: Option<bool>,
    /// The place among its siblings (1-based) and their number.
    pub pos: usize,
    pub size: usize,
}

fn window_id(serial: u32) -> Id {
    Id::unique(("orb_mod_chooser_window", serial))
}

fn filter_id(serial: u32) -> Id {
    Id::unique(("orb_mod_chooser_filter", serial))
}

/// The id of the tree (its `Ui`'s, its node and its focus stop).
pub fn list_id(serial: u32) -> Id {
    Id::unique(("orb_mod_chooser_list", serial))
}

/// The id of an item's node.
pub fn item_id(serial: u32, item: Item) -> Id {
    Id::unique(("orb_mod_chooser_item", serial, item))
}

fn bounds(r: Rect) -> accesskit::Rect {
    accesskit::Rect {
        x0: r.min.x.into(),
        y0: r.min.y.into(),
        x1: r.max.x.into(),
        y1: r.max.y.into(),
    }
}

impl App {
    /// Mods > Choose Mod... (Ctrl/Cmd+M): opens the dialog on the playing mod. Screen reader
    /// mode only; with the mode off it does nothing.
    pub fn show_chooser(&mut self) {
        if !self.screen_reader_mode() || self.chooser.is_some() {
            return;
        }
        let list = self.chooser_list();
        let cursor = match &self.doc.kit {
            Kit::Mod(_) => match self.mod_checks(list) {
                (_, Some(i)) => Item::Entry(i),
                _ => Item::Standard,
            },
            Kit::Standard => Item::Standard,
        };
        self.chooser_serial += 1;
        self.chooser = Some(Chooser::new(cursor));
    }

    /// Whether the dialog is open.
    pub fn chooser_is_open(&self) -> bool {
        self.chooser.is_some()
    }

    /// The cached list of the Mods menu (empty before the first read or after an error).
    fn chooser_list(&self) -> &[ModEntry] {
        match &self.mod_list {
            Some(Ok(l)) => l,
            _ => &[],
        }
    }

    /// The items on show with their texts.
    fn chooser_rows(&self, st: &Chooser) -> Vec<RowView> {
        let list = self.chooser_list();
        let (std_on, checked) = self.mod_checks(list);
        let active = match &self.doc.kit {
            Kit::Mod(p) => Some(p),
            Kit::Standard => None,
        };
        let items = visible(list, &st.filter, &st.expanded);
        let top = items
            .iter()
            .filter(|i| !matches!(i, Item::Variant(..)))
            .count();
        let mut n_top = 0;
        let mut out = Vec::with_capacity(items.len());
        for item in items {
            let mut parts: Vec<String> = Vec::new();
            let (level, expanded, pos, size);
            match item {
                Item::Standard => {
                    parts.push(STANDARD.into());
                    if std_on {
                        parts.push("current".into());
                    }
                    (level, expanded) = (1, None);
                    n_top += 1;
                    (pos, size) = (n_top, top);
                }
                Item::Entry(i) => {
                    let e = &list[i];
                    parts.push(e.name.clone());
                    if !e.variants.is_empty() {
                        parts.push(format!("variants: {}", e.variants.len()));
                    }
                    if e.demo {
                        parts.push("has demo".into());
                    }
                    if checked == Some(i) {
                        parts.push("current".into());
                    }
                    level = 1;
                    expanded = (!e.variants.is_empty()).then(|| st.expanded.contains(&i));
                    n_top += 1;
                    (pos, size) = (n_top, top);
                }
                Item::Variant(i, j) => {
                    let e = &list[i];
                    let v = &e.variants[j];
                    parts.push(v.label());
                    if self.preferred_variant(e).is_some_and(|p| p == v) {
                        parts.push("preferred".into());
                    }
                    if v.demo {
                        parts.push("has demo".into());
                    }
                    if active == Some(&v.path) {
                        parts.push("current".into());
                    }
                    (level, expanded) = (2, None);
                    (pos, size) = (j + 1, e.variants.len());
                }
            }
            out.push(RowView {
                item,
                text: parts.join(", "),
                level,
                expanded,
                pos,
                size,
            });
        }
        out
    }

    /// The texts of the items on show, when the dialog is open (for tests).
    pub fn chooser_texts(&self) -> Option<Vec<String>> {
        let st = self.chooser.as_ref()?;
        Some(self.chooser_rows(st).into_iter().map(|r| r.text).collect())
    }

    /// The id of the tree of the open dialog and of an item's node (for tests).
    pub fn chooser_ids(&self, item: Item) -> (Id, Id) {
        (
            list_id(self.chooser_serial),
            item_id(self.chooser_serial, item),
        )
    }

    /// The keys of the dialog, before it draws. Escape closes (`close`); Ctrl+I and Enter ask
    /// for an outcome; Down in the filter moves to the list; the list's keys change `st`.
    fn chooser_keys(&self, ctx: &Context, st: &mut Chooser, close: &mut bool) -> Outcome {
        let serial = self.chooser_serial;
        let (in_filter, in_list) =
            ctx.memory(|m| (m.has_focus(filter_id(serial)), m.has_focus(list_id(serial))));
        let list = self.chooser_list();
        if in_list || in_filter {
            st.list_stop = in_list;
        }
        let mut outcome = Outcome::None;
        // nothing acts on an item the filter hides (with no rows, Enter and Ctrl+I do nothing)
        let shown = visible(list, &st.filter, &st.expanded).contains(&st.cursor);
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            *close = true;
            return outcome;
        }
        if ctx.input_mut(|i| i.consume_shortcut(&shortcuts::CHOOSER_INFO)) && shown {
            outcome = Outcome::Info(st.cursor);
        }
        if !in_filter && !in_list {
            return outcome;
        }
        // the plain key presses of this frame in order (auto-repeat included)
        let pressed: Vec<Key> = ctx.input(|i| {
            i.events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } if !modifiers.any() => Some(*key),
                    _ => None,
                })
                .collect()
        });
        for key in pressed {
            if in_filter {
                match key {
                    Key::ArrowDown => {
                        ctx.memory_mut(|m| m.request_focus(list_id(serial)));
                        ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown));
                    }
                    Key::Enter => {
                        if shown {
                            outcome = Outcome::Choose(st.cursor);
                        }
                        ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
                    }
                    _ => {}
                }
            } else if let Some((_, k)) = ListKey::ALL.iter().find(|(c, _)| *c == key) {
                let rows = visible(list, &st.filter, &st.expanded);
                if let Some(item) = st.key(list, &rows, *k) {
                    outcome = Outcome::Choose(item);
                }
                ctx.input_mut(|i| i.consume_key(Modifiers::NONE, key));
            }
        }
        outcome
    }

    /// Draws the dialog when it is open and takes its keys.
    pub fn chooser_ui(&mut self, ctx: &Context) {
        if self.chooser.is_none() {
            return;
        }
        if !self.screen_reader_mode() {
            self.chooser = None;
            return;
        }
        let Some(mut st) = self.chooser.take() else {
            return;
        };
        let due = self.window_focus_due(Win::Chooser);
        let serial = self.chooser_serial;
        let mut close = false;
        let mut outcome = self.chooser_keys(ctx, &mut st, &mut close);
        let mut open = !close;
        if open {
            let screen = ctx.content_rect();
            let id = window_id(serial);
            let this: &App = self;
            egui::Window::new("Choose Mod")
                .id(id)
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .pivot(Align2::CENTER_CENTER)
                .default_pos(screen.center())
                .fixed_size(vec2(
                    (screen.width() - 40.0).clamp(200.0, 440.0),
                    (screen.height() - 120.0).clamp(150.0, 400.0),
                ))
                .show(ctx, |ui| {
                    win::window(ui, true, id, "Choose Mod");
                    if let Some(o) = this.chooser_contents(ui, &mut st, due) {
                        outcome = o;
                    }
                });
        }
        if !open {
            close = true;
        }
        let list = self.chooser_list();
        let cmd = match outcome {
            Outcome::Choose(item) => choose_cmd(list, item),
            _ => None,
        };
        let info = match outcome {
            Outcome::Info(item) => info_kit(list, item),
            _ => None,
        };
        if cmd.is_some() {
            close = true;
        }
        if !close {
            self.chooser = Some(st);
        }
        if let Some(c) = cmd {
            self.command(c);
        }
        if let Some(k) = info {
            self.show_mod_info_of(k);
        }
    }

    /// The window's contents: the filter, the count, the tree and the hint line. Returns the
    /// outcome of a click.
    fn chooser_contents(&self, ui: &mut Ui, st: &mut Chooser, due: bool) -> Option<Outcome> {
        let serial = self.chooser_serial;
        let ctx = ui.ctx().clone();
        // the focus goes into the dialog: to the filter, or back to the list when that is
        // where it was (Mod Info closed)
        if due && st.list_stop {
            ui.memory_mut(|m| {
                if let Some(f) = m.focused() {
                    m.surrender_focus(f);
                }
                m.request_focus(list_id(serial));
            });
        } else {
            crate::app::focus_into_window(ui, due);
        }
        let mut out = None;
        ui.label(RichText::new("Filter").color(INK_DIM));
        let edit = egui::TextEdit::singleline(&mut st.filter)
            .id(filter_id(serial))
            .hint_text("Type to filter by name")
            .desired_width(f32::INFINITY);
        let r = ui.add(edit);
        ctx.accesskit_node_builder(r.id, |n| n.set_label("Filter mods"));
        let rows = self.chooser_rows(st);
        let items: Vec<Item> = rows.iter().map(|r| r.item).collect();
        if r.changed() {
            st.filtered(&items);
        }
        st.keep_valid(&items);
        let total = self.chooser_list().len() + 1;
        let n = items.len();
        ui.label(
            RichText::new(format!("{n} of {total} mods"))
                .small()
                .color(INK_DIM),
        );
        if let Some(Err(e)) = self.mods() {
            ui.label(RichText::new(e).color(INK_DIM));
        }
        let focused = ctx.memory(|m| m.has_focus(list_id(serial)));
        let scroll_to = (focused && st.scrolled != Some(st.cursor)).then_some(st.cursor);
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 28.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                out = self.chooser_tree(ui, st, &rows, focused, scroll_to);
            });
        if scroll_to.is_some() {
            st.scrolled = scroll_to;
        }
        let hint = format!(
            "Enter chooses, Right opens variants, {} mod info, Escape closes",
            shortcuts::text(&ctx, &shortcuts::CHOOSER_INFO)
        );
        ui.label(RichText::new(hint).small().color(INK_DIM));
        out
    }

    /// The tree: one focus stop (a Tree node) whose active descendant is the selected item;
    /// every item is a TreeItem node, painted when in view.
    fn chooser_tree(
        &self,
        ui: &mut Ui,
        st: &mut Chooser,
        rows: &[RowView],
        focused: bool,
        scroll_to: Option<Item>,
    ) -> Option<Outcome> {
        let serial = self.chooser_serial;
        let lid = list_id(serial);
        let w = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(
            vec2(w, (ROW_H * rows.len() as f32).max(ROW_H)),
            Sense::hover(),
        );
        let g = ui.new_child(
            UiBuilder::new()
                .scope_id(lid)
                .max_rect(rect)
                .sense(Sense::FOCUSABLE)
                .accessibility_role(accesskit::Role::Tree)
                .accessibility_label("Mods"),
        );
        if focused {
            // the arrows are the list's (`App::chooser_keys`), not egui's focus moves
            let filter = egui::EventFilter {
                vertical_arrows: true,
                horizontal_arrows: true,
                ..Default::default()
            };
            g.memory_mut(|m| m.set_focus_lock_filter(lid, filter));
        }
        let ctx = g.ctx().clone();
        ctx.accesskit_node_builder(lid, |n| {
            n.set_bounds(bounds(rect));
            n.set_active_descendant(item_id(serial, st.cursor).accesskit_id());
        });
        let clip = g.clip_rect();
        let mut out = None;
        for (i, row) in rows.iter().enumerate() {
            let top = rect.min.y + ROW_H * i as f32;
            let r = Rect::from_min_size(egui::pos2(rect.min.x, top), vec2(w, ROW_H));
            let iid = item_id(serial, row.item);
            let resp = g.interact(r, iid, Sense::CLICK);
            ctx.accesskit_node_builder(iid, |n| {
                n.set_role(accesskit::Role::TreeItem);
                n.set_label(row.text.clone());
                n.set_level(row.level);
                n.set_position_in_set(row.pos);
                n.set_size_of_set(row.size);
                n.set_selected(row.item == st.cursor);
                if let Some(e) = row.expanded {
                    n.set_expanded(e);
                }
                n.set_bounds(bounds(r));
            });
            if scroll_to == Some(row.item) {
                g.scroll_to_rect(r, None);
            }
            if resp.clicked() || resp.double_clicked() {
                st.cursor = row.item;
                g.memory_mut(|m| m.request_focus(lid));
            }
            if resp.double_clicked() {
                out = Some(Outcome::Choose(row.item));
            }
            if r.intersects(clip) {
                let sel = row.item == st.cursor;
                if sel {
                    let fill = if focused { SLATE_LINE } else { SLATE_DARK };
                    g.painter().rect_filled(r, 2.0, fill);
                    if focused {
                        g.painter().rect_stroke(
                            r,
                            2.0,
                            Stroke::new(1.5, AMBER),
                            egui::StrokeKind::Inside,
                        );
                    }
                }
                let x = r.min.x + 6.0 + 18.0 * (row.level - 1) as f32;
                let mark = match row.expanded {
                    Some(true) => "- ",
                    Some(false) => "+ ",
                    None => "",
                };
                let color: Color32 = if row.level == 2 { INK_DIM } else { INK };
                g.painter().text(
                    egui::pos2(x, r.center().y),
                    Align2::LEFT_CENTER,
                    format!("{mark}{}", row.text),
                    FontId::proportional(13.0),
                    color,
                );
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::ModVariant;
    use std::path::PathBuf;

    fn entry(name: &str, variants: usize) -> ModEntry {
        let v = |j: usize| ModVariant {
            path: PathBuf::from(format!("{name}{j}.rbm")),
            demo: false,
            version: String::new(),
        };
        ModEntry {
            path: PathBuf::from(format!("{name}0.rbm")),
            name: name.into(),
            variants: if variants > 1 {
                (0..variants).map(v).collect()
            } else {
                vec![]
            },
            demo: false,
        }
    }

    #[test]
    fn the_filter_ignores_ascii_case_and_matches_anywhere() {
        assert!(name_matches("Dejenerate", "jen"));
        assert!(name_matches("Dejenerate", "DEJ"));
        assert!(name_matches("anything", ""));
        assert!(!name_matches("Dejenerate", "x"));
        // no Unicode folding, as the menu's order
        assert!(!name_matches("\u{c9}cole", "\u{e9}"));
    }

    #[test]
    fn the_rows_are_standard_then_the_entries_with_open_variants() {
        let list = vec![entry("Alpha", 0), entry("Beta", 3), entry("Gamma", 0)];
        let none = BTreeSet::new();
        assert_eq!(
            visible(&list, "", &none),
            [
                Item::Standard,
                Item::Entry(0),
                Item::Entry(1),
                Item::Entry(2)
            ]
        );
        let open: BTreeSet<usize> = [1].into();
        assert_eq!(visible(&list, "", &open).len(), 7);
        assert_eq!(
            visible(&list, "ta", &open)[1..],
            [
                Item::Entry(1),
                Item::Variant(1, 0),
                Item::Variant(1, 1),
                Item::Variant(1, 2)
            ]
        );
        // "standard rebirth" matches itself
        assert_eq!(visible(&list, "stand", &none), [Item::Standard]);
    }

    #[test]
    fn the_keys_walk_and_open_the_tree() {
        let list = vec![entry("Alpha", 0), entry("Beta", 2), entry("Gamma", 0)];
        let mut st = Chooser::new(Item::Standard);
        let rows = |st: &Chooser| visible(&list, &st.filter, &st.expanded);
        st.key(&list, &rows(&st), ListKey::Down);
        assert_eq!(st.cursor, Item::Entry(0));
        st.key(&list, &rows(&st), ListKey::End);
        assert_eq!(st.cursor, Item::Entry(2));
        st.key(&list, &rows(&st), ListKey::Down);
        assert_eq!(st.cursor, Item::Entry(2), "stops at the end");
        st.key(&list, &rows(&st), ListKey::Home);
        assert_eq!(st.cursor, Item::Standard);
        st.key(&list, &rows(&st), ListKey::Up);
        assert_eq!(st.cursor, Item::Standard);
        // Right on an entry without variants does nothing
        st.key(&list, &rows(&st), ListKey::Right);
        assert!(st.expanded.is_empty());
        st.cursor = Item::Entry(1);
        st.key(&list, &rows(&st), ListKey::Right);
        assert!(st.expanded.contains(&1));
        assert_eq!(st.cursor, Item::Entry(1));
        st.key(&list, &rows(&st), ListKey::Right);
        assert_eq!(st.cursor, Item::Variant(1, 0));
        st.key(&list, &rows(&st), ListKey::Down);
        assert_eq!(st.cursor, Item::Variant(1, 1));
        st.key(&list, &rows(&st), ListKey::Left);
        assert_eq!(st.cursor, Item::Entry(1));
        st.key(&list, &rows(&st), ListKey::Left);
        assert!(st.expanded.is_empty());
        assert_eq!(
            st.key(&list, &rows(&st), ListKey::Enter),
            Some(Item::Entry(1))
        );
    }

    #[test]
    fn choosing_is_the_menus_command() {
        let list = vec![entry("Alpha", 0), entry("Beta", 2)];
        assert_eq!(
            choose_cmd(&list, Item::Standard),
            Some(Cmd::UseKit(Kit::Standard))
        );
        assert_eq!(
            choose_cmd(&list, Item::Entry(1)),
            Some(Cmd::UseKit(Kit::Mod("Beta0.rbm".into())))
        );
        assert_eq!(
            choose_cmd(&list, Item::Variant(1, 1)),
            Some(Cmd::ChooseVariant("Beta1.rbm".into()))
        );
        assert_eq!(choose_cmd(&list, Item::Entry(9)), None);
    }
}
