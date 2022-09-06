
fn get_pixel_wrap(pos: vec2<i32>, dims: vec2<i32>, tex: texture_2d<f32>) -> vec4<f32> {
    let x = (pos.x + dims.x) %dims.x;
    let y = (pos.y +  dims.y) %  dims.y;
    let col: vec4<f32> = textureLoad(tex, vec2<i32>(x,y), 0);
    return col;
 }

fn close(a: f32, b: i32) -> bool {
    return abs(a - f32(b)) < 0.1;
}

 fn compute_cell(val: f32, sum: f32) -> f32 {
    if (close(val,0) && close(sum,3)) {
        return 1.0;
    }
     if (close(val,1) && (close(sum,2) || close(sum,3))) {
        return 1.0;
    }
    return 0.0;
 }

[[group(0), binding(0)]] var input_texture : texture_2d<f32>;
[[group(0), binding(1)]] var output_texture : texture_storage_2d<rgba8unorm, write>;


[[stage(compute), workgroup_size(16, 16)]]
fn life_main(
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
    let cell =  compute_cell(me, sum);
    let new_color = vec4<f32>(cell,cell,cell,1.0);

    textureStore(output_texture, coords.xy, new_color);
}