use crate::{
    common::{
        Action, DiffKey, Diffs,
        RepoAction::{AddAll, AddOrRestore, RestoreAll},
    },
    panels::Show,
};
use eframe::egui::{
    Align, Button, Context, Key, Layout, ScrollArea, SidePanel, Ui,
    collapsing_header::CollapsingState,
};

pub struct PathsPanel<'a> {
    is_executing: bool,
    diffs: &'a Diffs,
    selected_key: &'a mut Option<DiffKey>,
}

impl<'a> PathsPanel<'a> {
    pub fn new(
        is_executing: bool,
        diffs: &'a Diffs,
        selected_key: &'a mut Option<DiffKey>,
    ) -> Self {
        Self {
            is_executing,
            diffs,
            selected_key,
        }
    }

    fn show_keys<P>(&mut self, ui: &mut Ui, action: &mut Option<Action>, predicate: P)
    where
        P: FnMut(&&DiffKey) -> bool,
    {
        let keys = self.diffs.keys().filter(predicate);
        for key in keys {
            let checked = self.selected_key.as_ref() == Some(key);
            let response = ui.selectable_label(checked, &key.path);
            let key_pressed = ui.input(|i| i.key_pressed(Key::Enter) || i.key_pressed(Key::Space));

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
    }

    fn show_unstaged(&mut self, ui: &mut Ui, action: &mut Option<Action>) {
        let id = ui.make_persistent_id("unstaged");

        CollapsingState::load_with_default_open(ui.ctx(), id, true)
            .show_header(ui, |ui| {
                ui.label("unstaged");

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add_enabled(!self.is_executing, Button::selectable(false, "+"))
                        .clicked()
                    {
                        *action = Some(Action::Repo(AddAll));
                    }
                });
            })
            .body(|ui| {
                let max_scroll_height = ui.available_height() * 0.5;

                ScrollArea::both()
                    .id_salt("unstaged_scroll")
                    .max_height(max_scroll_height)
                    .show(ui, |ui| {
                        ui.take_available_space();
                        self.show_keys(ui, action, DiffKey::is_not_staged);
                    });
            });
    }

    fn show_staged(&mut self, ui: &mut Ui, action: &mut Option<Action>) {
        let id = ui.make_persistent_id("staged");

        CollapsingState::load_with_default_open(ui.ctx(), id, true)
            .show_header(ui, |ui| {
                ui.label("staged");

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add_enabled(!self.is_executing, Button::selectable(false, "-"))
                        .clicked()
                    {
                        *action = Some(Action::Repo(RestoreAll));
                    }
                });
            })
            .body(|ui| {
                ScrollArea::both().id_salt("staged_scroll").show(ui, |ui| {
                    ui.take_available_space();
                    self.show_keys(ui, action, DiffKey::is_staged);
                });
            });
    }
}

impl<'a> Show for PathsPanel<'a> {
    fn show(&mut self, ctx: &Context, action: &mut Option<Action>) {
        let max_panel_width = ctx.content_rect().width() * 0.5;

        SidePanel::left("paths")
            .max_width(max_panel_width)
            .show(ctx, |ui| {
                self.show_unstaged(ui, action);

                ui.separator();

                self.show_staged(ui, action);
            });
    }
}
