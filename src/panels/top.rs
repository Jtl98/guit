use crate::{
    common::{
        Action, Branches,
        FileAction::Close,
        RepoAction::{Create, Fetch, Pull, Push, Refresh, StashPop, StashPush, Switch},
    },
    panels::{AddWidget, Show},
};
use eframe::egui::{Align, ComboBox, Context, Key, Layout, TextEdit, TopBottomPanel, Ui};

pub struct TopPanel<'a> {
    is_executing: &'a bool,
    branches: &'a Branches,
    branch_name: &'a mut Option<String>,
    branch_filter: &'a mut String,
}

impl<'a> TopPanel<'a> {
    pub fn new(
        is_executing: &'a bool,
        branches: &'a Branches,
        branch_name: &'a mut Option<String>,
        branch_filter: &'a mut String,
    ) -> Self {
        Self {
            is_executing,
            branches,
            branch_name,
            branch_filter,
        }
    }

    fn show_branches(&mut self, ui: &mut Ui, action: &mut Option<Action>) {
        let branch_box = ComboBox::from_id_salt("branches")
            .selected_text(&self.branches.current)
            .show_ui(ui, |ui| {
                ui.take_available_height();
                ui.text_edit_singleline(self.branch_filter).request_focus();

                let branch_filter = self.branch_filter.to_lowercase();
                for branch in &self.branches.other {
                    if !branch.name.to_lowercase().contains(&branch_filter) {
                        continue;
                    }

                    if ui.selectable_label(false, branch.to_string()).clicked() {
                        *action = Some(Action::Repo(Switch(branch.clone())));
                    }
                }
            });

        if branch_box.response.clicked() {
            self.branch_filter.clear();
        }

        let branch_name = self.branch_name.get_or_insert_default();
        let branch_name_provided = !branch_name.trim().is_empty();
        let button = ui.enabled_button(!self.is_executing && branch_name_provided, "+");
        let text = ui.add_enabled(!self.is_executing, TextEdit::singleline(branch_name));

        if !branch_name_provided {
            return;
        }

        let key_pressed = text.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        if !button.clicked() && !key_pressed {
            return;
        }

        if let Some(branch_name) = self.branch_name.take() {
            *action = Some(Action::Repo(Create(branch_name)));
        }
    }
}

impl<'a> Show for TopPanel<'a> {
    fn show(&mut self, ctx: &Context, action: &mut Option<Action>) {
        TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.style_mut().spacing.item_spacing.x = 4.0;

                ui.action_button(!self.is_executing, "refresh", action, Action::Repo(Refresh));

                ui.separator();
                ui.action_button(!self.is_executing, "fetch", action, Action::Repo(Fetch));
                ui.action_button(!self.is_executing, "pull", action, Action::Repo(Pull));
                ui.action_button(!self.is_executing, "push", action, Action::Repo(Push));

                ui.separator();
                ui.label("stash");
                ui.action_button(!self.is_executing, "push", action, Action::Repo(StashPush));
                ui.action_button(!self.is_executing, "pop", action, Action::Repo(StashPop));

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.action_button(true, "close", action, Action::File(Close));

                    self.show_branches(ui, action);
                });
            });
        });
    }
}
