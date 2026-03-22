use crate::{
    common::{
        Action,
        RepoAction::{self, Commit},
    },
    panels::{AddWidget, Show},
};
use eframe::egui::{
    Align, Align2, Context, Key, Layout, TextEdit, TextStyle, TopBottomPanel, Ui, Vec2, Window,
};
use std::path::Path;

pub struct BottomPanel<'a> {
    is_executing: bool,
    show_logs: &'a mut bool,
    dir: &'a Path,
    commit_subject: &'a mut Option<String>,
    commit_body: &'a mut Option<String>,
    show_commit_body: &'a mut bool,
}

impl<'a> BottomPanel<'a> {
    const COMMIT_BODY_WIDTH: f32 = 384.0;
    const COMMIT_BODY_OFFSET: Vec2 = Vec2::new(8.0, -32.0);

    pub fn new(
        is_executing: bool,
        show_logs: &'a mut bool,
        dir: &'a Path,
        commit_subject: &'a mut Option<String>,
        commit_body: &'a mut Option<String>,
        show_commit_body: &'a mut bool,
    ) -> Self {
        Self {
            is_executing,
            show_logs,
            dir,
            commit_subject,
            commit_body,
            show_commit_body,
        }
    }

    fn show_commit_body(&mut self, ui: &mut Ui) {
        let arrow = if *self.show_commit_body { "⏶" } else { "⏵" };
        let clicked = ui.enabled_button(!self.is_executing, arrow).clicked();

        if clicked {
            *self.show_commit_body = !*self.show_commit_body;
        }

        if !*self.show_commit_body {
            return;
        }

        let text_height = ui.text_style_height(&TextStyle::Body);
        let commit_body = self.commit_body.get_or_insert_default();

        let window = Window::new("commit_body")
            .title_bar(false)
            .scroll(true)
            .fixed_size([Self::COMMIT_BODY_WIDTH, text_height])
            .anchor(Align2::LEFT_BOTTOM, Self::COMMIT_BODY_OFFSET)
            .show(ui.ctx(), |ui| {
                ui.add_enabled_ui(!self.is_executing, |ui| {
                    ui.add_sized(ui.available_size(), TextEdit::multiline(commit_body))
                })
            });

        if clicked
            && let Some(window) = window
            && let Some(enabled) = window.inner
        {
            enabled.inner.request_focus();
        }
    }

    fn show_commit_subject(&mut self, ui: &mut Ui, action: &mut Option<Action>) {
        let commit_subject = self.commit_subject.get_or_insert_default();
        let commit_subject_provided = !commit_subject.trim().is_empty();

        let text_edit = ui.add_enabled(!self.is_executing, TextEdit::singleline(commit_subject));
        let button = ui.enabled_button(!self.is_executing && commit_subject_provided, "commit");

        if !commit_subject_provided {
            return;
        }

        let key_pressed = text_edit.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        if !button.clicked() && !key_pressed {
            return;
        }

        let Some(commit_subject) = self.commit_subject.take() else {
            return;
        };

        let commit_body = self.commit_body.take();
        *action = Some(Action::Repo(Commit(commit_subject, commit_body)));
    }
}

impl<'a> Show for BottomPanel<'a> {
    fn show(&mut self, ctx: &Context, action: &mut Option<Action>) {
        TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.show_commit_body(ui);

                self.show_commit_subject(ui, action);

                let undo_button = ui.enabled_button(!self.is_executing, "undo");
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
