use crate::{
    common::{Action, Diff, StringDiff},
    panels::Show,
};
use eframe::egui::{CentralPanel, Color32, Context, Label, RichText, ScrollArea, TextWrapMode, Ui};

pub struct DiffPanel<'a> {
    diff: &'a Diff,
}

impl<'a> DiffPanel<'a> {
    pub fn new(diff: &'a Diff) -> Self {
        Self { diff }
    }

    fn show_binary(&self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label("binary file");
        });

        ui.separator();
    }

    fn show_string(&self, ui: &mut Ui, diff: &StringDiff) {
        ui.horizontal_wrapped(|ui| {
            ui.colored_label(Color32::GREEN, &diff.numstat.additions);
            ui.colored_label(Color32::RED, &diff.numstat.deletions);
        });

        ui.separator();

        ScrollArea::both().show(ui, |ui| {
            ui.take_available_space();

            for line in diff.content.lines() {
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
    }
}

impl<'a> Show for DiffPanel<'a> {
    fn show(&mut self, ctx: &Context, _action: &mut Option<Action>) {
        CentralPanel::default().show(ctx, |ui| match self.diff {
            Diff::Binary => self.show_binary(ui),
            Diff::String(diff) => self.show_string(ui, diff),
        });
    }
}
