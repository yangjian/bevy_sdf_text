#import bevy_pbr::forward_io::VertexOutput

struct SdfTextMaterial {
    px_range: f32,
    padding0: u32,
    padding1: u32,
    padding2: u32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material: SdfTextMaterial;

@group(#{MATERIAL_BIND_GROUP}) @binding(1)
var atlas_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(2)
var atlas_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Very valuable discussion about anti-aliasing:
    // https://github.com/Chlumsky/msdfgen/issues/22#issuecomment-234958005
    let tex_dim = textureDimensions(atlas_texture);
    // let dx = dpdxFine(mesh.uv.x) * f32(tex_dim.x);
    // let dy = dpdyFine(mesh.uv.y) * f32(tex_dim.y);
    let dx = f32(tex_dim.x) * length(vec2(dpdxFine(mesh.uv.x), dpdyFine(mesh.uv.x)));
    let dy = f32(tex_dim.y) * length(vec2(dpdxFine(mesh.uv.y), dpdyFine(mesh.uv.y)));
    let to_pixels = material.px_range * inverseSqrt(dx * dx + dy * dy);

    let signed_distance = sample_msdf(mesh.uv) - 0.5;
    // let opacity = clamp(signed_distance * to_pixels + 0.5, 0.0, 1.0);
    let opacity = smoothstep(-0.5, 0.5, signed_distance * to_pixels);
    return mix(vec4<f32>(0.0), mesh.color, opacity);
}

fn sample_msdf(texcoord: vec2f) -> f32 {
    let c = textureSample(atlas_texture, atlas_sampler, texcoord);
    return max(min(c.r, c.g), min(max(c.r, c.g), c.b)); // median value
}
