use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct SdfTextMaterialUniform {
    pub px_range: f32,
    pub fg_color: Color,
    pub bg_color: Color,
}

#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "6ad7223b-7ce9-4083-818b-84ed1b5ca806"]
pub struct SdfTextMaterial {
    #[uniform(0)]
    pub uniform: SdfTextMaterialUniform,

    #[texture(1)]
    #[sampler(2)]
    pub atlas_texture: Handle<Image>,

    pub alpha_mode: AlphaMode,
}

impl Material for SdfTextMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sdf_text_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct SdfRectMaterialUniform {
    pub px_range: f32,
    pub fg_color: Color,
    pub bg_color: Color,
}

#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "6ad7223b-7ce9-4083-818b-84ed1b5ca806"]
pub struct SdfRectMaterial {
    #[uniform(0)]
    pub uniform: SdfRectMaterialUniform,

    pub alpha_mode: AlphaMode,
}

impl Material for SdfRectMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/sdf_rect_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}
