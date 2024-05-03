use std::num::NonZeroU32;

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

const MAX_TEXTURE_COUNT: usize = 4;

#[derive(Asset, TypePath, Debug, Clone)]
pub struct TilemapMaterial {
    size: UVec2,
    textures: Vec<Handle<Image>>,
    mapping: Vec<u32>,
}

impl TilemapMaterial {
    pub fn new(size: UVec2, textures: Vec<Handle<Image>>, mapping: Vec<u32>) -> Self {
        Self {
            size,
            textures,
            mapping,
        }
    }
}

impl AsBindGroup for TilemapMaterial {
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

        let n = MAX_TEXTURE_COUNT;
        let textures = vec![&fallback_image.texture_view; n];

        let mut textures: Vec<_> = textures.into_iter().map(|texture| &**texture).collect();

        for (id, image) in images.into_iter().enumerate() {
            textures[id] = &*image.texture_view;
        }

        let mapping = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("tilemap_material_mapping"),
            contents: &self
                .mapping
                .iter()
                .flat_map(|kind| bytemuck::bytes_of(kind).to_vec())
                .collect::<Vec<u8>>(),
            usage: BufferUsages::STORAGE,
        });

        let size = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("tilemap_material_size"),
            contents: &bytemuck::bytes_of(&self.size).to_vec(),
            usage: BufferUsages::UNIFORM,
        });

        let bind_group = render_device.create_bind_group(
            "tilemap_material_bind_group",
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
            // @group(2) @binding(3) var<uniform> size: vec2<u32>;
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

impl Material for TilemapMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/tilemap.wgsl".into()
    }
}
