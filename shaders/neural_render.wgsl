// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct Camera {
    position: vec3<f32>,
    zoom: f32,
};

struct RenderParams {
    window_size: vec2<i32>,
    sim_size: vec2<i32>,
};

fn close(a: f32, b: i32) -> bool {
    return abs(a - f32(b)) < 0.1;
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(0) @binding(2)
var t_diffuse2: texture_2d<f32>;
@group(0) @binding(3)
var s_diffuse2: sampler;
@group(1) @binding(0)
var<uniform> cam: Camera;
@group(1) @binding(1)
var<uniform> render_params: RenderParams;

fn cam_to_tex_coords(cam: Camera, p: vec2<f32>) -> vec2<f32> {
    let cam_rect_size = 1.0/cam.zoom;
    let cx = (cam.position.x + 1.0) / 2.0;
    let cy = 1.0 - ((cam.position.y + 1.0) / 2.0);
    let x = cx - (cam_rect_size/2.0) + (p.x*cam_rect_size);
    let y = cy - (cam_rect_size/2.0) + (p.y*cam_rect_size);
    return vec2<f32>(x,y);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dimensions = render_params.sim_size;
    let cam2tex = cam_to_tex_coords(cam,in.tex_coords.xy);
    let cell = textureSample(t_diffuse, s_diffuse, cam2tex);
    let c = cell.rrra;
    return c;
}