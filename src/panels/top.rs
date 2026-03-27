use crate::{
    common::{
        Action, Branch, Branches,
        FileAction::Close,
        RepoAction::{Create, Fetch, Pull, Push, Refresh, StashPop, StashPush, Switch},
    },
    panels::{AddWidget, Show},
};
use eframe::egui::{Align, ComboBox, Context, Key, Layout, Popup, TextEdit, TopBottomPanel, Ui};

pub struct TopPanel<'a> {
    is_executing: bool,
    branches: &'a Branches,
    branch_name: &'a mut Option<String>,
    branch_filter: &'a mut String,
}

impl<'a> TopPanel<'a> {
    const CREATE_TEXT_EDIT_WIDTH: f32 = 96.0;

    pub fn new(
        is_executing: bool,
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
        let mut first_branch = None;

        let branch_box = ui
            .add_enabled_ui(!self.is_executing, |ui| {
                ComboBox::from_id_salt("branches")
                    .width(0.0)
                    .selected_text(&self.branches.current)
                    .show_ui(ui, |ui| {
                        ui.take_available_height();

                        let filter_edit = ui.text_edit_singleline(self.branch_filter);
                        let branch_filter = self.branch_filter.to_lowercase();

                        let mut filtered_branches = self
                            .branches
                            .other
                            .iter()
                            .filter(|Branch { name, .. }| {
                                name.to_lowercase().contains(&branch_filter)
                            })
                            .peekable();

                        first_branch = filtered_branches.peek().copied();
                        for branch in filtered_branches {
                            if ui.selectable_label(false, branch.to_string()).clicked() {
                                *action = Some(Action::Repo(Switch(branch.clone())));
                            }
                        }

                        filter_edit
                    })
            })
            .inner;

        let Some(filter_edit) = branch_box.inner else {
            return;
        };

        if branch_box.response.clicked() {
            filter_edit.request_focus();
            self.branch_filter.clear();
        }

        let Some(first_branch) = first_branch else {
            return;
        };

        let key_pressed = filter_edit.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        if key_pressed {
            Popup::close_all(ui.ctx());
            *action = Some(Action::Repo(Switch(first_branch.clone())));
        }
    }

    fn show_branch_creation(&mut self, ui: &mut Ui, action: &mut Option<Action>) {
        let branch_name = self.branch_name.get_or_insert_default();
        let branch_name_provided = !branch_name.trim().is_empty();
        let create_text_edit = ui.add_enabled(
            !self.is_executing,
            TextEdit::singleline(branch_name).desired_width(Self::CREATE_TEXT_EDIT_WIDTH),
        );
        let create_button = ui.enabled_button(!self.is_executing && branch_name_provided, "create");

        if !branch_name_provided {
            return;
        }

        let key_pressed = create_text_edit.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        if !create_button.clicked() && !key_pressed {
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

                ui.separator();
                ui.label("branch");
                self.show_branches(ui, action);
                self.show_branch_creation(ui, action);

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.action_button(true, "close", action, Action::File(Close));
                });
            });
        });
    }
}
