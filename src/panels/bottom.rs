use crate::{
    common::{Action, DiffKey, RepoAction::Commit},
    panels::Show,
    repo::Repo,
};
use eframe::egui::{Align, Button, Context, Key, Layout, TextEdit, TopBottomPanel};

pub struct BottomPanel<'a> {
    is_executing: bool,
    show_logs: &'a mut bool,
    repo: &'a Repo,
    selected_key: &'a mut Option<DiffKey>,
    commit_message: &'a mut String,
}

impl<'a> BottomPanel<'a> {
    pub fn new(
        is_executing: bool,
        show_logs: &'a mut bool,
        repo: &'a Repo,
        selected_key: &'a mut Option<DiffKey>,
        commit_message: &'a mut String,
    ) -> Self {
        Self {
            is_executing,
            show_logs,
            repo,
            selected_key,
            commit_message,
        }
    }
}

impl<'a> Show for BottomPanel<'a> {
    fn show(&mut self, ctx: &Context, action: &mut Option<Action>) {
        TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let commit_message_provided = !self.commit_message.trim().is_empty();
                let text = ui.add_enabled(
                    !self.is_executing,
                    TextEdit::singleline(self.commit_message),
                );
                let button = ui.add_enabled(
                    !self.is_executing && commit_message_provided,
                    Button::new("commit"),
                );

                if commit_message_provided
                    && (button.clicked()
                        || (text.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter))))
                {
                    *action = Some(Action::Repo(Commit(self.commit_message.to_owned())));
                    *self.selected_key = None;
                    self.commit_message.clear();
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.selectable_label(*self.show_logs, "logs").clicked() {
                        *self.show_logs = !*self.show_logs;
                    }

                    let dir = self.repo.dir.to_string_lossy();

                    ui.label(dir);

                    if self.is_executing {
                        ui.spinner();
                    }
                });
            });
        });
    }
}
