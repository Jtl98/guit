use crate::{
    common::{
        Action,
        FileAction::{Open, OpenRecent},
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
    fn show(&self, ctx: &Context, action: &mut Option<Action>) {
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.spacing_mut().button_padding = Vec2::new(32.0, 16.0);
                ui.add_space(ui.available_height() / 3.0);

                if ui.button(RichText::new("open").size(32.0)).clicked() {
                    *action = Some(Action::File(Open));
                }

                for RecentRepo { path, .. } in self.config.recent_repos() {
                    if ui.button(path.to_string_lossy()).clicked() {
                        *action = Some(Action::File(OpenRecent(path.clone())))
                    }
                }
            });
        });
    }
}
