#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var<uniform> color_amount: f32;
@group(2) @binding(1) var<uniform> color: vec4<f32>;
@group(2) @binding(2) var base_color_texture: texture_2d<f32>;
@group(2) @binding(3) var base_color_sampler: sampler;
@group(2) @binding(4) var<uniform> disabled_color: vec4<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var sample = textureSample(base_color_texture, base_color_sampler, mesh.uv);
    var col = mix(disabled_color, color, color_amount);
    return sample * col;
    // return vec4(1.0, 1.0, 1.0, 1.0);
}
