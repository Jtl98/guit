pub mod app_logs;
pub mod bottom;
pub mod diff;
pub mod git_logs;
pub mod paths;
pub mod top;
pub mod welcome;

use crate::common::Action;
use eframe::egui::{Button, Context, Response, Ui};

pub trait Show {
    fn show(&mut self, ctx: &Context, action: &mut Option<Action>);
}

trait AddWidget {
    fn executing_button(&mut self, is_executing: bool, text: &str) -> Response;
}

impl AddWidget for Ui {
    fn executing_button(&mut self, is_executing: bool, text: &str) -> Response {
        self.add_enabled(!is_executing, Button::new(text))
    }
}
