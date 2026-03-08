pub mod app_logs;
pub mod bottom;
pub mod paths;
pub mod top;
pub mod welcome;

use crate::common::Action;
use eframe::egui::Context;

pub trait Show {
    fn show(&mut self, ctx: &Context, action: &mut Option<Action>);
}
