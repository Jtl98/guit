use eframe::egui::{Context, Key, ScrollArea, SidePanel, Ui};

use crate::{
    common::{Action, DiffKey, RepoAction::AddOrRestore},
    panels::Show,
    repo::Repo,
};

pub struct PathsPanel<'a> {
    repo: &'a Repo,
    selected_key: &'a mut Option<DiffKey>,
}

impl<'a> PathsPanel<'a> {
    pub fn new(repo: &'a Repo, selected_key: &'a mut Option<DiffKey>) -> Self {
        Self { repo, selected_key }
    }
}

impl<'a> Show for PathsPanel<'a> {
    fn show(&mut self, ctx: &Context, action: &mut Option<Action>) {
        SidePanel::left("paths").show(ctx, |ui| {
            let mut show_keys = |ui: &mut Ui, id, max_height, filter: fn(&&DiffKey) -> bool| {
                ScrollArea::both()
                    .id_salt(id)
                    .max_height(max_height)
                    .show(ui, |ui| {
                        ui.take_available_space();

                        let keys = self.repo.diffs.keys().filter(filter);
                        for key in keys {
                            let checked = self.selected_key.as_ref() == Some(key);
                            let response = ui.selectable_label(checked, &key.path);
                            let key_pressed = ui
                                .input(|i| i.key_pressed(Key::Enter) || i.key_pressed(Key::Space));

                            if response.double_clicked() || (key_pressed && response.has_focus()) {
                                *action = Some(Action::Repo(AddOrRestore(key.clone())));
                                *self.selected_key = None;
                            } else if response.clicked() {
                                *self.selected_key = if checked {
                                    None
                                } else {
                                    response.request_focus();
                                    Some(key.clone())
                                }
                            }
                        }
                    });
            };

            show_keys(
                ui,
                "unstaged",
                ui.available_height() / 2.0,
                DiffKey::is_not_staged,
            );
            ui.separator();
            show_keys(ui, "staged", ui.available_height(), DiffKey::is_staged);
        });
    }
}
