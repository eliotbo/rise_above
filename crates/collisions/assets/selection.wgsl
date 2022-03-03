type float4 = vec4<f32>;
type float2 = vec2<f32>;

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};



var<private>  solid: f32 = 0.001;  
var<private>  smooth_dist2: f32 = 0.003;
var<private>  point_radius: f32 = 0.03;
var<private>  out_of_bounds: f32 = 100000.0;
var<private>  bluish : float4 = float4 (0.13, 0.28, 0.86, 1.0);
var<private>  num_segments: i32 = 256;




struct GraphEditShader {
    pos: vec2<f32>;
    color: vec4<f32>;
    radius: f32;
};

[[group(1), binding(0)]]
var<uniform> mate: GraphEditShader;

fn sdCircle(pos: vec2<f32>, r: f32) -> f32 {
    return length(pos)-r;
}


[[stage(fragment)]]
fn fragment(in: VertexOutput) -> [[location(0)]] vec4<f32> {

    // // ///////////////////// coordinates /////////////////////////////////
    // let x_max = mate.bounds.up.x;
    // let y_max = mate.bounds.up.y;

    // let x_min = mate.bounds.lo.x;
    // let y_min = mate.bounds.lo.y;

    // let x_range = x_max - x_min;
    // let y_range = y_max - y_min;
    // // ///////////////////// coordinates /////////////////////////////////


    let black = vec4<f32> (0.0, 0.0, 0.0, 1.0);

    let pos = float2(mate.pos.x, mate.pos.y);
    var d2 = sdCircle(in.uv - pos, mate.radius * 2.0);

    var out_col = black;
    out_col.a = 0.0;

    let s2 = smoothStep(0.0 ,  2.0, abs(d2) - 2.0   );
    out_col = mix(out_col, black, 1.0 - s2);


    return out_col;
}
