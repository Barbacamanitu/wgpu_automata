// Vertex shader

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] tex_coords: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
};

struct Camera {
    pos: vec3<f32>;
    zoom: f32;
};

struct RenderParams {
    window_size: vec2<i32>;
    sim_size: vec2<i32>;
};

fn close(a: f32, b: i32) -> bool {
    return abs(a - f32(b)) < 0.1;
}

[[stage(vertex)]]
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;
[[group(0), binding(2)]]
var t_diffuse2: texture_2d<f32>;
[[group(0), binding(3)]]
var s_diffuse2: sampler;
[[group(1), binding(0)]]
var<uniform> cam: Camera;
[[group(1), binding(1)]]
var<uniform> render_params: RenderParams;


[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let alive = vec4<f32>(0.09,0.47,0.0,1.0);
    let dead = vec4<f32>(0.0,0.0,0.0,1.0);
    let grid = vec4<f32>(0.45,0.45,0.45,1.0);
    let dimensions = render_params.sim_size;
    

    let in_coords = vec2<f32>(in.tex_coords.x + (cam.pos.x/f32(dimensions.x)),in.tex_coords.y + (cam.pos.y/f32(dimensions.y)));
    let in_coords_scaled = in_coords * cam.zoom;
   
   
    let cell = textureSample(t_diffuse, s_diffuse, in_coords_scaled);
    
    
    if (close(cell.r,1)) {
        return alive;
    } else {
        return dead;
    }
}