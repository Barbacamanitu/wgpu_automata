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



//Transforms camera coordinates to texture coordinates.
fn cam_to_tex_coords(cam: Camera, p: vec2<f32>,r_params: RenderParams) -> vec2<f32> {
    let ssize = r_params.sim_size;
    let rsize = r_params.window_size;
    let ar = f32(rsize.y) / f32(rsize.x);
    let r = 1.0/cam.zoom;
    let cam_rect_size = vec2<f32>(r,r*ar);
    let cx = (cam.position.x + 1.0) / 2.0;
    let cy = 1.0 - ((cam.position.y + 1.0) / 2.0);
    let x = cx - (cam_rect_size.x/2.0) + (p.x*cam_rect_size.x);
    let y = cy - (cam_rect_size.y/2.0) + (p.y*cam_rect_size.y);
    return vec2<f32>(x,y);
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
@group(1) @binding(0)
var<uniform> cam: Camera;
@group(1) @binding(1)
var<uniform> render_params: RenderParams;


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let alive = vec4<f32>(0.09,0.47,0.0,1.0);
    let dead = vec4<f32>(0.0,0.0,0.0,1.0);
    let grid = vec4<f32>(0.15,0.15,0.15,1.0);
    let dimensions = render_params.sim_size;
    
    let cam2tex = cam_to_tex_coords(cam,in.tex_coords.xy,render_params);

    let cell = textureSample(t_diffuse, s_diffuse, cam2tex.xy);
  
    
   
    var c = dead;
    if (close(cell.r,1)) {
        c = alive;
    }
    return c;
}