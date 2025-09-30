pub mod fps;
pub mod full_screen;

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((fps::plugin, full_screen::plugin));
}

pub fn exp_lerp(lag_weight: f32, delta: f32) -> f32 {
    1.0 - f32::exp(f32::ln(lag_weight) * 60.0 * delta)
}
