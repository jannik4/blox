use super::ScreenSetup;
use crate::{AppState, AssetsState, camera_controller::CameraController};
use bevy::{core_pipeline::oit::OrderIndependentTransparencySettings, prelude::*};
use bevy_asset_loader::prelude::*;
use std::f32::consts::PI;

pub fn plugin(app: &mut App) {
    // Setup and cleanup
    app.add_systems(OnEnter(AppState::Game), setup.in_set(ScreenSetup));
    app.add_systems(OnExit(AppState::Game), cleanup);

    // Assets
    app.configure_loading_state(
        LoadingStateConfig::new(AssetsState::Loading).load_collection::<GameAssets>(),
    );
}

#[derive(AssetCollection, Resource)]
struct GameAssets {}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        Projection::Perspective(PerspectiveProjection {
            fov: PI / 4.0,
            near: 0.01,
            far: 100.0,
            aspect_ratio: 1.0,
        }),
        OrderIndependentTransparencySettings::default(),
        Msaa::Off, // Msaa currently doesn't work with OIT
        Transform::from_xyz(-32.0, 32.0, -32.0).looking_at(Vec3::ZERO, Vec3::Y),
        CameraController::default(),
        StateScoped(AppState::Game),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 5_000.0,
            shadows_enabled: true,
            ..default()
        },
        bevy::pbr::CascadeShadowConfigBuilder {
            num_cascades: 4,
            minimum_distance: 0.1,
            maximum_distance: 75.0,
            first_cascade_far_bound: 5.0,
            overlap_proportion: 0.2,
        }
        .build(),
        Transform::from_xyz(-1.0, 0.5, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        StateScoped(AppState::Game),
    ));
    commands.spawn((
        PointLight {
            intensity: 1_000_000.0,
            range: 75.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(11.5, 5.5, 7.5),
        StateScoped(AppState::Game),
    ));
}

fn cleanup(mut _commands: Commands) {}
