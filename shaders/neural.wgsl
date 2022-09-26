fn get_pixel_wrap(pos: vec2<i32>, dims: vec2<i32>, tex: texture_2d<f32>) -> vec4<f32> {
    let x = (pos.x + dims.x) %dims.x;
    let y = (pos.y +  dims.y) %  dims.y;
    let col: vec4<f32> = textureLoad(tex, vec2<i32>(x,y), 0);
    return col;
 }

fn close(a: f32, b: i32) -> bool {
    return abs(a - f32(b)) < 0.2;
}

fn inverse_gaussian(x: f32) -> f32 {
  return -1.0/pow(2., (0.6*pow(x, 2.)))+1.;
}

fn activation(x: f32) -> f32 {
  return inverse_gaussian(x);
}
 
 struct Filter {
     w0: f32,
     w1: f32,
     w2: f32,
     w3: f32,
     w4: f32,
     w5: f32,
     w6: f32,
     w7: f32,
     w8: f32
 };

@group(0) @binding(0) var input_texture : texture_2d<f32>;
@group(0) @binding(1) var output_texture : texture_storage_2d<rgba8unorm, write>;
@group(1) @binding(0) var<uniform> c_filter : Filter;
@compute @workgroup_size(16,16)
fn main(
  @builtin(global_invocation_id) global_id : vec3<u32>,
) {
    let dimensions = textureDimensions(input_texture);
    let coords = vec2<i32>(global_id.xy);

    if(coords.x >= dimensions.x || coords.y >= dimensions.y) {
        return;
    }
    let x: i32 = coords.x;
    let y: i32 = coords.y;
    
    let c_left = vec2<i32>(x - 1,y);
    let c_up = vec2<i32>(x,y + 1);
    let c_right = vec2<i32>(x + 1,y);
    let c_down = vec2<i32>(x,y - 1);

    let c_left_up = vec2<i32>(x - 1,y + 1);
    let c_right_up = vec2<i32>(x + 1,y + 1);
    let c_left_down = vec2<i32>(x - 1,y - 1);
    let c_right_down = vec2<i32>(x + 1,y - 1);

    let me      =  get_pixel_wrap(coords,dimensions,input_texture);
    let left    =  get_pixel_wrap(c_left,dimensions,input_texture).r;    
    let right   =  get_pixel_wrap(c_right,dimensions,input_texture).r;    
    let up      =  get_pixel_wrap(c_up,dimensions,input_texture).r;    
    let down    =  get_pixel_wrap(c_down,dimensions,input_texture).r;    

    let l_up    =  get_pixel_wrap(c_left_up,dimensions,input_texture).r;    
    let r_up    =  get_pixel_wrap(c_right_up,dimensions,input_texture).r;    
    let l_down  =  get_pixel_wrap(c_left_down,dimensions,input_texture).r;    
    let r_down  =  get_pixel_wrap(c_right_down,dimensions,input_texture).r;    
    let me_r = me.r;

    let conv_filter : array<f32, 9> = array<f32, 9>
    (-0.72, 0.90, -0.68, 
    0.92, 0.68, 0.91, 
    -0.68, 0.9, -0.72);

    let conv = c_filter.w0 * l_up + c_filter.w1 * up + c_filter.w2 * r_up + c_filter.w3 * left + c_filter.w4 * me_r + c_filter.w5 * right + c_filter.w6 * l_down + c_filter.w7 * down + c_filter.w8 * r_down;
    let val = clamp(activation(conv),0.0,1.0);
    var g = me.g;
    var b = me.b;
    if (val > 0.8 && g < 0.5) {
        g = 1.0;
    } else {
        g = g * .99;
    }

    if (g < 0.2) {
        g = g * .5;
    }

 

    let cell = vec4<f32>(val,g,0.0,1.0);
     

    
    
    
    

    textureStore(output_texture, coords.xy, cell);
}