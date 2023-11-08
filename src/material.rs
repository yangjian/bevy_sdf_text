use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};

pub const SDF_TEXT_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(20704338815944035659);
pub const SDF_RECT_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(70563445263975664926);

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct SdfTextMaterialUniform {
    pub px_range: f32,
}

#[derive(Asset, AsBindGroup, TypeUuid, TypePath, Debug, Clone)]
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

#[derive(Asset, AsBindGroup, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "59697CD1-6E29-40E4-A878-75DF4F19B5FF"]
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
