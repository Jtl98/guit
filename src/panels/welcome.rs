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
            ui.spacing_mut().button_padding = Vec2::new(16.0, 8.0);
            ui.add_space(32.0);

            ui.columns_const(|[_, c2, c3, _]| {
                c2.label(RichText::new("file").size(32.0));
                c2.add_space(16.0);

                if c2.button(RichText::new("init").size(16.0)).clicked() {
                    *action = Some(Action::File(Init));
                }

                c2.add_space(16.0);

                if c2.button(RichText::new("open").size(16.0)).clicked() {
                    *action = Some(Action::File(Open));
                }

                c3.label(RichText::new("recent").size(32.0));
                c3.add_space(16.0);

                for RecentRepo { path, .. } in self.config.recent_repos() {
                    let text = RichText::new(path.to_string_lossy()).size(16.0);
                    if c3.button(text).clicked() {
                        *action = Some(Action::File(OpenRecent(path.clone())))
                    }

                    c3.add_space(16.0);
                }
            });
        });
    }
}
