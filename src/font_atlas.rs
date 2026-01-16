use std::collections::HashMap;
use std::io::Cursor;

use ab_glyph::FontArc;
use anyhow::{anyhow, Result};
use bevy::asset::{AssetId, RenderAssetUsages};
use bevy::image::{Image, ImageSampler};
use bevy::prelude::{AlphaMode, Assets, Handle, Resource};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use image::{
    DynamicImage, EncodableLayout, ImageBuffer, ImageFormat, ImageReader, Rgba, Rgba32FImage,
};
use serde::{Deserialize, Serialize};

use crate::{GlyphMetrics, SdfFont, SdfTextMaterial, SdfTextMaterialUniform};

pub type Rgba16Image = ImageBuffer<Rgba<u16>, Vec<u16>>;

#[derive(Default, Debug, Resource)]
pub struct SdfFontAtlasRes {
    atlases: HashMap<AssetId<SdfFont>, SdfFontAtlas>,
}

impl SdfFontAtlasRes {
    pub fn font_atlas(&self, handle: AssetId<SdfFont>) -> Option<&SdfFontAtlas> {
        self.atlases.get(&handle)
    }

    pub fn insert(
        &mut self,
        id: AssetId<SdfFont>,
        font: &SdfFont,
        textures: &mut Assets<Image>,
        materials: &mut Assets<SdfTextMaterial>,
    ) -> Result<()> {
        let ab_font: FontArc = FontArc::try_from_vec(font.data.clone())?;

        let atlas = &font.atlas;
        let image = atlas.to_f32_image()?;
        let (width, height) = image.dimensions();
        let image_data = image.as_bytes().to_owned();

        let mut image = Image::new(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            image_data,
            TextureFormat::Rgba32Float,
            RenderAssetUsages::RENDER_WORLD,
        );
        image.sampler = ImageSampler::linear();

        let texture = textures.add(image);
        let material = materials.add(SdfTextMaterial {
            uniform: SdfTextMaterialUniform::new(atlas.px_range()),
            atlas_texture: texture.clone(),
            alpha_mode: AlphaMode::Premultiplied,
        });

        self.atlases.insert(
            id,
            SdfFontAtlas {
                ab_font,
                params: atlas.params,
                grid_size: atlas.grid_size,
                glyphs: atlas.glyphs.clone(),
                texture,
                material,
            },
        );

        Ok(())
    }
}

#[derive(Debug)]
pub struct SdfFontAtlas {
    pub ab_font: FontArc,
    pub params: SdfAtlasParams,
    pub grid_size: [u32; 2],
    pub glyphs: HashMap<char, SdfAtlasGlyph>,
    pub texture: Handle<Image>,
    pub material: Handle<SdfTextMaterial>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SdfAtlasParams {
    pub char_size: u32,
    pub char_gap: u32,
    pub px_range: u32,
    pub distance_scale_offset: [f32; 2],
}

impl Default for SdfAtlasParams {
    fn default() -> Self {
        Self {
            char_size: 64,
            char_gap: 8,
            px_range: 4,
            distance_scale_offset: [32.0, 0.5],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SdfAtlasGlyph {
    pub char: char,
    pub metrics: GlyphMetrics,
    pub margin_ratio: [f32; 2],
    pub tex_coord_p00: [f32; 2],
    pub tex_coord_p11: [f32; 2],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SdfAtlas {
    pub params: SdfAtlasParams,
    pub chars: Vec<char>,
    pub grid_size: [u32; 2],
    pub glyphs: HashMap<char, SdfAtlasGlyph>,
    pub image_size: [u32; 2],
    pub png_data: Vec<u8>,
}

impl SdfAtlas {
    pub fn px_range(&self) -> f32 {
        self.params.px_range as f32
    }

    pub fn get_glyph(&self, char: char) -> Option<&SdfAtlasGlyph> {
        self.glyphs.get(&char)
    }

    pub fn to_image(&self) -> Result<Rgba16Image> {
        let buf = Cursor::new(&self.png_data);
        let img = ImageReader::with_format(buf, ImageFormat::Png).decode()?;
        match img {
            DynamicImage::ImageRgba16(p) => Ok(p),
            _ => Err(anyhow!("expect but not RGBA16")),
        }
    }

    pub fn to_f32_image(&self) -> Result<Rgba32FImage> {
        let [scale, offset] = self.params.distance_scale_offset;
        let image = self.to_image()?;
        let (width, height) = image.dimensions();
        let mut buffer: Rgba32FImage = Rgba32FImage::new(width, height);
        for (to, from) in buffer.pixels_mut().zip(image.pixels()) {
            to[0] = (from[0] as f32 / u16::MAX as f32 - offset) * scale;
            to[1] = (from[1] as f32 / u16::MAX as f32 - offset) * scale;
            to[2] = (from[2] as f32 / u16::MAX as f32 - offset) * scale;
            to[3] = (from[3] as f32 / u16::MAX as f32 - offset) * scale;
        }
        Ok(buffer)
    }
}

pub fn printable_ascii_chars() -> Vec<char> {
    (33..=126_u32).filter_map(char::from_u32).collect()
}
