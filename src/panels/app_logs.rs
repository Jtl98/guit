use crate::{common::Action, log::LOGGER, panels::Show};
use eframe::egui::{Align, Color32, Label, Layout, Panel, RichText, ScrollArea, TextWrapMode, Ui};

pub struct AppLogsPanel;

impl Show for AppLogsPanel {
    fn show(&mut self, ui: &mut Ui, _action: &mut Option<Action>) {
        Panel::bottom("logs").resizable(true).show_inside(ui, |ui| {
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
