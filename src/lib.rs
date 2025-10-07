mod camera_controller;
mod ground;
mod ray_tracer;
mod screens;
mod util;
mod world;

use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};

pub use self::world::{Block, BloxScene, BloxWorld};

pub struct BloxPlugin;

impl Plugin for BloxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Blox".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::srgb(0.5, 0.5, 1.0)))
        .insert_resource(AmbientLight {
            brightness: 80.0,
            ..default()
        });

        app.init_state::<AppState>().init_state::<AssetsState>();
        app.enable_state_scoped_entities::<AppState>();
        app.add_loading_state({
            LoadingState::new(AssetsState::Loading)
                .continue_to_state(AssetsState::Loaded)
                .on_failure_continue_to_state(AssetsState::Error)
        });

        app.add_plugins((
            screens::plugin,
            ground::plugin,
            world::plugin,
            camera_controller::plugin,
            ray_tracer::plugin,
            util::plugin,
        ));
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    SplashScreen,
    Game,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AssetsState {
    #[default]
    Loading,
    Loaded,
    Error,
}
