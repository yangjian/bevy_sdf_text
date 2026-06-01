#import bevy_pbr::forward_io::VertexOutput

struct SdfRectMaterial {
    size: vec2<f32>,
    border_radius: f32,
    border_size: f32,
    border_color: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material: SdfRectMaterial;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let half_size = material.size * 0.5;
    let distance = distanceOfRoundedBox(mesh.uv, half_size, material.border_radius);
    let delta_per_pixel = length(vec2(dpdxFine(distance), dpdyFine(distance)));

    let low = -(material.border_size + 0.5) * delta_per_pixel;
    let high = low + delta_per_pixel;
    let t0 = smoothstep(low, high, distance);

    let mesh_color = straight_to_premultiplied(mesh.color);
    let border_color = straight_to_premultiplied(material.border_color);
    let color = mix(mesh_color, border_color, t0);

    let t1 = smoothstep(-delta_per_pixel * 0.5, delta_per_pixel * 0.5, distance);
    return mix(color, vec4<f32>(0.0), t1);
}

fn distanceOfRoundedBox(pos: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let q = abs(pos) - half_size + radius;
    let outside_distance = length(max(q, vec2<f32>(0.0)));
    let inside_distance = min(max(q.x, q.y), 0.0);
    return outside_distance + inside_distance - radius;
}

// Helper function to convert straight alpha to premultiplied alpha
fn straight_to_premultiplied(color: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(color.rgb * color.a, color.a);
}
