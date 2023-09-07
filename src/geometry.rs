use std::collections::HashMap;

use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;

use crate::{PositionedSection, SdfAtlasGlyph};

pub fn build_sdf_text_mesh(
    section: &PositionedSection,
    atlas_glyphs: &HashMap<char, SdfAtlasGlyph>,
) -> Mesh {
    let n = section.glyphs.len();
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(n * 4);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n * 4);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(n * 4);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(n * 4);
    let mut indices: Vec<u16> = Vec::with_capacity(n * 6);

    let up = Vec3::Y.to_array();

    for glyph in section.glyphs.iter() {
        let atlas_glyph = match atlas_glyphs.get(&glyph.char) {
            Some(o) => o,
            None => continue,
        };

        let n = positions.len() as u16;

        let bbox = glyph.bbox_with_margin(atlas_glyph.margin_ratio);
        let (x_min, x_max) = (bbox.min.x, bbox.max.x);
        let (y_min, y_max) = (bbox.min.y, bbox.max.y);
        positions.push([x_min, y_min, 0.0]);
        positions.push([x_max, y_min, 0.0]);
        positions.push([x_max, y_max, 0.0]);
        positions.push([x_min, y_max, 0.0]);

        for _ in 0..4 {
            normals.push(up); // TODO: reduce memory usage
        }

        for _ in 0..4 {
            colors.push(section.color.as_rgba_f32()); // TODO: reduce memory usage
        }

        uvs.push([atlas_glyph.tex_coord_p00[0], atlas_glyph.tex_coord_p11[1]]);
        uvs.push(atlas_glyph.tex_coord_p11);
        uvs.push([atlas_glyph.tex_coord_p11[0], atlas_glyph.tex_coord_p00[1]]);
        uvs.push(atlas_glyph.tex_coord_p00);

        indices.push(n);
        indices.push(n + 1);
        indices.push(n + 2);
        indices.push(n);
        indices.push(n + 2);
        indices.push(n + 3);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U16(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}

/// A square on the `XZ` plane centered at the origin.
#[derive(Debug, Copy, Clone)]
pub struct PlaneWithUV {
    pub rect: Rect,
    pub color: Color,
    pub p00_uv: [f32; 2],
    pub p11_uv: [f32; 2],
}

impl Default for PlaneWithUV {
    fn default() -> Self {
        PlaneWithUV {
            rect: Rect {
                min: Vec2::ZERO,
                max: Vec2::ONE,
            },
            color: Default::default(),
            p00_uv: [0.0, 0.0],
            p11_uv: [1.0, 1.0],
        }
    }
}

impl From<PlaneWithUV> for Mesh {
    fn from(plane: PlaneWithUV) -> Self {
        let [min_x, min_y] = plane.rect.min.to_array();
        let [max_x, max_y] = plane.rect.max.to_array();
        let up = Vec3::Y.to_array();
        let color = plane.color.as_rgba_f32();

        let positions: Vec<[f32; 3]> = vec![
            [min_x, min_y, 0.0],
            [max_x, min_y, 0.0],
            [max_x, max_y, 0.0],
            [min_x, max_y, 0.0],
        ];
        let normals: Vec<[f32; 3]> = vec![up, up, up, up];
        let colors: Vec<[f32; 4]> = vec![color, color, color, color];
        let uvs: Vec<[f32; 2]> = vec![
            [plane.p00_uv[0], plane.p00_uv[1]],
            [plane.p11_uv[0], plane.p00_uv[1]],
            [plane.p11_uv[0], plane.p11_uv[1]],
            [plane.p00_uv[0], plane.p11_uv[1]],
        ];
        let indices: Vec<u16> = vec![0, 1, 2, 0, 2, 3];

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U16(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
