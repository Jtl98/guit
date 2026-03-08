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
    init_name: &'a mut String,
}

impl<'a> WelcomePanel<'a> {
    pub fn new(config: &'a Config, init_name: &'a mut String) -> Self {
        Self { config, init_name }
    }
}

impl<'a> Show for WelcomePanel<'a> {
    fn show(&mut self, ctx: &Context, action: &mut Option<Action>) {
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.2);
                ui.spacing_mut().button_padding = Vec2::new(32.0, 16.0);

                if ui.button(RichText::new("open").size(32.0)).clicked() {
                    *action = Some(Action::File(Open));
                }

                for RecentRepo { path, .. } in self.config.recent_repos() {
                    if ui.button(path.to_string_lossy()).clicked() {
                        *action = Some(Action::File(OpenRecent(path.clone())))
                    }
                }

                ui.add_space(ui.available_height() * 0.1);
                ui.spacing_mut().button_padding = Vec2::new(16.0, 8.0);

                ui.text_edit_singleline(self.init_name);
                if ui.button("init").clicked() {
                    *action = Some(Action::File(Init(self.init_name.clone())));
                }
            });
        });
    }
}
