pub mod app_logs;
pub mod bottom;
pub mod diff;
pub mod git_logs;
pub mod paths;
pub mod top;
pub mod welcome;

use crate::common::Action;
use eframe::egui::{Button, Context, IntoAtoms, Response, Ui};

pub trait Show {
    fn show(&mut self, ctx: &Context, action: &mut Option<Action>);
}

trait AddWidget {
    fn enabled_button(&mut self, enabled: bool, text: impl IntoAtoms<'static>) -> Response;

    fn action_button(
        &mut self,
        enabled: bool,
        text: impl IntoAtoms<'static>,
        action: &mut Option<Action>,
        new_action: Action,
    );
}

impl AddWidget for Ui {
    fn enabled_button(&mut self, enabled: bool, text: impl IntoAtoms<'static>) -> Response {
        self.add_enabled(enabled, Button::new(text))
    }

    fn action_button(
        &mut self,
        enabled: bool,
        text: impl IntoAtoms<'static>,
        action: &mut Option<Action>,
        new_action: Action,
    ) {
        if self.enabled_button(enabled, text).clicked() {
            *action = Some(new_action);
        }
    }
}
