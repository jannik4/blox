use crate::{AppState, AssetsState, screens::ScreenSetup};
use bevy::{
    color::palettes::tailwind,
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};
use bevy_asset_loader::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins(MaterialPlugin::<
        ExtendedMaterial<StandardMaterial, GroundExtension>,
    >::default());

    // Setup and cleanup
    app.add_systems(OnEnter(AppState::Game), setup.after(ScreenSetup));
    app.add_systems(OnExit(AppState::Game), cleanup);

    // Assets
    app.configure_loading_state(
        LoadingStateConfig::new(AssetsState::Loading).load_collection::<GroundAssets>(),
    );
}

#[derive(AssetCollection, Resource)]
struct GroundAssets {
    #[expect(unused)] // Only place this here to ensure the shader is loaded
    #[asset(path = "shaders/ground.wgsl")]
    ground_shader: Handle<Shader>,
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, GroundExtension>>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn((
        Name::new("Ground"),
        Transform::from_xyz(7.5, 0.0, 7.5),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(7.5)))),
        MeshMaterial3d(materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: tailwind::GREEN_800.into(),
                alpha_mode: AlphaMode::Blend,
                reflectance: 0.05,
                ..default()
            },
            extension: GroundExtension {},
        })),
        StateScoped(AppState::Game),
    ));
}

fn cleanup(mut _commands: Commands) {}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
struct GroundExtension {}

impl MaterialExtension for GroundExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/ground.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/ground.wgsl".into()
    }
}
