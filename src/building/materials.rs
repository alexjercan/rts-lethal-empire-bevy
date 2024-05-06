use bevy::{prelude::*, render::render_resource::{AsBindGroup, ShaderRef}};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub(super) struct ValidBuildingToolMaterial {
    #[uniform(0)]
    pub valid: u32,
}

impl Material for ValidBuildingToolMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/valid_building.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
