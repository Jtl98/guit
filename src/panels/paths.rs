use crate::{
    common::{Action, DiffKey, Diffs, RepoAction::AddOrRestore},
    panels::Show,
};
use eframe::egui::{CollapsingHeader, Context, Key, ScrollArea, SidePanel, Ui};

pub struct PathsPanel<'a> {
    diffs: &'a Diffs,
    selected_key: &'a mut Option<DiffKey>,
}

impl<'a> PathsPanel<'a> {
    pub fn new(diffs: &'a Diffs, selected_key: &'a mut Option<DiffKey>) -> Self {
        Self {
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
}

impl<'a> Show for PathsPanel<'a> {
    fn show(&mut self, ctx: &Context, action: &mut Option<Action>) {
        let max_panel_width = ctx.content_rect().width() * 0.5;

        SidePanel::left("paths")
            .max_width(max_panel_width)
            .show(ctx, |ui| {
                CollapsingHeader::new("unstaged")
                    .default_open(true)
                    .show(ui, |ui| {
                        let max_scroll_height = ui.available_height() * 0.5;

                        ScrollArea::both()
                            .id_salt("unstaged_scroll")
                            .max_height(max_scroll_height)
                            .show(ui, |ui| {
                                ui.take_available_space();
                                self.show_keys(ui, action, DiffKey::is_not_staged);
                            });
                    });

                ui.separator();

                CollapsingHeader::new("staged")
                    .default_open(true)
                    .show(ui, |ui| {
                        ScrollArea::both().id_salt("staged_scroll").show(ui, |ui| {
                            ui.take_available_space();
                            self.show_keys(ui, action, DiffKey::is_staged);
                        });
                    });
            });
    }
}
