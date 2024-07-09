struct Camera {
    position: vec3<f32>,
    direction: vec3<f32>,
    up: vec3<f32>,
    fov: f32,
    far: f32,
    screen_width: u32,
    screen_height: u32
};

@group(0) @binding(0)
var<uniform> camera: Camera;
@group(0) @binding(1)
var output_texture: texture_storage_2d<bgra8unorm, write>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    var red = 0.0;
    if id.x > 800 && id.x < 900 && id.y > 800 && id.y < 900 {
        red = 1.0;
    }
    textureStore(output_texture, vec2<u32>(id.x, id.y), vec4<f32>(red, 0.0, 0.0, 1.0));
}
