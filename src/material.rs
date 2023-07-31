use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct MtsdfMaterialUniform {
    pub px_range: f32,
    pub fg_color: Color,
    pub bg_color: Color,
}

#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "6ad7223b-7ce9-4083-818b-84ed1b5ca806"]
pub struct MtsdfMaterial {
    #[uniform(0)]
    pub uniform: MtsdfMaterialUniform,

    #[texture(1)]
    #[sampler(2)]
    pub atlas_texture: Handle<Image>,

    pub alpha_mode: AlphaMode,
}

impl Material for MtsdfMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/mtsdf_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}
