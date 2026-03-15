use crate::{
    common::{Action, Diff},
    panels::Show,
};
use eframe::egui::{CentralPanel, Color32, Context, Label, RichText, ScrollArea, TextWrapMode};

pub struct DiffPanel<'a> {
    diff: &'a Diff,
}

impl<'a> DiffPanel<'a> {
    pub fn new(diff: &'a Diff) -> Self {
        Self { diff }
    }
}

impl<'a> Show for DiffPanel<'a> {
    fn show(&mut self, ctx: &Context, _action: &mut Option<Action>) {
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.colored_label(Color32::GREEN, &self.diff.numstat.additions);
                ui.colored_label(Color32::RED, &self.diff.numstat.deletions);
            });

            ui.separator();

            ScrollArea::both().show(ui, |ui| {
                ui.take_available_space();

                for line in self.diff.content.lines() {
                    let colour = if line.starts_with('+') {
                        Color32::GREEN
                    } else if line.starts_with('-') {
                        Color32::RED
                    } else {
                        ui.visuals().text_color()
                    };

                    ui.add(
                        Label::new(RichText::new(line).monospace().color(colour))
                            .wrap_mode(TextWrapMode::Extend),
                    );
                }
            });
        });
    }
}
