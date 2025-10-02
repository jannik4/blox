use crate::{
    AppState, AssetsState,
    screens::ScreenSetup,
    world::{Block, BloxScene, BloxWorld, WORLD_SIZE, WorldAssets},
};
use bevy::{
    asset::RenderAssetUsages,
    platform::time::Instant,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::PrimaryWindow,
};
use bevy_asset_loader::loading_state::config::LoadingStateConfig;
use bevy_asset_loader::prelude::*;
use std::sync::Arc;

// TODO: Do not use bevy_ui but custom node that allows partial updates of the resulting image
// to stream pixels over multiple frames.

pub fn plugin(app: &mut App) {
    // Setup and cleanup
    app.add_systems(OnEnter(AppState::Game), setup.after(ScreenSetup));
    app.add_systems(OnExit(AppState::Game), cleanup);

    // Assets
    app.configure_loading_state(
        LoadingStateConfig::new(AssetsState::Loading).finally_init_resource::<BlockTextures>(),
    );

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

#[derive(Debug, Default, PartialEq, Eq)]
enum RenderMode {
    Disabled,
    #[default]
    Continuous,
    SingleFrame,
}

fn update(
    mut mode: Local<RenderMode>,
    mut transparent: Local<bool>,

    mut image: Single<(&mut Node, &mut ImageNode)>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&GlobalTransform, &Projection), With<Camera3d>>,
    clear_color: Res<ClearColor>,
    mut images: ResMut<Assets<Image>>,
    world: Res<BloxWorld>,
    block_textures: Res<BlockTextures>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let mut rebuild = false;

    if keyboard_input.just_pressed(KeyCode::Digit1) {
        *mode = RenderMode::Disabled;
    } else if keyboard_input.just_pressed(KeyCode::Digit2) {
        *mode = RenderMode::Continuous;
    } else if keyboard_input.just_pressed(KeyCode::Digit3) {
        *mode = RenderMode::SingleFrame;
        rebuild = true;
    }

    if keyboard_input.just_pressed(KeyCode::KeyT) {
        *transparent = !*transparent;
        rebuild = true;
    }

    if *mode == RenderMode::Disabled {
        image.0.display = Display::None;
        return;
    }
    image.0.display = Display::DEFAULT;

    if *mode == RenderMode::Continuous {
        rebuild = true;
    }

    if !rebuild {
        return;
    }

    let scale = match *mode {
        RenderMode::SingleFrame => 1,
        RenderMode::Continuous => 4,
        RenderMode::Disabled => unreachable!(),
    };

    let scene = LuxScene {
        scene: world.to_scene(),
        textures: block_textures.clone(),
    };
    let dimensions = window.physical_size() / scale;
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
    if *mode == RenderMode::SingleFrame {
        log::info!("Rendered in {:?}", elapsed);
    }

    *images.get_mut(&image.1.image).unwrap() = Image::new(
        Extent3d {
            width: dimensions.x,
            height: dimensions.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels
            .into_iter()
            .flat_map(|p| {
                if *transparent {
                    p.to_srgba().with_alpha(0.5).to_u8_array()
                } else {
                    p.to_srgba().with_alpha(1.0).to_u8_array()
                }
            })
            .collect(),
        TextureFormat::bevy_default(),
        RenderAssetUsages::default(),
    );
}

#[derive(Debug, Clone, Resource)]
struct BlockTextures {
    textures: Arc<[BlockTexture]>,
}

impl BlockTextures {
    fn sample(&self, block: Block, face: Face, uv: Vec2) -> lux::Material {
        lux::Material::Diffuse {
            albedo: match block {
                Block::Air => LinearRgba::NAN,
                Block::Dirt => self.textures[0].sample(uv),
                Block::Stone => self.textures[1].sample(uv),
                Block::Sand => self.textures[2].sample(uv),
                Block::Grass => match face {
                    Face::YPos => self.textures[4].sample(uv),
                    Face::YNeg => self.textures[0].sample(uv),
                    _ => self.textures[3].sample(uv),
                },
                Block::Wood => self.textures[5].sample(uv),
                Block::Leaves => self.textures[6].sample(uv),
                Block::Water => self.textures[7].sample(uv),
            },
        }
    }
}

impl FromWorld for BlockTextures {
    fn from_world(world: &mut World) -> Self {
        let mut textures = Vec::new();

        let world_assets = world.resource::<WorldAssets>();
        let images = world.resource::<Assets<Image>>();
        for handle in &world_assets.block_images {
            let image = images.get(handle).unwrap();

            assert_eq!(
                image.texture_descriptor.format,
                TextureFormat::Rgba8UnormSrgb
            );

            textures.push(BlockTexture {
                size: image.size(),
                data: image
                    .data
                    .as_ref()
                    .unwrap()
                    .chunks(4)
                    .map(|chunk| {
                        LinearRgba::from(Srgba::new(
                            chunk[0] as f32 / 255.0,
                            chunk[1] as f32 / 255.0,
                            chunk[2] as f32 / 255.0,
                            chunk[3] as f32 / 255.0,
                        ))
                    })
                    .collect(),
            });
        }

        Self {
            textures: textures.into(),
        }
    }
}

#[derive(Debug)]
struct BlockTexture {
    size: UVec2,
    data: Vec<LinearRgba>,
}

impl BlockTexture {
    fn sample(&self, uv: Vec2) -> LinearRgba {
        let uv = uv.fract();
        let u = (uv.x * self.size.x as f32).clamp(0.0, self.size.x as f32 - 1.0) as u32;
        let v = (uv.y * self.size.y as f32).clamp(0.0, self.size.y as f32 - 1.0) as u32;
        self.data[(v * self.size.x + u) as usize]
    }
}

#[derive(Debug)]
struct LuxScene {
    scene: BloxScene,
    textures: BlockTextures,
}

impl lux::Scene for LuxScene {
    fn cast_ray(&self, ray: Ray3d) -> Option<lux::RayHit> {
        fn interval(start: f32, speed: f32) -> Option<(f32, f32)> {
            if (start < 0.0 && speed <= 0.0) || (start > WORLD_SIZE as f32 && speed >= 0.0) {
                None
            } else if speed == 0.0 {
                (start >= 0.0 && start < WORLD_SIZE as f32)
                    .then_some((f32::NEG_INFINITY, f32::INFINITY))
            } else {
                let t1 = -start / speed;
                let t2 = (WORLD_SIZE as f32 - start) / speed;
                let (t1, t2) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
                Some((t1.max(0.0), t2))
            }
        }

        fn clamp_origin(ray: Ray3d) -> Option<Vec3> {
            if ray.origin.x >= 0.0
                && ray.origin.x < WORLD_SIZE as f32
                && ray.origin.y >= 0.0
                && ray.origin.y < WORLD_SIZE as f32
                && ray.origin.z >= 0.0
                && ray.origin.z < WORLD_SIZE as f32
            {
                return Some(ray.origin);
            }

            let x = interval(ray.origin.x, ray.direction.x)?;
            let y = interval(ray.origin.y, ray.direction.y)?;
            let z = interval(ray.origin.z, ray.direction.z)?;

            let interval = (x.0.max(y.0).max(z.0), x.1.min(y.1).min(z.1));

            // Add small epsilon to avoid rounding issues when clamped to edge
            (interval.0 <= interval.1)
                .then(|| ray.origin + interval.0 * ray.direction + Vec3::splat(0.001))
        }

        fn time_to_edge(pos: f32, block: i32, speed: f32) -> (f32, i32) {
            if speed > 0.0 {
                (((block as f32) + 1.0 - pos) / speed, 1)
            } else if speed < 0.0 {
                (((block as f32) - pos) / speed, -1)
            } else {
                (f32::INFINITY, 0)
            }
        }

        fn face_and_uv(pos: Vec3, block: IVec3) -> (Face, Vec2) {
            let rel = pos - block.as_vec3();
            let (face, _dis) = [
                (Face::XNeg, f32::abs(rel.x)),
                (Face::XPos, f32::abs(1.0 - rel.x)),
                (Face::YNeg, f32::abs(rel.y)),
                (Face::YPos, f32::abs(1.0 - rel.y)),
                (Face::ZNeg, f32::abs(rel.z)),
                (Face::ZPos, f32::abs(1.0 - rel.z)),
            ]
            .into_iter()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();
            let uv = match face {
                Face::XNeg => Vec2::new(1.0 - rel.z, 1.0 - rel.y),
                Face::XPos => Vec2::new(rel.z, 1.0 - rel.y),
                Face::YNeg => Vec2::new(rel.x, rel.z),
                Face::YPos => Vec2::new(rel.x, 1.0 - rel.z),
                Face::ZNeg => Vec2::new(1.0 - rel.x, 1.0 - rel.y),
                Face::ZPos => Vec2::new(rel.x, 1.0 - rel.y),
            };
            (face, uv)
        }

        // Clamp origin to world bounds
        let mut current_position = clamp_origin(ray)?;

        // Current block from position
        // - floor to get block coordinates
        // - clamp to world bounds
        let mut current_block = current_position
            .floor()
            .as_ivec3()
            .min(IVec3::splat(WORLD_SIZE as i32 - 1));

        // Distance traveled
        let mut distance = Vec3::distance(ray.origin, current_position);

        loop {
            {
                // Check block
                let block = self.scene.block(current_block)?;
                if block != Block::Air {
                    let (face, uv) = face_and_uv(current_position, current_block);
                    let normal = face.normal();

                    // Check direction against normal to avoid hitting back faces
                    if normal.dot(*ray.direction) < 0.0 {
                        return Some(lux::RayHit {
                            material: self.textures.sample(block, face, uv),
                            position: current_position,
                            normal,
                            distance,
                        });
                    }
                }

                // Find next edge over all 3 axes
                let (time, delta) = [0, 1, 2]
                    .into_iter()
                    .map(|i| {
                        let (time, delta_scalar) = time_to_edge(
                            current_position.to_array()[i],
                            current_block.to_array()[i],
                            ray.direction.to_array()[i],
                        );

                        let mut delta = IVec3::ZERO;
                        delta[i] = delta_scalar;

                        (time, delta)
                    })
                    .min_by(|(a_time, _), (b_time, _)| a_time.partial_cmp(b_time).unwrap())
                    .unwrap();

                // Step
                current_position += ray.direction * time;
                distance += time;
                current_block += delta;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Face {
    XNeg,
    XPos,
    YNeg,
    YPos,
    ZNeg,
    ZPos,
}

impl Face {
    fn normal(&self) -> Dir3 {
        match self {
            Face::XNeg => -Dir3::X,
            Face::XPos => Dir3::X,
            Face::YNeg => -Dir3::Y,
            Face::YPos => Dir3::Y,
            Face::ZNeg => -Dir3::Z,
            Face::ZPos => Dir3::Z,
        }
    }
}
