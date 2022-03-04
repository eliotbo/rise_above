// Import the standard 2d mesh uniforms and set their bind groups
#import bevy_sprite::mesh2d_view_bind_group
[[group(0), binding(0)]]
var<uniform> view: View;


#import bevy_sprite::mesh2d_struct

[[group(1), binding(0)]]
var<uniform> mesh: Mesh2d;

type float4 = vec4<f32>;
type float2 = vec2<f32>;

// The structure of the vertex buffer is as specified in `specialize()`
struct Vertex {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
    // [[location(3)]] tangent: vec4<f32>;

    // instanced
    [[location(3)]] i_pos_group_scale: vec4<u32>;
    [[location(4)]] i_color: vec4<u32>;

    [[location(5)]] x1: vec4<u32>;

    [[location(6)]] x2: vec4<u32>;
    [[location(7)]] x3: vec4<u32>;
    [[location(8)]] x4: vec4<u32>;
    [[location(9)]] x5: vec4<u32>;

    [[location(10)]] x6: vec4<u32>;
    [[location(11)]] x7: vec4<u32>;
    [[location(12)]] x8: vec4<u32>;
    [[location(13)]] x9: vec4<u32>;

    [[location(14)]] x10: vec4<u32>;
    [[location(15)]] x11: vec4<u32>;
    // [[location(16)]] x12: vec4<u32>;
    // [[location(17)]] x13: vec4<u32>;

    // [[location(18)]] x14: vec4<u32>;

    // [[location(19)]] x15: vec4<u32>;
    // [[location(20)]] x16: vec4<u32>;
    // [[location(21)]] x17: vec4<u32>;

    // [[location(22)]] x18: vec4<u32>;
    // [[location(23)]] x19: vec4<u32>;
    // [[location(24)]] x20: vec4<u32>;
    // [[location(25)]] x21: vec4<u32>;

    // [[location(26)]] x22: vec4<u32>;
    
    // [[location(27)]] x23: vec4<f32>;
    // [[location(28)]] x24: vec4<f32>;
    // [[location(29)]] x25: vec4<f32>;

    // [[location(5)]] i_color2: vec4<f32>;
    // [[location(5)]] i_uv_offset: vec2<f32>;
};

struct VertexOutput {
    // The vertex shader must set the on-screen position of the vertex
    [[builtin(position)]] clip_position: vec4<f32>;

    [[location(0)]] uv: vec2<f32>;

    [[location(1)]] pos_scale: vec4<u32>;
    [[location(2)]] color: vec4<u32>;

    //     [[location(3)]] x12: vec4<u32>;
    // [[location(4)]] x13: vec4<u32>;

    // [[location(5)]] x14: vec4<u32>;

    [[location(5)]] x1: vec4<u32>;
    [[location(6)]] x2: vec4<u32>;
    [[location(7)]] x3: vec4<u32>;
    [[location(8)]] x4: vec4<u32>;
    [[location(9)]] x5: vec4<u32>;

    [[location(10)]] x6: vec4<u32>;
    [[location(11)]] x7: vec4<u32>;
    [[location(12)]] x8: vec4<u32>;
    [[location(13)]] x9: vec4<u32>;

    [[location(14)]] x10: vec4<u32>;
    [[location(15)]] x11: vec4<u32>;

    // [[location(19)]] x15: vec4<u32>;
    // [[location(20)]] x16: vec4<u32>;
    // [[location(21)]] x17: vec4<u32>;

    // [[location(22)]] x18: vec4<u32>;
    // [[location(23)]] x19: vec4<u32>;
    // [[location(24)]] x20: vec4<u32>;
    // [[location(25)]] x21: vec4<u32>;

    // [[location(26)]] x22: vec4<u32>;

    // [[location(3)]] poss: array<vec2<u32>, 64>;
};

struct FragmentInput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
    [[location(1)]] pos_scale: vec4<u32>;
    [[location(2)]] color: vec4<u32>;

    // [[location(3)]] x12: vec4<u32>;
    // [[location(4)]] x13: vec4<u32>;

    // [[location(5)]] x14: vec4<u32>;


    [[location(5)]] x1: vec4<u32>;
    [[location(6)]] x2: vec4<u32>;
    [[location(7)]] x3: vec4<u32>;
    [[location(8)]] x4: vec4<u32>;
    [[location(9)]] x5: vec4<u32>;

    [[location(10)]] x6: vec4<u32>;
    [[location(11)]] x7: vec4<u32>;
    [[location(12)]] x8: vec4<u32>;
    [[location(13)]] x9: vec4<u32>;

    [[location(14)]] x10: vec4<u32>;
    [[location(15)]] x11: vec4<u32>;

    // [[location(19)]] x15: vec4<u32>;
    // [[location(20)]] x16: vec4<u32>;
    // [[location(21)]] x17: vec4<u32>;

    // [[location(22)]] x18: vec4<u32>;
    // [[location(23)]] x19: vec4<u32>;
    // [[location(24)]] x20: vec4<u32>;
    // [[location(25)]] x21: vec4<u32>;

    // [[location(26)]] x22: vec4<u32>;
    // [[location(3)]] pos: vec2<f32>;
};



[[stage(vertex)]]
fn vertex(vertex: Vertex) -> VertexOutput {

    let maxu = 4294967295.0;
    let scale = f32(vertex.i_pos_group_scale.w) / 4294967295.0;
    let pos = vec3<f32>(vertex.i_pos_group_scale.xyz) / maxu;

    let position = vertex.position * scale + pos ;
    let world_position = mesh.model * vec4<f32>(position, 1.0);

    var out: VertexOutput;

    out.clip_position = view.view_proj * world_position;
    let v = vertex.i_color;
    // out.color = v[0];
    out.color = vertex.i_color;

    // out.color = float4(v[0], v[1], v[2], v[3]);
    
    // out.color = v;
    out.uv = vertex.uv;
    out.pos_scale = vertex.i_pos_group_scale;
    // out.arr = array<f32>(1.0);
    // out.arr = 1.0;

    out.x1 =  vertex.x1;
    out.x2 =  vertex.x2;
    out.x3 =  vertex.x3;
    out.x4 =  vertex.x4;
    out.x5 =  vertex.x5;
    out.x6 =  vertex.x6;
    out.x7 =  vertex.x7;
    out.x8 =  vertex.x8;
    out.x9 =  vertex.x9;
    out.x10 = vertex.x10;
    out.x11 = vertex.x11;

    // out.x12 = vertex.x12;
    // out.x13 = vertex.x13;
    // out.x14 = vertex.x14;

    // out.x15 = vertex.x15;
    // out.x16 = vertex.x16;
    // out.x17 = vertex.x17;
    // out.x18 = vertex.x18;
    // out.x19 = vertex.x19;
    // out.x20 = vertex.x20;
    // out.x21 = vertex.x21;
    // out.x22 = vertex.x22;



    // out.pos = vertex.i_pos_group_scale.xy;

    return out;
}

fn fromLinear(linearRGB: float4) -> float4
{
    let cutoff: vec4<f32> = vec4<f32>(linearRGB < float4(0.0031308));
    let higher: vec4<f32> = float4(1.055)*pow(linearRGB, float4(1.0/2.4)) - float4(0.055);
    let lower: vec4<f32> = linearRGB * float4(12.92);

    return mix(higher, lower, cutoff);
}

// Converts a color from sRGB gamma to linear light gamma
fn toLinear(sRGB: float4) -> float4
{
    let cutoff = vec4<f32>(sRGB < float4(0.04045));
    let higher = pow((sRGB + float4(0.055))/float4(1.055), float4(2.4));
    let lower = sRGB/float4(12.92);

    return mix(higher, lower, cutoff);
}




fn cla(mi: f32, ma: f32, x: f32) -> f32 {
  if (x<mi) {
    return mi;
  }
  if (x>ma) {
    return ma;
  }
  return x;
}

fn sdSegment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
  let pa = p - a;
  let ba = b - a;
  let h = clamp(dot(pa, ba) / dot(ba, ba), 0., 1.);
  return length(pa - ba * h);
}



fn opSmoothUnion( d1: f32, d2: f32, k: f32 ) -> f32 {
    let h = clamp( 0.5 + 0.5 * (d2 - d1) / k , 0.0, 1.0 );
    let p = mix( d2, d1, h ) - k * h * (1.0 - h ); 
    return p;
}

fn sdCircle(p: vec2<f32>, c: vec2<f32>, r: f32) -> f32 {
  let d = length(p - c);
  return d - r;
}

fn sdCircle2(p: vec2<f32>, c: vec2<f32>, r: f32) -> f32 {
  let d = length(p - c);
  return d - r;
}



fn sdRoundedBox(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>) -> f32 {
  var x = r.x;
  var y = r.y;
  x = select(r.z, r.x, p.x > 0.);
  y = select(r.w, r.y, p.x > 0.);
  x  = select(y, x, p.y > 0.);
  let q = abs(p) - b + x;
  return min(max(q.x, q.y), 0.) + length(max(q, vec2<f32>(0.))) - x;
}

fn sdBox(p: vec2<f32>, b: vec2<f32>) -> f32 {
  let d = (abs(p) - b) ;
  return length(max(d, vec2<f32>(0.))) + min(max(d.x, d.y), 0.);
}

fn decode(input: u32, place: u32, precision: u32) -> f32 {
    let value_u32 = input >> (place - precision);

    var mask: u32 = 4294967295u;
    if (precision < 32u) {
        mask = (1u << (precision)) - 1u;
    }

    // println!("mask: {:#0b}", value_u32);
    let masked_value_u32 = value_u32 & mask;
    let max_val = 1u << (precision - 1u);
    let value_f32 = f32(masked_value_u32) / f32(max_val) ;

    return value_f32;
}

struct Data {
  pos: vec2<f32>;
  max_size: f32;
  frequency: f32;
  noise: f32;
  min_size: f32;
  morph: f32;
  hole_size: f32;
};

fn decode_all(input: vec2<u32>) -> Data {
    let pos = vec2<f32>(
        decode(input.x, 32u, 16u) - 0.5,
        decode(input.y, 32u, 16u) - 0.5,
    );

    let max_size = decode(input.x, 16u, 8u);
    let frequency = decode(input.x, 8u, 4u);
    let noise = decode(input.x, 4u, 4u);

    let min_size = decode(input.y, 16u, 8u);
    let morph = decode(input.y, 8u, 4u);
    let hole_size = decode(input.y, 4u, 4u);

    var data: Data;
    data.pos = pos;
    data.max_size = max_size;
    data.frequency = frequency;
    data.noise = noise;
    data.min_size = min_size;
    data.morph = morph;
    data.hole_size = hole_size;

    return data;
    
}

// fn hash(value: u32) -> u32 {
//     var state = value;
//     state = state ^ 2747636419u;
//     state = state * 2654435769u;
//     state = state ^ state >> 16u;
//     state = state * 2654435769u;
//     state = state ^ state >> 16u;
//     state = state * 2654435769u;
//     return state;
// }
// fn randomFloat(value: u32) -> f32 {
//     return f32(hash(value)) / 4294967295.0;
// }

fn permute4(x: vec4<f32>) -> vec4<f32> { return ((x * 34. + 1.) * x) % vec4<f32>(289.); }
fn fade2(t: vec2<f32>) -> vec2<f32> { return t * t * t * (t * (t * 6. - 15.) + 10.); }

fn perlinNoise2(P: vec2<f32>) -> f32 {
  var Pi: vec4<f32> = floor(P.xyxy) + vec4<f32>(0., 0., 1., 1.);
  let Pf = fract(P.xyxy) - vec4<f32>(0., 0., 1., 1.);
  Pi = Pi % vec4<f32>(289.); // To avoid truncation effects in permutation
  let ix = Pi.xzxz;
  let iy = Pi.yyww;
  let fx = Pf.xzxz;
  let fy = Pf.yyww;
  let i = permute4(permute4(ix) + iy);
  var gx: vec4<f32> = 2. * fract(i * 0.0243902439) - 1.; // 1/41 = 0.024...
  let gy = abs(gx) - 0.5;
  let tx = floor(gx + 0.5);
  gx = gx - tx;
  var g00: vec2<f32> = vec2<f32>(gx.x, gy.x);
  var g10: vec2<f32> = vec2<f32>(gx.y, gy.y);
  var g01: vec2<f32> = vec2<f32>(gx.z, gy.z);
  var g11: vec2<f32> = vec2<f32>(gx.w, gy.w);
  let norm = 1.79284291400159 - 0.85373472095314 *
      vec4<f32>(dot(g00, g00), dot(g01, g01), dot(g10, g10), dot(g11, g11));
  g00 = g00 * norm.x;
  g01 = g01 * norm.y;
  g10 = g10 * norm.z;
  g11 = g11 * norm.w;
  let n00 = dot(g00, vec2<f32>(fx.x, fy.x));
  let n10 = dot(g10, vec2<f32>(fx.y, fy.y));
  let n01 = dot(g01, vec2<f32>(fx.z, fy.z));
  let n11 = dot(g11, vec2<f32>(fx.w, fy.w));
  let fade_xy = fade2(Pf.xy);
  let n_x = mix(vec2<f32>(n00, n01), vec2<f32>(n10, n11), vec2<f32>(fade_xy.x));
  let n_xy = mix(n_x.x, n_x.y, fade_xy.y);
  return 2.3 * n_xy;
}


struct MarkerUniform {
    marker_size: f32;
    hole_size: f32;
    zoom: f32;
    time: f32;
    quad_size: f32;
    contour: f32;
    inner_canvas_size_in_pixels: float2;
    canvas_position_in_pixels: float2;
    color: float4;
    marker_point_color: float4;
};

[[group(2), binding(0)]]
var<uniform> uni: MarkerUniform;


[[stage(fragment)]]
fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {
 


    
    // var out_col = in.color;

    var uv = in.uv - float2(0.5,0.5) ;
    uv.y = - uv.y;

    // // var uv_in_pixels = float2(-uv.x, uv.y) * uni.quad_size - in.pos_scale.xy;

    // let marker_size = uni.marker_size;

    // let point_type = i32(uni.point_type);
    // // let point_type = 6;

    // // // change the aliasing as a function of the zoom
    // // var circ_zoom = zoom;

    // // if (zoom  >.0) {
    // //   circ_zoom =  pow(zoom, 0.05);
    // // }

    // // if (zoom < 1.0) {
    // //   circ_zoom =  sqrt(sqrt(zoom));
    // // }

    // let black = float4(0.0, 0.0, 0.0, 1.0);
    // let red = float4(0.7, 0.2, 0.2, 1.0);



    var arr = array<vec2<u32>, 22>(
      in.x1.xy,
      in.x1.zw,
      in.x2.xy,
      in.x2.zw,
      in.x3.xy,
      in.x3.zw,
      in.x4.xy,
      in.x4.zw,
      in.x5.xy,
      in.x5.zw,
      in.x6.xy,
      in.x6.zw,
      in.x7.xy,
      in.x7.zw,
      in.x8.xy,
      in.x8.zw,
      in.x9.xy,
      in.x9.zw,
      in.x10.xy,
      in.x10.zw,
      in.x11.xy,
      in.x11.zw,

      // in.x12.xy,
      // in.x12.zw,
      // in.x13.xy,
      // in.x13.zw,
      // in.x14.xy,
      // in.x14.zw,

      // in.x15.xy,
      // in.x15.zw,
      // in.x16.xy,
      // in.x16.zw,
      // in.x17.xy,
      // in.x17.zw,
      // in.x18.xy,
      // in.x18.zw,
      // in.x19.xy,
      // in.x19.zw,
      // in.x20.xy,
      // in.x20.zw,
      // in.x21.xy,
      // in.x21.zw,
      // in.x22.xy,
      // in.x22.zw,
    );

    let fushia = toLinear(float4(144.0, 17.0, 74.0, 255.0) / 255.0);
    let blue1 = toLinear(float4(11.0, 71.0, 191.0, 255.0) / 255.0);
    let blue2 = toLinear(float4(8.0, 52.0, 140.0, 255.0) / 255.0);
    let blue3 = toLinear(float4(7.0, 26.0, 64.0, 255.0) / 255.0 * 1.8);
    let green = toLinear(float4(5.0, 242.0, 175.0, 255.0) / 255.0);
    let joint_color = toLinear(float4(0.0, 148.0, 3.0, 255.0) / 255.0);

    let red = float4(0.7, 0.2, 0.2, 1.0);
    let ll = 1.4;
    let light = float4(blue1.x * ll, blue1.y * ll, blue1.z * ll, 1.0);

    let bl = 0.3;
    let black = float4(bl, bl, bl, 1.0);
    let dark = float4(blue3.x*bl, blue3.y*bl, blue3.z*bl, 1.0);

    let bgc = float4(0.92, 0.93, 0.95, 1.0) * 0.75;
    let background = blue2; // toLinear(uni.color);
    // let background = fushia;
    let main_color = fushia;
    let contour_color = blue3;

    // let background = float4(bgc.x, bgc.y, bgc.z, 1.0) ;
  
    

    let width = 0.0141 / uni.marker_size * 0.05;
    let zoom = uni.zoom;

    var w = width * zoom  ;
    var solid = width * zoom  ;
    

    var out_col = black;

    let biggest_size = 0.05;

    var circle_size = 0.02 ;

    var d_shadow = 10000.0;
    var d_light = 10000.0;
    var d_contour = 10000.0;
    var d_main = 10000.0;
    var d_center = 10000.0;

    var joints: array<u32, 5> = array<u32, 5>(555u, 555u, 555u,5550u,5550u);
    var joint_index = 0;
    
    for (var i = 0; i < 22; i=i+1) {

        var data = decode_all(arr[i]);

        if (data.noise > 0.99) {
          
          joints[joint_index] = u32(i);
          joint_index = joint_index + 1;
          continue;
        }

        if ((data.pos.x > 0.49) || (data.pos.x < -0.49)) {
          continue;
        }
        
        // morph
        // let sm = 0.5 * data.morph * 1.0;
        let sm = 1.0 ;//pow(2.0 * data.morph , 3.0);
        let sm = data.morph * 4.0; // pow(2.0 * data.morph , 3.0);
        let sm_shadow = 0.05 * sm;
        let sm_main = 0.05 * sm;
        let sm_center = 0.01 * sm;
        let sm_contour = 0.05 * sm;

        let neumorph = 0.5  ;

        // var freq = data.frequency;
        let freq = pow(2.0 * data.frequency , 3.0);
        let max_size =  data.max_size * biggest_size;
        let min_size = data.min_size * biggest_size;
        let noise = data.noise * 2.0;

        


        var hole_size = data.hole_size * 2.0;
        // hole_size = 0.6;

        // radius
        // let circle_size = cla(0.01, 0.45,  0.25 * uni.marker_size); 
        // let r = circle_size * (1.0 + 0.15 *sin(uni.time * 3.1415* freq));

        let r = (min_size + max_size) / 2.0  + (max_size - min_size ) 
            * sin(uni.time * 3.1415* freq);

        // let circle_size = r * (1.0 - hole_size);
        circle_size = r ;

        let r_shadow =  r * 0.65;
        let r_light =  r * 0.65;
        let r_main  = r * 0.65;
        let r_contour = r * 0.95;
        let r_center = r * 0.25 * hole_size;


        let shadow_offset= float2(0.01,-0.01) * 0.7 * 1.0 ;
    
        let light_offset=  -shadow_offset;

        let noise = perlinNoise2((uv - data.pos + f32(i) + noise) * 10.0) * 0.02 * noise;
        let noisy_uv = uv + noise;

        d_shadow = opSmoothUnion(d_shadow, sdCircle(noisy_uv + shadow_offset, data.pos, r_shadow), sm_shadow);
        d_light = opSmoothUnion(d_light, sdCircle(noisy_uv + light_offset, data.pos, r_light), sm_shadow);
        d_contour = opSmoothUnion(d_contour, sdCircle(noisy_uv, data.pos, r_contour), sm_contour);
        d_main  = opSmoothUnion(d_main  , sdCircle(noisy_uv  , data.pos, r_main  ) , sm_main);
        d_center = opSmoothUnion(d_center, sdCircle(noisy_uv, data.pos, r_center), sm_center);
              
    }
    
  let main_thi = 0.01;

    let s_shadow = smoothStep(0.0 ,  w * neumorph, d_shadow - circle_size / 3.0    );
    // let s_light = smoothStep(0.0 ,  w * neumorph, d_light - circle_size / 3.0   );
    let s_light = smoothStep(0.0 ,  w * neumorph, d_light - circle_size / 3.0   );
    let s_contour = smoothStep(0.0 ,  w , abs(d_contour) - circle_size / 3.0   );
    let s_center = smoothStep(0.0 ,  w , d_center + 0.006 );
    // let s_main = smoothStep(0.0+ main_thi ,  w + main_thi , abs(d_main ) - circle_size / 3.0    );
    let s_main = smoothStep(0.0 ,  w , abs(d_main ) - 0.01    );

    out_col = mix(background , dark, 1.0 - s_shadow); 
    out_col = mix(out_col, light, 1.0 - s_light); 
    // out_col = mix(out_col, contour_color, 1.0 - s_contour); 
    
    out_col = mix(out_col, main_color, 1.0 - s_main); 
    out_col = mix(out_col, green, 1.0 - s_center); 


//////////////////////////////// joints ////////////////////////////////////////////
    var d_shadow_j = 10000.0;
    var d_light_j = 10000.0;
    var d_contour_j = 10000.0;
    var d_main_j = 10000.0;
    var d_center_j = 10000.0;

    // let circle_size = r ;
    for (var j = 0; j < 5; j=j+1) {


        let joint_index = joints[j];
        if (joint_index == 555u) {
          continue;
        }
        var data = decode_all(arr[joint_index]);



        if ((data.pos.x > 0.49) || (data.pos.x < -0.49)) {
          continue;
        }

        // if (true) {
        //   continue;
        // }
        

        // if (data.noise < 0.99) {
          
        //   continue;
        // }
        
        // morph
        // let sm = 0.5 * data.morph * 1.0;
        let sm = 1.0 ;//pow(2.0 * data.morph , 3.0);
        let sm = data.morph * 4.0; // pow(2.0 * data.morph , 3.0);
        let sm_shadow = 0.05 * sm;
        let sm_main = 0.05 * sm;
        let sm_center = 0.01 * sm;
        let sm_contour = 0.05 * sm;

        let neumorph = 2.0 *  data.morph ;

        // var freq = data.frequency;
        let freq = pow(2.0 * data.frequency , 3.0);
        let max_size =  data.max_size * biggest_size;
        let min_size = data.min_size * biggest_size;
        let noise = data.noise * 2.0;

        


        var hole_size = data.hole_size * 2.0;
        hole_size = 0.6;

        // radius
        // let circle_size = cla(0.01, 0.45,  0.25 * uni.marker_size); 
        // let r = circle_size * (1.0 + 0.15 *sin(uni.time * 3.1415* freq));

        let r = (min_size + max_size) / 2.0  + (max_size - min_size ) 
            * sin(uni.time * 3.1415* freq);

        // let circle_size = r * (1.0 - hole_size);
        

        let r_shadow =  r;
        let r_light =  r;
        let r_main  = r * 0.65;
        let r_center = r * 0.85 * hole_size;


        let shadow_offset= float2(0.01,-0.01) * 0.7 * 1.0 ;
    
        let light_offset=  -shadow_offset;


        // d_shadow_j = opSmoothUnion(d_shadow_j, sdCircle(uv + shadow_offset, data.pos, r_shadow), sm_shadow);
        // d_light_j = opSmoothUnion(d_light_j, sdCircle(uv + light_offset, data.pos, r_light), sm_shadow);
        // d_contour_j = opSmoothUnion(d_contour_j, sdCircle(uv, data.pos, r * 0.75), sm_contour);
        d_main_j  = opSmoothUnion(d_main_j  , sdCircle(uv  , data.pos, r_main ) , sm_main);
        // d_center_j = opSmoothUnion(d_center_j, sdCircle(uv, data.pos, r_center), sm_center);
              
    }
    
    // let s_shadow_j = smoothStep(0.0 ,  w * neumorph, d_shadow_j - circle_size / 3.0    );
    // let s_light_j = smoothStep(0.0 ,  w * neumorph, d_light_j - circle_size / 3.0   );
    // let s_contour_j = smoothStep(0.0 ,  w , abs(d_contour_j) - circle_size / 3.0    );
    // let s_center_j = smoothStep(0.0 ,  w , d_center_j + 0.006 );

    let s_main_j = smoothStep(0.0 ,  w , abs(d_main_j ) - circle_size / 3.0    );
    
    // let s_main_j = smoothStep(0.0 ,  w , abs(d_main ) - 0.01    );

    // out_col = mix(out_col , dark, 1.0 - s_shadow); 
    // out_col = mix(out_col, light, 1.0 - s_light); 
    // out_col = mix(out_col, contour_color, 1.0 - s_contour_j); 
    // out_col = mix(out_col, green, 1.0 - s_center); 

    out_col = mix(out_col, joint_color, 1.0 - s_main_j); 
    /////////////////////////////////////////////////// joints ////////////////////////////////////////////


    return out_col + 0.0;

  // return float4(0.0, 1.0, 0.0, 1.0);



}

