use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowMode},
};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, full_screen);
}

fn full_screen(
    mut primary_window: Single<&mut Window, With<PrimaryWindow>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::F11) {
        primary_window.mode = match primary_window.mode {
            WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
            _ => WindowMode::Windowed,
        };
    }
}
