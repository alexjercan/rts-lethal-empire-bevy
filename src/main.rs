use std::{collections::HashMap, num::NonZeroU32};

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, AsBindGroupError, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntry,
            BindingType, BufferBindingType, BufferInitDescriptor, BufferUsages, PreparedBindGroup,
            SamplerBindingType, ShaderRef, ShaderStages, TextureSampleType, TextureViewDimension,
            UnpreparedBindGroup,
        },
        renderer::RenderDevice,
        texture::FallbackImage,
    },
};
use bevy_asset_loader::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use itertools::Itertools;
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal, Perlin,
};

#[cfg(feature = "debug")]
use debug::DebugModePlugin;

// Features that I want in my game:
//
// # Version 0.1
// - [x] Tile based map
//   - [x] create a perlin noise generator that can give us noise values for a given area (chunk)
//   - [x] implement a mapper from noise values to tile types
//   - [x] create a custom atlas with all the tile types and textures
//   - [x] write a shader that takes in the tile types (RGB) and the atlas and renders the map
// - [ ] Resources
//   - [ ] implement a system that randomly generates resources on the map
//   - [ ] implement a new tile type for trees and rocks
//   - [ ] add models for trees and rocks and spawn them in the world
//   - [ ] think about a better way than just random for V2
// - [ ] Buildings
//   - [ ] extremely simple buildings that can be placed on the map and give us resources over time
// - [ ] Main Goal
//   - [ ] need to pay quota of resources to the Empire over time
//   - [ ] UI with the timer and quota needed and also how much we have
//   - "TIME LEFT: 10:00" "QUOTA: 500/1000"
//
// # Version 0.2
// - [ ] Tile based map V2
//   - [ ] create a system that can extend the map in any direction
//   - [ ] implement loading and unloading tiles when scrolling trough the map
// - [ ] Resources
// - [ ] Pathfinding
//

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameStates {
    #[default]
    AssetLoading,
    Playing,
}

#[derive(AssetCollection, Resource)]
struct GameAssets {
    #[asset(
        paths(
            "textures/tiles/water.png",
            "textures/tiles/grass.png",
            "textures/tiles/forest.png",
            "textures/tiles/rock.png"
        ),
        collection(typed)
    )]
    tiles: Vec<Handle<Image>>,
}

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
enum TileKind {
    #[default]
    Water,
    Grass,
    Forest,
    Rock,
}

impl TileKind {
    fn from_noise(noise: f64) -> Self {
        match noise {
            n if n < 0.0 => TileKind::Water,
            n if n < 0.2 => TileKind::Grass,
            n if n < 0.4 => TileKind::Forest,
            _ => TileKind::Rock,
        }
    }
}

#[derive(Resource, Deref)]
struct TerrainGenerator(Fbm<Perlin>);

impl Default for TerrainGenerator {
    fn default() -> Self {
        TerrainGenerator(
            Fbm::<Perlin>::new(0)
                .set_frequency(1.0)
                .set_persistence(0.5)
                .set_lacunarity(2.0)
                .set_octaves(14),
        )
    }
}

impl TerrainGenerator {
    fn generate(&self, center: IVec2, size: UVec2) -> Vec<f64> {
        PlaneMapBuilder::new(self.0.clone())
            .set_size(size.x as usize, size.y as usize)
            .set_x_bounds((center.x as f64) * 1.0 - 0.5, (center.x as f64) * 1.0 + 0.5)
            .set_y_bounds((center.y as f64) * 1.0 - 0.5, (center.y as f64) * 1.0 + 0.5)
            .build()
            .into_iter()
            .collect_vec()
    }
}

fn main() {
    let mut app = App::new();

    #[cfg(feature = "debug")]
    app.add_plugins(DebugModePlugin);

    app.add_plugins(DefaultPlugins)
        // TODO: Using PanOrbitCameraPlugin for now, but we will need to create our own camera
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(MaterialPlugin::<BindlessMaterial>::default())
        .init_state::<GameStates>()
        .add_loading_state(
            LoadingState::new(GameStates::AssetLoading)
                .continue_to_state(GameStates::Playing)
                .load_collection::<GameAssets>(),
        )
        .init_resource::<TerrainGenerator>()
        .add_systems(OnEnter(GameStates::Playing), setup)
        .add_systems(Update, (update, update_tilemap).run_if(in_state(GameStates::Playing)))
        .run();
}

#[derive(Asset, TypePath, Debug, Clone)]
struct BindlessMaterial {
    textures: Vec<Handle<Image>>,
    mapping: Vec<TileKind>,
}

const MAX_TEXTURE_COUNT: usize = 4;
const TILEMAP_SIZE: usize = 128;
const TILEMAP_TILE_SIZE: f32 = 16.0;

impl AsBindGroup for BindlessMaterial {
    type Data = ();

    fn as_bind_group(
        &self,
        layout: &BindGroupLayout,
        render_device: &RenderDevice,
        image_assets: &RenderAssets<Image>,
        fallback_image: &FallbackImage,
    ) -> Result<PreparedBindGroup<Self::Data>, AsBindGroupError> {
        let mut images = vec![];
        for handle in self.textures.iter().take(MAX_TEXTURE_COUNT) {
            match image_assets.get(handle) {
                Some(image) => images.push(image),
                None => return Err(AsBindGroupError::RetryNextUpdate),
            }
        }

        let fallback_image = &fallback_image.d2;

        let textures = vec![&fallback_image.texture_view; MAX_TEXTURE_COUNT];

        let mut textures: Vec<_> = textures.into_iter().map(|texture| &**texture).collect();

        for (id, image) in images.into_iter().enumerate() {
            textures[id] = &*image.texture_view;
        }

        let mapping = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("bindless_material_mapping"),
            contents: &self
                .mapping
                .iter()
                .flat_map(|kind| bytemuck::bytes_of(&(*kind as u32)).to_vec())
                .collect::<Vec<u8>>(),
            usage: BufferUsages::STORAGE,
        });

        let size = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("bindless_material_size"),
            contents: &bytemuck::bytes_of(&(TILEMAP_SIZE as u32)).to_vec(),
            usage: BufferUsages::UNIFORM,
        });

        let bind_group = render_device.create_bind_group(
            "bindless_material_bind_group",
            layout,
            &BindGroupEntries::sequential((
                &textures[..],
                &fallback_image.sampler,
                mapping.as_entire_binding(),
                size.as_entire_binding(),
            )),
        );

        Ok(PreparedBindGroup {
            bindings: vec![],
            bind_group,
            data: (),
        })
    }

    fn unprepared_bind_group(
        &self,
        _: &BindGroupLayout,
        _: &RenderDevice,
        _: &RenderAssets<Image>,
        _: &FallbackImage,
    ) -> Result<UnpreparedBindGroup<Self::Data>, AsBindGroupError> {
        // we implement as_bind_group directly because
        panic!("bindless texture arrays can't be owned")
        // or rather, they can be owned, but then you can't make a `&'a [&'a TextureView]` from a vec of them in get_binding().
    }

    fn bind_group_layout_entries(_: &RenderDevice) -> Vec<BindGroupLayoutEntry>
    where
        Self: Sized,
    {
        vec![
            // @group(2) @binding(0) var textures: binding_array<texture_2d<f32>>;
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: NonZeroU32::new(MAX_TEXTURE_COUNT as u32),
            },
            // @group(2) @binding(1) var nearest_sampler: sampler;
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
            // @group(2) @binding(2) var<storage, read> mapping: array<u32>;
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                // count: NonZeroU32::new((TILEMAP_SIZE * TILEMAP_SIZE) as u32),
                count: None,
            },
            // @group(2) @binding(3) var<uniform> size: u32;
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ]
    }
}

impl Material for BindlessMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/bindless_material.wgsl".into()
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BindlessMaterial>>,
    game_assets: Res<GameAssets>,
) {
    // light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera {
            button_orbit: MouseButton::Right,
            button_pan: MouseButton::Middle,
            ..default()
        },
    ));

    // map
    let map_size = UVec2::splat(TILEMAP_SIZE as u32);
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = HashMap::<UVec2, Entity>::new();
    commands.entity(tilemap_entity).with_children(|parent| {
        for x in 0..map_size.x {
            for y in 0..map_size.y {
                let tile_coord = UVec2::new(x, y);
                let tile_entity = parent
                    .spawn((
                        TileCoord(tile_coord),
                        TileKind::default(),
                    ))
                    .id();
                tile_storage.insert(UVec2::new(x, y), tile_entity);
            }
        }
    });
    let tile_size = Vec2::splat(TILEMAP_TILE_SIZE);
    commands.entity(tilemap_entity).insert((
        TilemapSize(map_size),
        TilemapStorage(tile_storage),
        TilemapTileSize(tile_size),
        MaterialMeshBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(
                tile_size.x * map_size.x as f32,
                tile_size.y * map_size.y as f32,
            )),
            material: materials.add(BindlessMaterial {
                textures: game_assets.tiles.clone(),
                mapping: vec![TileKind::Grass; map_size.x as usize * map_size.y as usize],
            }),
            ..Default::default()
        },
    ));
}

#[derive(Component, Deref)]
struct TileCoord(UVec2);

#[derive(Component, Deref)]
struct TilemapSize(UVec2);

#[derive(Component, Deref)]
struct TilemapStorage(HashMap<UVec2, Entity>);

#[derive(Component, Deref)]
struct TilemapTileSize(Vec2);

fn update(
    mut commands: Commands,
    terrain_generator: Res<TerrainGenerator>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    q_tilemap: Query<(&TilemapSize, &TilemapStorage)>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let noisemap = terrain_generator.generate(IVec2::ZERO, UVec2::splat(TILEMAP_SIZE as u32));

        if let Ok((map_size, map_storage)) = q_tilemap.get_single() {
            for (coord, entity) in map_storage.iter() {
                let noise = noisemap[map_size.x as usize * coord.y as usize + coord.x as usize];
                let tile_kind = TileKind::from_noise(noise);
                commands
                    .entity(*entity)
                    .insert(tile_kind);
            }
        }
    }
}

fn update_tilemap(
    mut materials: ResMut<Assets<BindlessMaterial>>,
    q_tilemap: Query<(&TilemapSize, &TilemapStorage, &Handle<BindlessMaterial>)>,
    q_tiles: Query<&TileKind>,
) {
    if let Ok((map_size, map_storage, material)) = q_tilemap.get_single() {
        let material = materials.get_mut(material).unwrap();

        for (coord, tile) in map_storage.iter() {
            let tile_kind = q_tiles.get(*tile).unwrap();
            material.mapping[(map_size.x * coord.y + coord.x) as usize] = *tile_kind;
        }
    }
}

#[cfg(feature = "debug")]
mod debug {
    use bevy::prelude::*;

    use crate::GameStates;

    #[derive(Debug, Default)]
    pub(super) struct DebugModePlugin;

    impl Plugin for DebugModePlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(Update, draw_cursor.run_if(in_state(GameStates::Playing)));
        }
    }

    fn draw_cursor(
        q_camera: Query<(&Camera, &GlobalTransform)>,
        windows: Query<&Window>,
        mut gizmos: Gizmos,
    ) {
        let (camera, camera_transform) = q_camera.single();

        let Some(cursor_position) = windows.single().cursor_position() else {
            return;
        };

        let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
            return;
        };

        let Some(distance) = ray.intersect_plane(Vec3::ZERO, Plane3d::new(Vec3::Y)) else {
            return;
        };
        let point = ray.get_point(distance);

        gizmos.circle(point + Vec3::Y * 0.01, Direction3d::Y, 0.2, Color::WHITE);
    }
}
