#include("shader_tools.wgsl");

 

 struct Rules {
    b0: u32,
    b1: u32,
    b2: u32,
    b3: u32,
    b4: u32,
    b5: u32,
    b6: u32,
    b7: u32,
    s0: u32,
    s1: u32,
    s2: u32,
    s3: u32,
    s4: u32,
    s5: u32,
    s6: u32,
    s7: u32,
    
 };

fn compute_cell(val: f32, sum: f32, rules: Rules) -> f32 {
   
    var born = ( ((rules.b0 == 1u) && close(sum,1)) || ((rules.b1 == 1u) && close(sum,2)) || ((rules.b2  == 1u) && close(sum,3)) || ((rules.b3== 1u) && close(sum,4)) || ((rules.b4== 1u) && close(sum,5)) || ((rules.b5== 1u) && close(sum,6))  || ((rules.b6== 1u) && close(sum,7)) || ((rules.b7== 1u) && close(sum,8))   );
    var stay_alive = ( ((rules.s0 == 1u) && close(sum,1)) || ((rules.s1== 1u) && close(sum,2)) || ((rules.s2== 1u) && close(sum,3)) || ((rules.s3== 1u) && close(sum,4)) || ((rules.s4== 1u) && close(sum,5)) || ((rules.s5== 1u) && close(sum,6))  || ((rules.s6== 1u) && close(sum,7)) || ((rules.s7== 1u) && close(sum,8))   );


    
    
    if (close(val,0) && born) {
        return 1.0;
    }
    if (close(val,1) && stay_alive) {
        return 1.0;
    }
    return 0.0;
 }

@group(0) @binding(0) var input_texture : texture_2d<f32>;
@group(0) @binding(1) var output_texture : texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> rules : Rules;

@compute @workgroup_size(16, 16)
fn totalistic_main(
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

    let me      =  get_pixel_wrap(coords,dimensions,input_texture).r;
    let left    =  get_pixel_wrap(c_left,dimensions,input_texture).r;    
    let right   =  get_pixel_wrap(c_right,dimensions,input_texture).r;    
    let up      =  get_pixel_wrap(c_up,dimensions,input_texture).r;    
    let down    =  get_pixel_wrap(c_down,dimensions,input_texture).r;    

    let l_up    =  get_pixel_wrap(c_left_up,dimensions,input_texture).r;    
    let r_up    =  get_pixel_wrap(c_right_up,dimensions,input_texture).r;    
    let l_down  =  get_pixel_wrap(c_left_down,dimensions,input_texture).r;    
    let r_down  =  get_pixel_wrap(c_right_down,dimensions,input_texture).r;    

    let sum = left + right + up + down + l_up + r_up + l_down + r_down;
    let cell =  compute_cell(me, sum, rules);
    let new_color = vec4<f32>(cell,cell,cell,1.0);

    textureStore(output_texture, coords.xy, new_color);
}