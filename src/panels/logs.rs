use crate::{common::Action, log::LOGGER, panels::Show};
use eframe::egui::{
    Align, Color32, Context, Label, Layout, RichText, ScrollArea, TextWrapMode, TopBottomPanel,
};

pub struct LogsPanel;

impl Show for LogsPanel {
    fn show(&mut self, ctx: &Context, _action: &mut Option<Action>) {
        TopBottomPanel::bottom("logs")
            .resizable(true)
            .show(ctx, |ui| {
                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    if ui.button("clear").clicked() {
                        LOGGER.clear();
                    }
                });

                ScrollArea::both().show(ui, |ui| {
                    ui.take_available_space();

                    for entry in LOGGER.read().iter() {
                        let colour = match entry.level {
                            log::Level::Error => Color32::RED,
                            log::Level::Warn => Color32::YELLOW,
                            log::Level::Info => Color32::WHITE,
                            log::Level::Debug | log::Level::Trace => ui.visuals().text_color(),
                        };

                        ui.add(
                            Label::new(RichText::new(entry).monospace().color(colour))
                                .wrap_mode(TextWrapMode::Extend),
                        );
                    }
                });
            });
    }
}
