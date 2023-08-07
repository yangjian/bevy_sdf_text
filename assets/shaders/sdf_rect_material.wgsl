#import bevy_pbr::mesh_vertex_output MeshVertexOutput

struct SdfRectMaterial {
    size: vec2<f32>,
    radius: f32,
    color: vec4<f32>,

    border_size: f32,
    border_color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: SdfRectMaterial;

@fragment
fn fragment(mesh: MeshVertexOutput) -> @location(0) vec4<f32> {
    let sd = computeDistance(msd);

    let dx = dpdxFine(mesh.uv.x) * f32(tex_dim.x);
    let dy = dpdyFine(mesh.uv.y) * f32(tex_dim.y);
    let to_pixels = material.px_range / length(vec2(dx, dy));
    let opacity = clamp((sd - 0.5) * to_pixels + 0.5, 0.0, 1.0);

    return mix(material.bg_color, material.fg_color, opacity);
}

fn computeDistance(center: vec2<f32>, size: vec2<f32>, radius: f32) -> f32 {
    return length(max(abs(center) - size + radius, 0.0)) - radius;
}
