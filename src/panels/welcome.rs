use crate::{
    common::{
        Action,
        FileAction::{Init, Open, OpenRecent, RemoveRecent},
    },
    config::Config,
    panels::Show,
};
use eframe::egui::{Button, CentralPanel, Context, RichText, ScrollArea, TextWrapMode, Vec2};

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

                ScrollArea::both().id_salt("file").show(c2, |c2| {
                    if c2.button(RichText::new("init").size(16.0)).clicked() {
                        *action = Some(Action::File(Init));
                    }

                    c2.add_space(16.0);

                    if c2.button(RichText::new("open").size(16.0)).clicked() {
                        *action = Some(Action::File(Open));
                    }
                });

                c3.label(RichText::new("recent").size(32.0));
                c3.add_space(16.0);

                ScrollArea::both().id_salt("recent").show(c3, |c3| {
                    for repo in self.config.recent_repos() {
                        c3.horizontal(|c3| {
                            let remove_text = RichText::new("x").size(16.0);
                            let remove_button = Button::new(remove_text);
                            if c3.add(remove_button).clicked() {
                                *action = Some(Action::File(RemoveRecent(repo.clone())));
                            }

                            let open_text = RichText::new(repo.path.to_string_lossy()).size(16.0);
                            let open_button =
                                Button::new(open_text).wrap_mode(TextWrapMode::Extend);
                            if c3.add_sized(c3.available_size(), open_button).clicked() {
                                *action = Some(Action::File(OpenRecent(repo.path.clone())));
                            }
                        });

                        c3.add_space(16.0);
                    }
                });
            });
        });
    }
}
