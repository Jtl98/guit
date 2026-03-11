use crate::{
    common::{
        Action, Branches,
        FileAction::Close,
        RepoAction::{Create, Fetch, Pull, Push, Refresh, Switch},
    },
    panels::Show,
};
use eframe::egui::{Align, Button, ComboBox, Context, Key, Layout, TextEdit, TopBottomPanel};

pub struct TopPanel<'a> {
    is_executing: &'a bool,
    branches: &'a Branches,
    branch_name: &'a mut Option<String>,
}

impl<'a> TopPanel<'a> {
    pub fn new(
        is_executing: &'a bool,
        branches: &'a Branches,
        branch_name: &'a mut Option<String>,
    ) -> Self {
        Self {
            is_executing,
            branches,
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
                        .selected_text(&self.branches.current)
                        .show_ui(ui, |ui| {
                            for branch in &self.branches.other {
                                if ui.selectable_label(false, branch.to_string()).clicked() {
                                    *action = Some(Action::Repo(Switch(branch.clone())));
                                }
                            }
                        });

                    let branch_name = self.branch_name.get_or_insert_default();
                    let branch_name_provided = !branch_name.trim().is_empty();
                    let button = ui
                        .add_enabled(!self.is_executing && branch_name_provided, Button::new("+"));
                    let text =
                        ui.add_enabled(!self.is_executing, TextEdit::singleline(branch_name));
                    let key_pressed = text.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));

                    if branch_name_provided
                        && (button.clicked() || key_pressed)
                        && let Some(branch_name) = self.branch_name.take()
                    {
                        *action = Some(Action::Repo(Create(branch_name)));
                    }
                });
            });
        });
    }
}
