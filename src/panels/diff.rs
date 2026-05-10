use crate::{
    common::{Action, Diff, DiffNumstat, HunkDiff, LineType, StringDiff},
    panels::Show,
};
use eframe::egui::{CentralPanel, Color32, Label, RichText, ScrollArea, TextWrapMode, Ui};

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
            let total_lines = diff.lines.len().to_string();

            ui.label(total_lines);
        });

        ui.separator();

        ScrollArea::both().show(ui, |ui| {
            ui.take_available_space();

            for line in &diff.lines {
                let text = RichText::new(line).monospace();
                let label = Label::new(text).wrap_mode(TextWrapMode::Extend);

                ui.add(label);
            }
        });
    }

    fn show_hunk(&self, ui: &mut Ui, diff: &HunkDiff) {
        ui.horizontal_wrapped(|ui| {
            let DiffNumstat {
                additions,
                deletions,
            } = &diff.numstat;

            ui.colored_label(Color32::GREEN, additions);
            ui.colored_label(Color32::RED, deletions);
        });

        ui.separator();

        ScrollArea::both().show(ui, |ui| {
            ui.take_available_space();

            for hunk in &diff.hunks {
                let header_text = RichText::new(&hunk.header).monospace().strong();
                let header_label = Label::new(header_text).wrap_mode(TextWrapMode::Extend);

                ui.add(header_label);

                for line in &hunk.lines {
                    let line_colour = match line.line_type {
                        LineType::Addition => Color32::GREEN,
                        LineType::Deletion => Color32::RED,
                        LineType::Context => ui.visuals().text_color(),
                    };
                    let line_text = RichText::new(&line.content).monospace().color(line_colour);
                    let line_label = Label::new(line_text).wrap_mode(TextWrapMode::Extend);

                    ui.add(line_label);
                }
            }
        });
    }
}

impl<'a> Show for DiffPanel<'a> {
    fn show(&mut self, ui: &mut Ui, _action: &mut Option<Action>) {
        CentralPanel::default().show_inside(ui, |ui| match self.diff {
            Diff::Binary => self.show_binary(ui),
            Diff::String(diff) => self.show_string(ui, diff),
            Diff::Hunk(diff) => self.show_hunk(ui, diff),
        });
    }
}
