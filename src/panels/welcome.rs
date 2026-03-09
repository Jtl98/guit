use crate::{
    common::{
        Action,
        FileAction::{Init, Open, OpenRecent},
    },
    config::{Config, RecentRepo},
    panels::Show,
};
use eframe::egui::{CentralPanel, Context, RichText, Vec2};

pub struct WelcomePanel<'a> {
    config: &'a Config,
}

impl<'a> WelcomePanel<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }
}

impl<'a> Show for WelcomePanel<'a> {
    fn show(&mut self, ctx: &Context, action: &mut Option<Action>) {
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(32.0);
                ui.spacing_mut().button_padding = Vec2::new(16.0, 8.0);

                if ui.button(RichText::new("init").size(32.0)).clicked() {
                    *action = Some(Action::File(Init));
                }

                ui.add_space(16.0);

                if ui.button(RichText::new("open").size(32.0)).clicked() {
                    *action = Some(Action::File(Open));
                }

                ui.add_space(32.0);
                ui.spacing_mut().button_padding = Vec2::new(8.0, 4.0);

                ui.label(RichText::new("recent").size(32.0));
                for RecentRepo { path, .. } in self.config.recent_repos() {
                    ui.add_space(8.0);

                    let text = RichText::new(path.to_string_lossy()).size(16.0);
                    if ui.button(text).clicked() {
                        *action = Some(Action::File(OpenRecent(path.clone())))
                    }
                }
            });
        });
    }
}
