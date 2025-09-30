use super::ScreenSetup;
use crate::{AppState, AssetsState};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    // Setup and cleanup
    app.add_systems(OnEnter(AppState::SplashScreen), setup.in_set(ScreenSetup));
    app.add_systems(OnExit(AppState::SplashScreen), cleanup);

    // Update
    app.add_systems(
        Update,
        splash_screen.run_if(in_state(AppState::SplashScreen)),
    );
}

#[derive(Debug, Resource)]
struct SplashScreen {
    timer: Timer,
    clicked: bool,
}

impl Default for SplashScreen {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
            clicked: cfg!(feature = "dev"),
        }
    }
}

fn splash_screen(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    assets_state: Res<State<AssetsState>>,
    mut splash_screen: ResMut<SplashScreen>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    splash_screen.timer.tick(time.delta());
    splash_screen.clicked |= keyboard_input.just_pressed(KeyCode::Space)
        || mouse_input.just_pressed(MouseButton::Left)
        || touches.iter_just_pressed().any(|_| true);

    // TODO: Handle AssetsState::Error

    if **assets_state == AssetsState::Loaded
        && (splash_screen.timer.finished() || splash_screen.clicked)
    {
        next_state.set(AppState::Game);
    }
}

fn setup(mut commands: Commands) {
    commands.init_resource::<SplashScreen>();
    commands.spawn((Camera2d, StateScoped(AppState::SplashScreen)));
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<SplashScreen>();
}
