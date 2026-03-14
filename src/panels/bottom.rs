use crate::{
    common::{
        Action, DiffKey,
        RepoAction::{self, Commit},
    },
    panels::Show,
};
use eframe::egui::{Align, Button, Context, Key, Layout, TextEdit, TopBottomPanel};
use std::path::Path;

pub struct BottomPanel<'a> {
    is_executing: bool,
    show_logs: &'a mut bool,
    dir: &'a Path,
    selected_key: &'a mut Option<DiffKey>,
    commit_message: &'a mut Option<String>,
}

impl<'a> BottomPanel<'a> {
    pub fn new(
        is_executing: bool,
        show_logs: &'a mut bool,
        dir: &'a Path,
        selected_key: &'a mut Option<DiffKey>,
        commit_message: &'a mut Option<String>,
    ) -> Self {
        Self {
            is_executing,
            show_logs,
            dir,
            selected_key,
            commit_message,
        }
    }
}

impl<'a> Show for BottomPanel<'a> {
    fn show(&mut self, ctx: &Context, action: &mut Option<Action>) {
        TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let commit_message = self.commit_message.get_or_insert_default();
                let commit_message_provided = !commit_message.trim().is_empty();
                let text = ui.add_enabled(!self.is_executing, TextEdit::singleline(commit_message));
                let button = ui.add_enabled(
                    !self.is_executing && commit_message_provided,
                    Button::new("commit"),
                );
                let key_pressed = text.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));

                if commit_message_provided
                    && (button.clicked() || key_pressed)
                    && let Some(commit_message) = self.commit_message.take()
                {
                    *action = Some(Action::Repo(Commit(commit_message)));
                    *self.selected_key = None;
                }

                let undo_button = ui.add_enabled(!self.is_executing, Button::new("undo"));
                if undo_button.clicked() {
                    *action = Some(Action::Repo(RepoAction::UndoCommit));
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.selectable_label(*self.show_logs, "logs").clicked() {
                        *self.show_logs = !*self.show_logs;
                    }

                    ui.label(self.dir.to_string_lossy());

                    if self.is_executing {
                        ui.spinner();
                    }
                });
            });
        });
    }
}
