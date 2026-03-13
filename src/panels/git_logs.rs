use crate::{
    common::{Action, DatedLogs, RepoAction::LoadLogs},
    panels::Show,
};
use eframe::egui::{
    CentralPanel, Context, Label, RichText, ScrollArea,
    scroll_area::{ScrollAreaOutput, State},
};
use std::cmp::Reverse;

pub struct GitLogs<'a> {
    dated_logs: &'a DatedLogs,
    logs_scroll_threshold: &'a mut f32,
}

impl<'a> GitLogs<'a> {
    const SCROLL_THRESHOLD: f32 = 100.0;

    pub fn new(dated_logs: &'a DatedLogs, logs_scroll_threshold: &'a mut f32) -> Self {
        Self {
            dated_logs,
            logs_scroll_threshold,
        }
    }
}

impl<'a> Show for GitLogs<'a> {
    fn show(&mut self, ctx: &Context, action: &mut Option<Action>) {
        CentralPanel::default().show(ctx, |ui| {
            let output = ScrollArea::both().show(ui, |ui| {
                ui.take_available_space();

                for (Reverse(date), logs) in self.dated_logs {
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

            let ScrollAreaOutput {
                content_size,
                inner_rect,
                state,
                ..
            } = output;
            let State { offset, .. } = state;
            let end_threshold = content_size.y + inner_rect.min.y - inner_rect.max.y;
            let custom_threshold = end_threshold - Self::SCROLL_THRESHOLD;

            if offset.y > *self.logs_scroll_threshold && offset.y > custom_threshold {
                *action = Some(Action::Repo(LoadLogs));
                // ensures you need to scroll past the current height of the scroll area (i.e. after more logs load)
                // prevents executing git log when no more are available
                *self.logs_scroll_threshold = end_threshold;
            }
        });
    }
}
