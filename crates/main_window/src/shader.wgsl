struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) texture_coordinates: vec2<f32>,
    @location(2) opacity: f32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coordinates: vec2<f32>,
    @location(1) opacity: f32,
}

@group(0) @binding(0) var sprite_texture: texture_2d<f32>;
@group(0) @binding(1) var sprite_sampler: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    return VertexOutput(
        vec4<f32>(input.position, 0.0, 1.0),
        input.texture_coordinates,
        input.opacity,
    );
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(sprite_texture, sprite_sampler, input.texture_coordinates) * input.opacity;
}

struct RectangleVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct RectangleVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_rectangle(input: RectangleVertexInput) -> RectangleVertexOutput {
    return RectangleVertexOutput(
        vec4<f32>(input.position, 0.0, 1.0),
        input.color,
    );
}

@fragment
fn fs_rectangle(input: RectangleVertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}