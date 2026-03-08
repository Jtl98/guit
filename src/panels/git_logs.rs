use crate::{
    common::{Action, Logs},
    panels::Show,
};
use eframe::egui::{CentralPanel, Context, Label, RichText, ScrollArea};
use std::cmp::Reverse;

pub struct GitLogs<'a> {
    logs: &'a Logs,
}

impl<'a> GitLogs<'a> {
    pub fn new(logs: &'a Logs) -> Self {
        Self { logs }
    }
}

impl<'a> Show for GitLogs<'a> {
    fn show(&mut self, ctx: &Context, _action: &mut Option<Action>) {
        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::both().show(ui, |ui| {
                ui.take_available_space();

                for (Reverse(date), logs) in self.logs {
                    ui.add(Label::new(RichText::new(date).monospace().strong()).extend());

                    for log in logs {
                        let formatted = format!("[{}] {}", log.short_hash, log.subject);

                        ui.add(Label::new(RichText::new(formatted).monospace()).extend())
                            .on_hover_ui(|ui| {
                                ui.style_mut().interaction.selectable_labels = true;

                                let tooltip = format!(
                                    "author: {}\ndate: {}\nhash: {}",
                                    log.author, log.long_date, log.long_hash
                                );
                                ui.label(tooltip);
                            });
                    }
                }
            });
        });
    }
}
