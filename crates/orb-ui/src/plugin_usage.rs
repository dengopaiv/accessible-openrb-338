//! Help > Plugin Usage (task-221): the in-app guide to the plugin's modes, channels and MIDI
//! mappings, in a scrollable window of the plugin's editor (the standalone app has no such
//! item).
//!
//! The text is `docs/plugin_usage.md`, embedded and drawn from a small subset of Markdown
//! ([`parse`]): the page and the file cannot differ, because the page is the file.
//! The subset is headings (`#`, `##`, `###`), paragraphs, bullets (`- `), numbered items
//! (`1. `), tables, the inline `**bold**` and `` `code` ``, and a `<picture>` block, which
//! stands for Steinberg's VST Compatible Logo ([`LOGO_DARK`], the variant for the window's dark
//! background). The logo is drawn unaltered: no tint, no opacity, no effect, at a size above 15 x
//! 10 mm (57 x 38 px at 1x; Steinberg's guidelines, assets/README.md).

use egui::text::{LayoutJob, TextFormat};
use egui::{self, Color32, Context, FontFamily, FontId, Key, Modifiers, RichText};

use crate::app::App;
use crate::palette::{AMBER, INK, INK_DIM};

/// The page, `docs/plugin_usage.md`.
pub const TEXT: &str = include_str!("../../../docs/plugin_usage.md");
/// The official logo for a dark background (white box), Steinberg's file, unaltered.
pub const LOGO_DARK: &[u8] =
    include_bytes!("../../../assets/steinberg/VST_Compatible_Logo_Steinberg_negative.svg");
/// The loader URI of the logo.
const LOGO_URI: &str = "bytes://orb-ui/plugin_usage_vst_logo.svg";
/// The logo's size in points: the SVG's viewBox is 116.2 x 74.7, and the width is well above
/// the 57 px (15 mm at 96 dpi) of the guidelines' minimum at 1x.
pub const LOGO_W: f32 = 96.0;
pub const LOGO_H: f32 = LOGO_W * 74.7 / 116.2;
/// The sentence Steinberg's guidelines ask for in product documentation (section 15).
pub const TRADEMARK: &str = "VST is a registered trademark of Steinberg Media Technologies GmbH.";

/// One block of the page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    /// A heading: its level (1..3) and text.
    Heading(u8, String),
    /// A paragraph, its lines joined.
    Para(String),
    /// A list: numbered or not, one text per item.
    List(bool, Vec<String>),
    /// A table: the header cells and the rows.
    Table(Vec<String>, Vec<Vec<String>>),
    /// The VST Compatible Logo.
    Logo,
}

fn cells(line: &str) -> Vec<String> {
    line.trim()
        .trim_start_matches('|')
        .trim_end_matches('|')
        .split('|')
        .map(|c| c.trim().to_owned())
        .collect()
}

/// A numbered item's text after its `N. ` prefix, if the line has one.
fn numbered(line: &str) -> Option<&str> {
    let (n, rest) = line.split_once(". ")?;
    (!n.is_empty() && n.bytes().all(|b| b.is_ascii_digit())).then_some(rest)
}

/// Parses the page's Markdown subset.
pub fn parse(text: &str) -> Vec<Block> {
    let mut out = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let l = lines[i];
        if l.trim().is_empty() {
            i += 1;
        } else if l.starts_with("<picture>") {
            while i < lines.len() && !lines[i].starts_with("</picture>") {
                i += 1;
            }
            i += 1;
            out.push(Block::Logo);
        } else if l.starts_with('<') {
            // any other markup line (a stray `</picture>`): not text of the page
            i += 1;
        } else if let Some(h) = l.strip_prefix('#') {
            let extra = h.bytes().take_while(|&b| b == b'#').count();
            out.push(Block::Heading(
                extra as u8 + 1,
                h[extra..].trim().to_owned(),
            ));
            i += 1;
        } else if l.starts_with('|') {
            let head = cells(l);
            i += 2; // the separator row
            let mut rows = Vec::new();
            while i < lines.len() && lines[i].starts_with('|') {
                rows.push(cells(lines[i]));
                i += 1;
            }
            out.push(Block::Table(head, rows));
        } else if l.starts_with("- ") || numbered(l).is_some() {
            let ordered = numbered(l).is_some();
            let mut items = Vec::new();
            while i < lines.len() {
                let m = lines[i];
                let item = if ordered {
                    numbered(m)
                } else {
                    m.strip_prefix("- ")
                };
                match item {
                    Some(t) => items.push(t.trim().to_owned()),
                    None => break,
                }
                i += 1;
            }
            out.push(Block::List(ordered, items));
        } else {
            let mut para = String::new();
            while i < lines.len() {
                let m = lines[i];
                if m.trim().is_empty()
                    || m.starts_with(['#', '|', '<'])
                    || m.starts_with("- ")
                    || numbered(m).is_some()
                {
                    break;
                }
                if !para.is_empty() {
                    para.push(' ');
                }
                para.push_str(m.trim());
                i += 1;
            }
            out.push(Block::Para(para));
        }
    }
    out
}

/// The spans of a text with `**bold**` and `` `code` ``: (text, bold, code).
pub fn spans(text: &str) -> Vec<(String, bool, bool)> {
    let mut out: Vec<(String, bool, bool)> = Vec::new();
    let (mut bold, mut code) = (false, false);
    let mut cur = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        let toggle = if c == '*' && chars.peek() == Some(&'*') && !code {
            chars.next();
            Some(true)
        } else if c == '`' {
            Some(false)
        } else {
            None
        };
        match toggle {
            Some(is_bold) => {
                if !cur.is_empty() {
                    out.push((std::mem::take(&mut cur), bold, code));
                }
                if is_bold {
                    bold = !bold;
                } else {
                    code = !code;
                }
            }
            None => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push((cur, bold, code));
    }
    out
}

/// The text of the spans without the markup (what the page shows).
pub fn plain(text: &str) -> String {
    spans(text).into_iter().map(|s| s.0).collect()
}

fn job(ui: &egui::Ui, text: &str, color: Color32, bold_color: Color32) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = ui.available_width();
    for (t, bold, code) in spans(text) {
        let family = if code {
            FontFamily::Monospace
        } else {
            FontFamily::Proportional
        };
        let color = if bold { bold_color } else { color };
        job.append(
            &t,
            0.0,
            TextFormat {
                font_id: FontId::new(13.5, family),
                color,
                ..Default::default()
            },
        );
    }
    job
}

fn text(ui: &mut egui::Ui, t: &str) {
    let j = job(ui, t, INK, AMBER);
    ui.add(egui::Label::new(j).wrap());
}

fn draw(ui: &mut egui::Ui, blocks: &[Block]) {
    for (n, b) in blocks.iter().enumerate() {
        match b {
            // the window's title says it
            Block::Heading(1, _) => {}
            Block::Heading(level, t) => {
                ui.add_space(if *level == 2 { 12.0 } else { 8.0 });
                let size = if *level == 2 { 16.0 } else { 14.5 };
                ui.label(RichText::new(plain(t)).strong().size(size).color(AMBER));
                ui.add_space(2.0);
            }
            Block::Para(t) => {
                ui.add_space(4.0);
                text(ui, t);
            }
            Block::List(ordered, items) => {
                ui.add_space(3.0);
                for (k, item) in items.iter().enumerate() {
                    ui.horizontal_top(|ui| {
                        let mark = if *ordered {
                            format!("{}.", k + 1)
                        } else {
                            "\u{2022}".to_owned()
                        };
                        let mark = egui::Label::new(RichText::new(mark).color(INK_DIM));
                        ui.add_sized([18.0, 16.0], mark);
                        text(ui, item);
                    });
                }
            }
            Block::Table(head, rows) => {
                ui.add_space(4.0);
                egui::Grid::new(("orb_plugin_usage_table", n))
                    .num_columns(head.len())
                    .striped(true)
                    .spacing([14.0, 3.0])
                    .show(ui, |ui| {
                        for h in head {
                            ui.label(RichText::new(plain(h)).strong().color(INK_DIM));
                        }
                        ui.end_row();
                        for r in rows {
                            for c in r {
                                ui.label(RichText::new(plain(c)).color(INK));
                            }
                            ui.end_row();
                        }
                    });
            }
            Block::Logo => {
                ui.add_space(8.0);
                // unaltered: the file's own colors, no tint
                ui.add(
                    egui::Image::from_bytes(LOGO_URI, LOGO_DARK)
                        .fit_to_exact_size(egui::vec2(LOGO_W, LOGO_H)),
                );
            }
        }
    }
}

/// The id of the window of an open (`App::plugin_usage_serial`).
pub fn window_id(serial: u32) -> egui::Id {
    egui::Id::unique(("orb_plugin_usage_window", serial))
}

impl App {
    /// Help > Plugin Usage.
    pub fn show_plugin_usage(&mut self) {
        if !self.plugin_usage_open {
            // a new id: no stored place, so the window opens at its default, the center
            self.plugin_usage_serial += 1;
        } else {
            self.raise_window(self.plugin_usage_id());
        }
        self.plugin_usage_open = true;
    }

    /// The id of the window now (for tests, to look its place up).
    pub fn plugin_usage_id(&self) -> egui::Id {
        window_id(self.plugin_usage_serial)
    }

    /// Whether the window is open.
    pub fn plugin_usage_is_open(&self) -> bool {
        self.plugin_usage_open
    }

    /// Draws the window when it is open. The close button closes it, and Escape, as the other
    /// windows (the plugin's window passes keys to the host, so the mouse is the way).
    pub fn plugin_usage_ui(&mut self, ctx: &Context) {
        if !self.plugin_usage_open {
            return;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            self.plugin_usage_open = false;
            return;
        }
        let mut open = true;
        let screen = ctx.content_rect();
        let size = egui::vec2(
            (screen.width() - 24.0).clamp(200.0, 580.0),
            (screen.height() - 40.0).clamp(150.0, 520.0),
        );
        let blocks = ctx.memory_mut(|m| {
            m.data
                .get_temp_mut_or_insert_with(egui::Id::unique("orb_plugin_usage_blocks"), || {
                    std::sync::Arc::new(parse(TEXT))
                })
                .clone()
        });
        // centered the way File > Info is (info.rs): pivot, default place, an id per open
        egui::Window::new("Plugin Usage")
            .id(self.plugin_usage_id())
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(screen.center())
            .default_size(size)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| draw(ui, &blocks));
            });
        if !open {
            self.plugin_usage_open = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The text of the page.
    fn all_text() -> String {
        let mut s = String::new();
        for b in parse(TEXT) {
            match b {
                Block::Heading(_, t) | Block::Para(t) => s += &plain(&t),
                Block::List(_, v) => v.iter().for_each(|t| s += &plain(t)),
                Block::Table(h, r) => {
                    h.iter().for_each(|t| s += &plain(t));
                    r.iter().flatten().for_each(|t| s += &plain(t));
                }
                Block::Logo => {}
            }
            s.push('\n');
        }
        s
    }

    /// The page is the file: every word of the file's text lines is on the page.
    #[test]
    fn the_page_shows_the_whole_file() {
        let blocks = parse(TEXT);
        assert!(blocks.len() > 30, "{} blocks", blocks.len());
        let shown = all_text();
        for line in TEXT.lines() {
            if line.trim_start().starts_with(['<', '|']) {
                continue;
            }
            let body = line.trim_start_matches('#').trim();
            let body = body
                .strip_prefix("- ")
                .or_else(|| numbered(body))
                .unwrap_or(body);
            for w in plain(body).split_whitespace() {
                assert!(shown.contains(w), "{w:?} of {line:?} is not on the page");
            }
        }
        assert_eq!(blocks.iter().filter(|b| **b == Block::Logo).count(), 1);
        let tables = blocks.iter().filter(|b| matches!(b, Block::Table(..)));
        // the channels, the banks, the drum notes, the outputs (task-256)
        assert_eq!(tables.count(), 4);
    }

    /// Every list item of the file starts its own line (a glued `...mode.- **Keys.**` showed a
    /// stray hyphen and lost an item), and the page has exactly one item per such line.
    #[test]
    fn every_list_item_is_a_line_of_its_own() {
        let items = TEXT
            .lines()
            .filter(|l| l.starts_with("- ") || numbered(l).is_some())
            .count();
        let drawn: usize = parse(TEXT)
            .iter()
            .map(|b| {
                if let Block::List(_, v) = b {
                    v.len()
                } else {
                    0
                }
            })
            .sum();
        assert!(items > 30, "{items} items");
        assert_eq!(drawn, items);
        for l in TEXT.lines() {
            let body = l.strip_prefix("- ").unwrap_or(l);
            assert!(
                !body.contains(".- ") && !body.contains(". - **"),
                "a glued item in {l:?}"
            );
        }
    }

    /// The page ends with the logo and the sentence.
    #[test]
    fn the_page_ends_with_the_logo_and_the_sentence() {
        let blocks = parse(TEXT);
        let n = blocks.len();
        assert_eq!(blocks[n - 2], Block::Logo);
        assert_eq!(blocks[n - 1], Block::Para(TRADEMARK.into()));
    }

    /// Steinberg's terms (assets/README.md, task-233): the sentence is on the page, the first
    /// "VST" of the page carries the registered sign, and the page shows the official logo file
    /// of the dark variant, unaltered and big enough.
    #[test]
    fn the_page_carries_the_trademark_terms() {
        let shown = all_text();
        assert!(shown.contains(TRADEMARK), "the trademark sentence");
        let first = shown.find("VST").expect("VST on the page");
        assert!(
            shown[first..].starts_with("VST\u{ae}"),
            "the registered sign at the first VST"
        );
        let svg = std::str::from_utf8(LOGO_DARK).unwrap();
        assert!(svg.contains("viewBox=\"0 0 116.2 74.7\""));
        assert!(
            svg.contains("fill:#FFFFFF"),
            "the negative variant, white box"
        );
        // 15 x 10 mm at 96 dpi is 56.7 x 37.8 px, at 1x
        const { assert!(LOGO_W >= 57.0 && LOGO_H >= 38.0) };
        assert!(
            (LOGO_W / LOGO_H - 116.2 / 74.7).abs() < 1e-3,
            "the aspect ratio"
        );
        // the file names both of Steinberg's variants, as the page's picture does
        for f in [
            "VST_Compatible_Logo_Steinberg.svg",
            "VST_Compatible_Logo_Steinberg_negative.svg",
        ] {
            assert!(TEXT.contains(f), "{f} in docs/plugin_usage.md");
        }
    }

    /// `parse` terminates, with a bounded number of blocks, on odd input: a stray markup line,
    /// an unclosed `<picture>`, a table cut short, empty cells and headings, no final newline.
    #[test]
    fn parse_terminates_on_odd_input() {
        for text in [
            "</picture>",
            "text\n</picture>\ntext",
            "<div>\n<b>x</b>",
            "<picture>\n<img src=\"a\">",
            "|",
            "| a |",
            "| a |\n|---|",
            "|\n|\n|",
            "#",
            "####",
            "- ",
            "1. ",
            "1.",
            "-",
            "a\n<\nb",
            "\n\n\n",
            "",
            "**",
            "`",
        ] {
            let blocks = parse(text);
            assert!(
                blocks.len() <= text.lines().count() + 1,
                "{text:?}: {blocks:?}"
            );
            let _ = spans(text);
        }
        assert_eq!(
            parse("a\n</picture>\nb"),
            vec![Block::Para("a".into()), Block::Para("b".into())]
        );
    }

    /// The README links the page.
    #[test]
    fn the_readme_links_the_page() {
        let readme = include_str!("../../../README.md");
        assert!(readme.contains("docs/plugin_usage.md"));
    }

    #[test]
    fn the_inline_markup_is_cut_out() {
        assert_eq!(
            spans("a **b** `c` d"),
            vec![
                ("a ".to_owned(), false, false),
                ("b".to_owned(), true, false),
                (" ".to_owned(), false, false),
                ("c".to_owned(), false, true),
                (" d".to_owned(), false, false),
            ]
        );
        assert_eq!(plain("**x**: 1 * 2"), "x: 1 * 2");
    }

    /// The page covers its subjects (task-221's list); the facts themselves were checked
    /// against the code and docs/plugin.md, docs/midi.md (the task's notes).
    #[test]
    fn the_page_covers_its_subjects() {
        let shown = all_text().to_lowercase();
        for s in [
            "latch pattern",
            "gate pattern",
            "note mode",
            "preview",
            "channel",
            "0 is a1",
            "concert pitch",
            "accent",
            "slide",
            "tie",
            "general midi",
            "112 to 127",
            "host parameter",
            "64 samples",
            "drag a pattern out",
            "recipes",
        ] {
            assert!(shown.contains(s), "{s:?} missing from the page");
        }
        // the first release has no remote channel (the user, 2026-11-16)
        for gone in ["standard mapping", "channel 16", "remote"] {
            assert!(!shown.contains(gone), "{gone:?} on the page");
        }
    }
}
