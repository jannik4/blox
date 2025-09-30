mod game;
mod splash_screen;

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((splash_screen::plugin, game::plugin));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct ScreenSetup;
