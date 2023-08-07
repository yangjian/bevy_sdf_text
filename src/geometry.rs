use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;

/// A square on the `XZ` plane centered at the origin.
#[derive(Debug, Copy, Clone)]
pub struct PlaneWithUV {
    pub size: [f32; 2],
    pub p00_uv: [f32; 2],
    pub p11_uv: [f32; 2],
}

impl Default for PlaneWithUV {
    fn default() -> Self {
        PlaneWithUV {
            size: [1.0, 1.0],
            p00_uv: [0.0, 0.0],
            p11_uv: [1.0, 1.0],
        }
    }
}

impl From<PlaneWithUV> for Mesh {
    fn from(plane: PlaneWithUV) -> Self {
        let (min_x, max_x) = (-0.5 * plane.size[0], 0.5 * plane.size[0]);
        let (min_y, max_y) = (-0.5 * plane.size[1], 0.5 * plane.size[1]);
        let up = Vec3::Y.to_array();

        let positions: Vec<[f32; 3]> = vec![
            [min_x, min_y, 0.0],
            [max_x, min_y, 0.0],
            [max_x, max_y, 0.0],
            [min_x, max_y, 0.0],
        ];
        let uvs: Vec<[f32; 2]> = vec![
            [plane.p00_uv[0], plane.p00_uv[1]],
            [plane.p11_uv[0], plane.p00_uv[1]],
            [plane.p11_uv[0], plane.p11_uv[1]],
            [plane.p00_uv[0], plane.p11_uv[1]],
        ];
        let normals: Vec<[f32; 3]> = vec![up, up, up, up];
        let indices: Vec<u16> = vec![0, 1, 2, 0, 2, 3];

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U16(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
