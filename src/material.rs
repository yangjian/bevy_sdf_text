use bevy::asset::uuid_handle;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

pub const SDF_TEXT_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("ED699522-1AA7-4DA0-871C-AAD7D201A821");
pub const SDF_RECT_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("555686D4-71A7-4BDD-B625-124934555F37");

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct SdfTextMaterialUniform {
    pub px_range: f32,
    _padding0: u32,
    _padding1: u32,
    _padding2: u32,
}

impl SdfTextMaterialUniform {
    pub fn new(px_range: f32) -> Self {
        Self {
            px_range,
            _padding0: 0,
            _padding1: 0,
            _padding2: 0,
        }
    }
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
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
        ShaderRef::Handle(SDF_TEXT_SHADER_HANDLE)
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct SdfRectMaterialUniform {
    pub size: Vec2,
    pub border_radius: f32,
    pub border_pixels: f32,
    pub border_color: Vec4,
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct SdfRectMaterial {
    #[uniform(0)]
    pub uniform: SdfRectMaterialUniform,

    pub alpha_mode: AlphaMode,
}

impl Material for SdfRectMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(SDF_RECT_SHADER_HANDLE)
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    fn depth_bias(&self) -> f32 {
        -1e-3
    }
}
