fn get_pixel_wrap(pos: vec2<i32>, dims: vec2<i32>, tex: texture_2d<f32>) -> vec4<f32> {
    let x = (pos.x + dims.x) %dims.x;
    let y = (pos.y +  dims.y) %  dims.y;
    let col: vec4<f32> = textureLoad(tex, vec2<i32>(x,y), 0);
    return col;
 }

fn close(a: f32, b: i32) -> bool {
    return abs(a - f32(b)) < 0.2;
}