use crate::{AppState, AssetsState, screens::ScreenSetup};
use bevy::{
    asset::RenderAssetUsages,
    pbr::{ExtendedMaterial, MaterialExtension},
    platform::collections::HashSet,
    prelude::*,
    render::{
        mesh::{Indices, MeshTag, PrimitiveTopology},
        render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat},
    },
};
use bevy_asset_loader::prelude::*;

pub const WORLD_SIZE: usize = 15;
const WORLD_BLOCK_COUNT: usize = WORLD_SIZE * WORLD_SIZE * WORLD_SIZE;

pub fn plugin(app: &mut App) {
    app.add_plugins(MaterialPlugin::<
        ExtendedMaterial<StandardMaterial, BlockExtension>,
    >::default());

    // Setup and cleanup
    app.add_systems(OnEnter(AppState::Game), setup.after(ScreenSetup));
    app.add_systems(OnExit(AppState::Game), cleanup);

    // Assets
    app.configure_loading_state(
        LoadingStateConfig::new(AssetsState::Loading)
            .load_collection::<WorldAssets>()
            .finally_init_resource::<WorldAssetsDyn>(),
    );

    // Update world
    app.add_systems(PostUpdate, update_world.run_if(in_state(AppState::Game)));
}

#[derive(AssetCollection, Resource)]
pub struct WorldAssets {
    #[asset(
        paths(
            "blocks/000_dirt.png",
            "blocks/001_stone.png",
            "blocks/002_sand.png",
            "blocks/003_grass_side.png",
            "blocks/004_grass_top.png",
            "blocks/005_wood.png",
            "blocks/006_leaves.png",
            "blocks/007_water.png"
        ),
        collection(typed)
    )]
    pub block_images: Vec<Handle<Image>>,

    #[expect(unused)] // Only place this here to ensure the shader is loaded
    #[asset(path = "shaders/block.wgsl")]
    block_shader: Handle<Shader>,
}

#[derive(Resource)]
struct WorldAssetsDyn {
    block_mesh: Handle<Mesh>,
    block_material: Handle<ExtendedMaterial<StandardMaterial, BlockExtension>>,
}

impl FromWorld for WorldAssetsDyn {
    fn from_world(world: &mut World) -> Self {
        Self {
            block_mesh: {
                let mut meshes = world.resource_mut::<Assets<Mesh>>();
                meshes.add(block_mesh())
            },
            block_material: {
                //
                let mut array_texture = Vec::new();
                let (mut size, mut layers) = (0, 0);
                let world_assets = world.resource::<WorldAssets>();
                let images = world.resource::<Assets<Image>>();
                for handle in &world_assets.block_images {
                    let image = images.get(handle).unwrap();
                    array_texture.extend_from_slice(image.data.as_ref().unwrap());
                    size = image.width();
                    layers += 1;
                }

                //
                let mut images = world.resource_mut::<Assets<Image>>();
                let blocks = images.add(Image::new(
                    Extent3d {
                        width: size,
                        height: size,
                        depth_or_array_layers: layers,
                    },
                    TextureDimension::D2,
                    array_texture,
                    TextureFormat::bevy_default(),
                    RenderAssetUsages::RENDER_WORLD,
                ));

                //
                // TODO: Create two of these, one for opaque/mask and one for blend
                let mut materials = world
                    .resource_mut::<Assets<ExtendedMaterial<StandardMaterial, BlockExtension>>>();
                materials.add(ExtendedMaterial {
                    base: StandardMaterial {
                        alpha_mode: AlphaMode::Blend,
                        reflectance: 0.1,
                        ..default()
                    },
                    extension: BlockExtension { blocks },
                })
            },
        }
    }
}

fn setup(mut commands: Commands) {
    commands.insert_resource(BloxWorld::from_scene(&default_scene()));
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<BloxWorld>();
}

fn update_world(
    mut commands: Commands,
    mut world: ResMut<BloxWorld>,
    mut tags: Query<&mut MeshTag>,
    world_assets: Res<WorldAssetsDyn>,
) {
    world.update(&mut commands, &mut tags, &world_assets);
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
struct BlockExtension {
    #[texture(100, dimension = "2d_array")]
    #[sampler(101)]
    blocks: Handle<Image>,
}

impl MaterialExtension for BlockExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/block.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/block.wgsl".into()
    }
}

fn block_mesh() -> Mesh {
    let min = -0.5;
    let max = 0.5;

    let vertices = &[
        // Front
        ([min, min, max], [0.0, 0.0, 1.0], [0.0, 1.0]),
        ([max, min, max], [0.0, 0.0, 1.0], [1.0, 1.0]),
        ([max, max, max], [0.0, 0.0, 1.0], [1.0, 0.0]),
        ([min, max, max], [0.0, 0.0, 1.0], [0.0, 0.0]),
        // Back
        ([min, max, min], [0.0, 0.0, -1.0], [1.0, 0.0]),
        ([max, max, min], [0.0, 0.0, -1.0], [0.0, 0.0]),
        ([max, min, min], [0.0, 0.0, -1.0], [0.0, 1.0]),
        ([min, min, min], [0.0, 0.0, -1.0], [1.0, 1.0]),
        // Right
        ([max, min, min], [1.0, 0.0, 0.0], [1.0, 1.0]),
        ([max, max, min], [1.0, 0.0, 0.0], [1.0, 0.0]),
        ([max, max, max], [1.0, 0.0, 0.0], [0.0, 0.0]),
        ([max, min, max], [1.0, 0.0, 0.0], [0.0, 1.0]),
        // Left
        ([min, min, max], [-1.0, 0.0, 0.0], [1.0, 1.0]),
        ([min, max, max], [-1.0, 0.0, 0.0], [1.0, 0.0]),
        ([min, max, min], [-1.0, 0.0, 0.0], [0.0, 0.0]),
        ([min, min, min], [-1.0, 0.0, 0.0], [0.0, 1.0]),
        // Top
        ([max, max, min], [0.0, 1.0, 0.0], [1.0, 0.0]),
        ([min, max, min], [0.0, 1.0, 0.0], [0.0, 0.0]),
        ([min, max, max], [0.0, 1.0, 0.0], [0.0, 1.0]),
        ([max, max, max], [0.0, 1.0, 0.0], [1.0, 1.0]),
        // Bottom
        ([max, min, max], [0.0, -1.0, 0.0], [1.0, 0.0]),
        ([min, min, max], [0.0, -1.0, 0.0], [0.0, 0.0]),
        ([min, min, min], [0.0, -1.0, 0.0], [0.0, 1.0]),
        ([max, min, min], [0.0, -1.0, 0.0], [1.0, 1.0]),
    ];

    let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
    let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
    let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

    let indices = Indices::U32(vec![
        0, 1, 2, 2, 3, 0, // front
        4, 5, 6, 6, 7, 4, // back
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // top
        20, 21, 22, 22, 23, 20, // bottom
    ]);

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(indices)
}

#[derive(Debug)]
pub struct BloxScene {
    blocks: Box<[Block; WORLD_BLOCK_COUNT]>,
}

impl BloxScene {
    pub fn empty() -> Self {
        Self {
            blocks: vec![Block::Air; WORLD_BLOCK_COUNT].try_into().unwrap(),
        }
    }

    pub fn block(&self, pos: IVec3) -> Option<Block> {
        linearize(pos).map(|i| self.blocks[i])
    }

    pub fn set_block(&mut self, pos: IVec3, block: Block) {
        if let Some(i) = linearize(pos) {
            self.blocks[i] = block;
        }
    }
}

#[derive(Debug, Resource)]
pub struct BloxWorld {
    blocks: Box<[BlockInstance; WORLD_BLOCK_COUNT]>,
    dirty: Dirty,
}

impl BloxWorld {
    pub fn empty() -> Self {
        Self {
            blocks: vec![BlockInstance::default(); WORLD_BLOCK_COUNT]
                .try_into()
                .unwrap(),
            dirty: Dirty::Blocks(Vec::new()),
        }
    }

    pub fn from_scene(scene: &BloxScene) -> Self {
        let mut world = Self::empty();
        world.load_scene(scene);
        world
    }

    pub fn to_scene(&self) -> BloxScene {
        let mut scene = BloxScene::empty();
        for i in 0..(WORLD_BLOCK_COUNT) {
            scene.blocks[i] = self.blocks[i].block;
        }
        scene
    }

    pub fn block(&self, pos: IVec3) -> Option<Block> {
        linearize(pos).map(|i| self.blocks[i].block)
    }

    pub fn set_block(&mut self, pos: IVec3, block: Block) {
        if let Some(i) = linearize(pos) {
            self.blocks[i].block = block;
            self.dirty.push(pos);
        }
    }

    pub fn load_scene(&mut self, scene: &BloxScene) {
        for i in 0..(WORLD_BLOCK_COUNT) {
            self.blocks[i].block = scene.blocks[i];
        }
        self.dirty = Dirty::All;
    }

    // TODO: raycast ray to (block position or ground position) + hit data or none

    fn update(
        &mut self,
        commands: &mut Commands,
        tags: &mut Query<&mut MeshTag>,
        world_assets: &Res<WorldAssetsDyn>,
    ) {
        match &self.dirty {
            Dirty::Blocks(positions) => {
                let mut positions_and_neighbors = HashSet::new();
                for pos in positions {
                    positions_and_neighbors.insert(*pos);
                    for offset in &[
                        IVec3::new(-1, 0, 0),
                        IVec3::new(1, 0, 0),
                        IVec3::new(0, -1, 0),
                        IVec3::new(0, 1, 0),
                        IVec3::new(0, 0, -1),
                        IVec3::new(0, 0, 1),
                    ] {
                        positions_and_neighbors.insert(*pos + *offset);
                    }
                }
                for pos in positions_and_neighbors {
                    self.update_block(pos, commands, tags, world_assets);
                }
            }
            Dirty::All => {
                for x in 0..WORLD_SIZE as i32 {
                    for y in 0..WORLD_SIZE as i32 {
                        for z in 0..WORLD_SIZE as i32 {
                            self.update_block(IVec3::new(x, y, z), commands, tags, world_assets);
                        }
                    }
                }
            }
        }
        self.dirty = Dirty::Blocks(Vec::new());
    }

    fn update_block(
        &mut self,
        pos: IVec3,
        commands: &mut Commands,
        tags: &mut Query<&mut MeshTag>,
        world_assets: &Res<WorldAssetsDyn>,
    ) {
        let Some(i) = linearize(pos) else {
            return;
        };

        if self.blocks[i].block == Block::Air {
            if let Some(entity) = self.blocks[i].entity.take() {
                commands.entity(entity).despawn();
            }

            return;
        }

        let neighbors = [
            IVec3::new(-1, 0, 0),
            IVec3::new(1, 0, 0),
            IVec3::new(0, -1, 0),
            IVec3::new(0, 1, 0),
            IVec3::new(0, 0, -1),
            IVec3::new(0, 0, 1),
        ]
        .map(|offset| self.block(pos + offset).unwrap_or(Block::Air));

        let mut tag = self.blocks[i].block as u32;
        for (j, neighbor) in neighbors.into_iter().enumerate() {
            let discard = neighbor.is_solid() || self.blocks[i].block == neighbor;
            tag |= (discard as u32) << (8 + j);
        }

        let mut height = 1.0;
        if self.blocks[i].block == Block::Water && neighbors[3] != Block::Water {
            height = 0.9;
            tag &= !(1 << (8 + 3)); // Don't discard top face
        }

        match self.blocks[i].entity {
            Some(entity) => {
                *tags.get_mut(entity).unwrap() = MeshTag(tag);
            }
            None => {
                let entity = commands
                    .spawn((
                        Name::new("Block"),
                        Transform {
                            translation: pos.as_vec3() + Vec3::new(0.5, height / 2.0, 0.5),
                            scale: Vec3::new(1.0, height, 1.0),
                            ..default()
                        },
                        MeshTag(tag),
                        Mesh3d(world_assets.block_mesh.clone()),
                        MeshMaterial3d(world_assets.block_material.clone()),
                        StateScoped(AppState::Game),
                    ))
                    .id();
                self.blocks[i].entity = Some(entity);
            }
        }
    }
}

#[derive(Debug)]
enum Dirty {
    Blocks(Vec<IVec3>),
    All,
}

impl Dirty {
    fn push(&mut self, pos: IVec3) {
        match self {
            Dirty::Blocks(vec) => vec.push(pos),
            Dirty::All => (),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct BlockInstance {
    block: Block,
    entity: Option<Entity>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Block {
    #[default]
    Air = 0,
    Dirt = 1,
    Stone = 2,
    Sand = 3,
    Grass = 4,
    Wood = 5,
    Leaves = 6,
    Water = 7,
}

impl Block {
    pub fn is_solid(&self) -> bool {
        match self {
            Block::Air | Block::Leaves | Block::Water => false,
            Block::Dirt | Block::Stone | Block::Sand | Block::Grass | Block::Wood => true,
        }
    }
}

fn linearize(pos: IVec3) -> Option<usize> {
    let size = WORLD_SIZE as i32;
    if (0..size).contains(&pos.x) && (0..size).contains(&pos.y) && (0..size).contains(&pos.z) {
        Some((pos.x + pos.y * size + pos.z * size * size) as usize)
    } else {
        None
    }
}

fn default_scene() -> BloxScene {
    let mut scene = BloxScene::empty();

    let size = WORLD_SIZE as i32;

    for x in 0..size {
        for z in 0..size {
            scene.set_block(IVec3::new(x, 0, z), Block::Stone);

            scene.set_block(
                IVec3::new(x, 1, z),
                if (6..=8).contains(&x) && (6..=8).contains(&z) {
                    Block::Water
                } else {
                    Block::Grass
                },
            );

            if x == 0 || x == size - 1 || z == 0 || z == size - 1 {
                scene.set_block(IVec3::new(x, 2, z), Block::Wood);
            }
        }
    }

    scene.set_block(IVec3::new(9, 2, 5), Block::Sand);
    scene.set_block(IVec3::new(9, 3, 5), Block::Sand);

    scene
}
