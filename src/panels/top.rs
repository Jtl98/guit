use crate::{
    common::{
        Action,
        FileAction::Close,
        RepoAction::{Create, Fetch, Pull, Push, Refresh, Switch},
    },
    panels::Show,
    repo::Repo,
};
use eframe::egui::{Align, Button, ComboBox, Context, Key, Layout, TextEdit, TopBottomPanel};

pub struct TopPanel<'a> {
    is_executing: &'a bool,
    repo: &'a Repo,
    branch_name: &'a mut String,
}

impl<'a> TopPanel<'a> {
    pub fn new(is_executing: &'a bool, repo: &'a Repo, branch_name: &'a mut String) -> Self {
        Self {
            is_executing,
            repo,
            branch_name,
        }
    }
}

impl<'a> Show for TopPanel<'a> {
    fn show(&mut self, ctx: &Context, action: &mut Option<Action>) {
        TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!self.is_executing, Button::new("refresh"))
                    .clicked()
                {
                    *action = Some(Action::Repo(Refresh));
                }

                if ui
                    .add_enabled(!self.is_executing, Button::new("fetch"))
                    .clicked()
                {
                    *action = Some(Action::Repo(Fetch));
                }

                if ui
                    .add_enabled(!self.is_executing, Button::new("pull"))
                    .clicked()
                {
                    *action = Some(Action::Repo(Pull));
                }

                if ui
                    .add_enabled(!self.is_executing, Button::new("push"))
                    .clicked()
                {
                    *action = Some(Action::Repo(Push));
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("close").clicked() {
                        *action = Some(Action::File(Close));
                    }

                    ComboBox::from_id_salt("branches")
                        .selected_text(&self.repo.branches.current)
                        .show_ui(ui, |ui| {
                            for branch in &self.repo.branches.other {
                                if ui.selectable_label(false, branch.to_string()).clicked() {
                                    *action = Some(Action::Repo(Switch(branch.clone())));
                                }
                            }
                        });

                    let branch_name_provided = !self.branch_name.trim().is_empty();
                    let button = ui
                        .add_enabled(!self.is_executing && branch_name_provided, Button::new("+"));
                    let text =
                        ui.add_enabled(!self.is_executing, TextEdit::singleline(self.branch_name));

                    if branch_name_provided
                        && (button.clicked()
                            || (text.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter))))
                    {
                        *action = Some(Action::Repo(Create(self.branch_name.clone())));
                        self.branch_name.clear();
                    }
                });
            });
        });
    }
}
