@group(1) @binding(0)
var<uniform> light: Light;

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ambient_strength = 0.2;
    let ambient_color = vec4<f32>(light.color, 1.0) * ambient_strength;

    let light_dir = normalize(light.position - in.world_position);

    let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse_color = vec4<f32>(light.color, 1.0) * diffuse_strength;

    return vec4<f32>((ambient_color + diffuse_color) * in.color);
}
