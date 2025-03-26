use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};

pub const SDF_TEXT_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(20704338815944035659);
pub const SDF_RECT_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(70563445263975664926);

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
}
