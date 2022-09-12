#include("shader_tools.wgsl");

 

 struct Rules {
    born: array<u32, 8>;
    stay_alive: array<u32,8>;
    
 };

fn compute_cell(val: f32, sum: f32, rules: Rules) -> f32 {
   
    var born = ( ((rules.born[0] == 1u) && close(sum,1)) || ((rules.born[1] == 1u) && close(sum,2)) || ((rules.born[2] == 1u) && close(sum,3)) || ((rules.born[3] == 1u) && close(sum,4)) || ((rules.born[4] == 1u) && close(sum,5)) || ((rules.born[5] == 1u) && close(sum,6))  || ((rules.born[6] == 1u) && close(sum,7)) || ((rules.born[7] == 1u) && close(sum,8))   );
    var stay_alive = ( ((rules.stay_alive[0] == 1u) && close(sum,1)) || ((rules.stay_alive[1] == 1u) && close(sum,2)) || ((rules.stay_alive[2] == 1u) && close(sum,3)) || ((rules.stay_alive[3] == 1u) && close(sum,4)) || ((rules.stay_alive[4] == 1u) && close(sum,5)) || ((rules.stay_alive[5] == 1u) && close(sum,6))  || ((rules.stay_alive[6] == 1u) && close(sum,7)) || ((rules.stay_alive[7] == 1u) && close(sum,8))   );


    
    
    if (close(val,0) && born) {
        return 1.0;
    }
    if (close(val,1) && stay_alive) {
        return 1.0;
    }
    return 0.0;
 }

[[group(0), binding(0)]] var input_texture : texture_2d<f32>;
[[group(0), binding(1)]] var output_texture : texture_storage_2d<rgba8unorm, write>;
[[group(0), binding(2)]] var<uniform> rules : Rules;

[[stage(compute), workgroup_size(16, 16)]]
fn totalistic_main(
  [[builtin(global_invocation_id)]] global_id : vec3<u32>,
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