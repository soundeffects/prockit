struct Camera {
    position: vec3f,
    direction: vec3f,
    up: vec3f,
    fov: f32,
    far: f32,
    screen: vec2f
};

@group(0) @binding(0)
var<uniform> camera: Camera;
@group(0) @binding(1)
var output_texture: texture_storage_2d<bgra8unorm, write>;
@group(0) @binding(2)
var<storage> voxel_store: array<u32>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3u) {
    let centered = vec2f(id.xy) - camera.screen * 0.5;
    let max_dimension = f32(max(camera.screen.x, camera.screen.y));
    let angles = camera.fov * centered / max_dimension;

    let horizontal_cross = cross(camera.direction, camera.up);
    let vertical_cross = cross(horizontal_cross, camera.direction);

    let ray_direction = rotate(rotate(camera.direction, vertical_cross, angles.x), horizontal_cross, angles.y);

    let center = vec3f(0.0, 0.0, 0.0);
    let radius = 1.0;
    let error_tolerance = 0.001;

    var current_position = camera.position;
    var current_distance = distance(current_position, center) - radius;
    var previous_distance = current_distance + 1.0;
    var fill_color = vec4f(0.0, 0.0, 0.0, 1.0);

    while previous_distance > current_distance {        
        current_position += ray_direction * current_distance;
        previous_distance = current_distance;
        current_distance = distance(current_position, center) - radius;

        if fill_color.b < 1.0 {
            fill_color.b += 0.05;
            fill_color.g += 0.01;
        }

        if current_distance < error_tolerance {
            fill_color.g += 0.5;
            fill_color.r += 0.2;
            break;
        }
    }

    textureStore(output_texture, vec2u(id.x, id.y), fill_color);
}

fn rotate(vector: vec3f, axis: vec3f, angle: f32) -> vec3f {
    let q_rot = rotation_quaternion(axis, angle);
    let q_conj = conjugate_quaternion(q_rot);
    let q_vec = vec4f(vector, 0);
    return multiply_quaternions(multiply_quaternions(q_rot, q_vec), q_conj).xyz;
}

fn rotation_quaternion(axis: vec3f, angle: f32) -> vec4f {
    let degrees_to_radians = 0.0174532925; // approximately pi / 180
    let half_angle = (angle * 0.5) * degrees_to_radians;
    return vec4f(axis * sin(half_angle), cos(half_angle));
}

fn multiply_quaternions(q1: vec4f, q2: vec4f) -> vec4f {
    return vec4f(
        (q1.w * q2.x) + (q1.x * q2.w) + (q1.y * q2.z) - (q1.z * q2.y),
        (q1.w * q2.y) - (q1.x * q2.z) + (q1.y * q2.w) + (q1.z * q2.x),
        (q1.w * q2.z) + (q1.x * q2.y) - (q1.y * q2.x) + (q1.z * q2.w),
        (q1.w * q2.w) - (q1.x * q2.x) - (q1.y * q2.y) - (q1.z * q2.z)
    );
}

fn conjugate_quaternion(q: vec4f) -> vec4f {
    return vec4f(-q.x, -q.y, -q.z, q.w);
}