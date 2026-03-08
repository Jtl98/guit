pub mod welcome;

use crate::common::Action;
use eframe::egui::Context;

pub trait Show {
    fn show(&self, ctx: &Context, action: &mut Option<Action>);
}
