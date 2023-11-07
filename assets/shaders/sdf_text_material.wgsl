#import bevy_pbr::forward_io::VertexOutput

struct SdfTextMaterial {
    px_range: f32,
};

@group(1) @binding(0)
var<uniform> material: SdfTextMaterial;

@group(1) @binding(1)
var atlas_texture: texture_2d<f32>;

@group(1) @binding(2)
var atlas_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let msd = textureSample(atlas_texture, atlas_sampler, mesh.uv).rgb;
    let sd = median(msd);

    // Very valuable discussion about anti-aliasing:
    // https://github.com/Chlumsky/msdfgen/issues/22#issuecomment-234958005
    let tex_dim = textureDimensions(atlas_texture);
    let dx = dpdxFine(mesh.uv.x) * f32(tex_dim.x);
    let dy = dpdyFine(mesh.uv.y) * f32(tex_dim.y);
    let to_pixels = material.px_range / length(vec2(dx, dy));
    let opacity = clamp((sd - 0.5) * to_pixels + 0.5, 0.0, 1.0);

    return mix(vec4<f32>(0.0), mesh.color, opacity);
}

fn median(color: vec3<f32>) -> f32 {
    return max(min(color.r, color.g), min(max(color.r, color.g), color.b));
}
