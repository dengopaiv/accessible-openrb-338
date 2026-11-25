//! Options > Output Routing... (task-300, a plugin's editor only; `docs/plugin.md` section 13):
//! which of the plugin's aux outputs also go to Mix-L/R (the four devices) and to their drum
//! machine's bus (each instrument), as the original's ReWire channels do
//! (`docs/rewire.md` section 4). The boxes follow the drum machines' output tables
//! ([`orb_engine::tr808::OUTPUTS`], [`orb_engine::tr909::OUTPUTS`]); each shows its output's
//! name, and its tooltip the output's VST3 bus index, "#N" (Mix is bus 0, so 303-1 is #1 and
//! 909 RC #26; the user's review of 2026-11-25: REAPER numbers its pins flat from 1, so a
//! number on the label did not match REAPER's, and other hosts show the names).

use egui::{self, Context, Key, Modifiers, RichText};
use orb_engine::frame::{OutputRouting, TAP_808, TAP_909};
use orb_engine::{tr808, tr909};

use crate::app::App;
use crate::palette::INK_DIM;

/// The window's title and its menu item's text (without the dots).
pub const TITLE: &str = "Output Routing";

/// The per-device outputs' names, the mixer's channels 0 to 3: outputs (1) to (4).
pub const DEVICES: [&str; 4] = ["303-1", "303-2", "808", "909"];

/// One column of the window: its heading and its boxes, each the output's VST3 bus index (from
/// 1, after the Mix's 0) and its name.
pub struct Column {
    pub heading: &'static str,
    pub boxes: Vec<(usize, &'static str)>,
}

/// The window's three columns: To Mix-L/R, To 808 and To 909.
pub fn columns() -> [Column; 3] {
    let drums = |first: usize, outs: &[orb_engine::drums::DrumOutput]| {
        outs.iter()
            .enumerate()
            .map(|(i, o)| (first + i + 1, o.name))
            .collect()
    };
    [
        Column {
            heading: "To Mix-L/R",
            boxes: DEVICES
                .iter()
                .enumerate()
                .map(|(i, &n)| (i + 1, n))
                .collect(),
        },
        Column {
            heading: "To 808",
            boxes: drums(TAP_808, &tr808::OUTPUTS),
        },
        Column {
            heading: "To 909",
            boxes: drums(TAP_909, &tr909::OUTPUTS),
        },
    ]
}

/// A box's tooltip: its output's VST3 bus index, "#7" for the 808's LT.
pub fn tooltip(number: usize) -> String {
    format!("#{number}")
}

/// The id of the window of an open (`App::routing_serial`).
pub fn window_id(serial: u32) -> egui::Id {
    egui::Id::unique(("orb_output_routing_window", serial))
}

impl App {
    /// Options > Output Routing... (a plugin's editor that has a routing).
    pub fn show_routing(&mut self) {
        if !self.has_routing() {
            return;
        }
        if !self.routing_open {
            // a new id: no stored place, so the window opens at its default, the center
            self.routing_serial += 1;
        } else {
            self.raise_window(self.routing_id());
        }
        self.routing_open = true;
    }

    /// Whether the plugin has an output routing ([`crate::embed::Embed::routing`]).
    pub fn has_routing(&self) -> bool {
        self.plugin
            .as_ref()
            .is_some_and(|p| p.embed.routing().is_some())
    }

    /// The id of the window now (for tests, to look its place up).
    pub fn routing_id(&self) -> egui::Id {
        window_id(self.routing_serial)
    }

    /// Whether the window is open.
    pub fn routing_is_open(&self) -> bool {
        self.routing_open
    }

    /// Draws the window when it is open. A click on a box sets the plugin's routing at once;
    /// the close button and Escape close the window.
    pub fn routing_ui(&mut self, ctx: &Context) {
        if !self.routing_open {
            return;
        }
        let Some(current) = self.plugin.as_ref().and_then(|p| p.embed.routing()) else {
            self.routing_open = false;
            return;
        };
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            self.routing_open = false;
            return;
        }
        let mut bits = current.to_bits();
        let mut open = true;
        let screen = ctx.content_rect();
        egui::Window::new(TITLE)
            .id(self.routing_id())
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(screen.center())
            // the plugin's smallest window (536 x 485) holds it; a smaller host area scrolls
            .max_height((screen.height() - 40.0).max(120.0))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // two short lines: an auto-sized window would stretch to a long one
                    for line in [
                        "Each box is one of the plugin's outputs.",
                        "Unchecked, an output leaves the Mix or its machine.",
                    ] {
                        ui.label(RichText::new(line).color(INK_DIM));
                    }
                    ui.add_space(6.0);
                    ui.horizontal_top(|ui| {
                        for col in columns() {
                            ui.vertical(|ui| {
                                ui.set_min_width(110.0);
                                ui.strong(col.heading);
                                for (n, name) in col.boxes {
                                    let k = n - 1;
                                    let mut on = bits & (1 << k) != 0;
                                    let r = ui.checkbox(&mut on, name).on_hover_text(tooltip(n));
                                    if r.changed() {
                                        bits ^= 1 << k;
                                    }
                                }
                            });
                        }
                    });
                });
            });
        if bits != current.to_bits()
            && let Some(p) = &mut self.plugin
        {
            p.embed.set_routing(OutputRouting::from_bits(bits));
        }
        if !open {
            self.routing_open = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_columns_number_every_output_once_in_bus_order() {
        let cols = columns();
        let n: Vec<usize> = cols
            .iter()
            .flat_map(|c| c.boxes.iter().map(|b| b.0))
            .collect();
        assert_eq!(n, (1..=26).collect::<Vec<_>>());
        assert_eq!(
            cols.iter().map(|c| c.heading).collect::<Vec<_>>(),
            ["To Mix-L/R", "To 808", "To 909"]
        );
        assert_eq!(cols[1].boxes[2], (7, "LT"));
        assert_eq!(tooltip(cols[1].boxes[2].0), "#7");
        assert_eq!(cols[2].boxes[10], (26, "RC"));
        assert_eq!(tooltip(cols[2].boxes[10].0), "#26");
        assert_eq!(cols[0].boxes[0], (1, "303-1"));
    }
}
