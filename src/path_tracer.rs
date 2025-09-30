use crate::{AppState, screens::ScreenSetup, world::BloxWorld};
use bevy::{
    asset::RenderAssetUsages,
    platform::time::Instant,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::PrimaryWindow,
};

// TODO: Do not use bevy_ui but custom node that allows partial updates of the resulting image
// to stream pixels over multiple frames.

pub fn plugin(app: &mut App) {
    // Setup and cleanup
    app.add_systems(OnEnter(AppState::Game), setup.after(ScreenSetup));
    app.add_systems(OnExit(AppState::Game), cleanup);

    // Update
    app.add_systems(Update, update.run_if(in_state(AppState::Game)));
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn((
        Name::new("Path Tracer"),
        Node {
            display: Display::None,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        ImageNode::new(images.add(Image::transparent())),
        Pickable::IGNORE,
        StateScoped(AppState::Game),
    ));
}

fn cleanup(mut _commands: Commands) {}

fn update(
    mut image: Single<(&mut Node, &mut ImageNode)>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&GlobalTransform, &Projection), With<Camera3d>>,
    clear_color: Res<ClearColor>,
    mut images: ResMut<Assets<Image>>,
    world: Res<BloxWorld>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // if !keyboard_input.just_pressed(KeyCode::KeyT) {
    //     return;
    // }
    // let rebuild = keyboard_input.pressed(KeyCode::ShiftLeft);
    let rebuild = true;

    if !rebuild {
        image.0.display = match image.0.display {
            Display::DEFAULT => Display::None,
            _ => Display::DEFAULT,
        };
        return;
    }
    image.0.display = Display::DEFAULT;

    let scene = world.to_scene();
    let dimensions = window.physical_size() / 1;
    let renderer = lux::Renderer::init(
        lux::Camera {
            translation: camera.0.translation(),
            direction: camera.0.forward(),
            up: Dir3::Y,
            fov: match camera.1 {
                Projection::Perspective(p) => p.fov,
                _ => PerspectiveProjection::default().fov,
            },
            background: **clear_color,
        },
        dimensions,
    );

    let start = Instant::now();
    let pixels = renderer.render(&scene);
    let elapsed = start.elapsed();
    log::info!("Rendered in {:?}", elapsed);

    *images.get_mut(&image.1.image).unwrap() = Image::new(
        Extent3d {
            width: dimensions.x,
            height: dimensions.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels
            .into_iter()
            .flat_map(|p| p.to_srgba().with_alpha(0.75).to_u8_array())
            .collect(),
        TextureFormat::bevy_default(),
        RenderAssetUsages::default(),
    );
}
