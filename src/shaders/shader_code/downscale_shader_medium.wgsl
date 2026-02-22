struct DogBinding {
    resolution: vec2f,
}

struct WorkgroupSize {
    x: i32,
    y: i32,
    z: i32,
}

@group(0) @binding(0) var<storage, read_write> storage_buffer: array<u32>;
@group(0) @binding(1) var u_texture: texture_2d<f32>;
@group(0) @binding(2) var u_sobel: texture_2d<f32>;
@group(0) @binding(3) var<uniform> u_res: DogBinding;
@group(0) @binding(4) var<uniform> u_quantize: f32;

@group(0) @binding(5) var<uniform> u_brightness: f32;
@group(0) @binding(6) var<uniform> u_contrast: f32;
@group(0) @binding(7) var<uniform> u_draw_edges: i32;
@group(0) @binding(8) var<uniform> u_edge_threshold: f32;

fn vec4Equals(a: vec4<f32>, b: vec4<f32>) -> bool {
    var boolVec = a == b;
    if(boolVec.x == false || boolVec.y == false || boolVec.z == false || boolVec.w == false) {
        return false;
    }
    return true;
}

struct PixelData {
    luma: f32,
    edge_data: f32,
}

fn getLuma(tex: vec4<f32>) -> f32 {
    var tex_luma = tex.r * 0.2126 + tex.g * 0.7152 + tex.b * 0.0722;
    return tex_luma;
}

fn quantize(luma: f32) -> f32 {
    return round(luma * (u_quantize - 1.0));
}

// TODO: this should be a pre-processing stage before DoG
fn contrast(input: vec4<f32>) -> vec4<f32> {
    var tex = input;
    tex.r = mix(0.5, tex.r + u_brightness - 1.0, u_contrast);
    tex.g = mix(0.5, tex.g + u_brightness - 1.0, u_contrast);
    tex.b = mix(0.5, tex.b + u_brightness - 1.0, u_contrast);
    tex.r = clamp(0.0, 1.0, tex.r);
    tex.g = clamp(0.0, 1.0, tex.g);
    tex.b = clamp(0.0, 1.0, tex.b);
    return tex;
}

// A designated file for each wg_size is gross, but I do not know of a better way
const wg_x = 6;
const wg_y = 13;
const x = f32(wg_x);
const y = f32(wg_y);
const TILE_DIM = x * y;
var<workgroup> tile: array<array<PixelData, wg_y>, wg_x>;

@compute @workgroup_size(wg_x, wg_y, 1)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) wg_id: vec3<u32>
) {
    var coords: vec2<i32> = vec2(i32(global_id.x), i32(global_id.y));
    var sobel = textureLoad(u_sobel, coords, 0);
    var tex = textureLoad(u_texture, coords, 0);
    if(tex.a < 0.01) {
        tex.r *= tex.a;
        tex.g *= tex.a;
        tex.b *= tex.a;
    }
    tex = contrast(tex);

    var edge_data = 0.0;
    var luma = 0.0;
    if(!vec4Equals(vec4(0.0,0.0,0.0,1.0), sobel)) {
        if(sobel.b == 1.0) {
            edge_data = 1.0; // blue = __ = 1
        }
        else if(sobel.r == 1.0 && sobel.g == 0.0) {
            edge_data = 2.0; // red = | = 2
        }
        else if(sobel.r == 0.0 && sobel.g == 1.0) {
            edge_data = 3.0; // green = / = 3
        }
        else {
            edge_data = 4.0; // yellow = \ = 4
        }
    }

    luma = getLuma(tex);

    tile[local_id.x][local_id.y].luma = luma;
    tile[local_id.x][local_id.y].edge_data = edge_data;

    workgroupBarrier();

    var histogram = vec4(0.0); // rgby
    for(var i = 0; i < wg_x; i++) {
        for(var j = 0; j < wg_y; j++) {
            var iter_edge = tile[i][j].edge_data;

            if(iter_edge == 2) {histogram += vec4(1.0,0.0,0.0,0.0);}
            else if(iter_edge == 3) {histogram += vec4(0.0,1.0,0.0,0.0);}
            else if(iter_edge == 1) {histogram += vec4(0.0,0.0,1.0,0.0);}
            else if(iter_edge == 4) {histogram += vec4(0.0,0.0,0.0,1.0);}
        }
    }

    // if there are NO detected sobel gradients in a tile, then skip this step
    var res = 0.0;
    var edge_threshold = u_edge_threshold;
    if(u_draw_edges == 1) {
        edge_threshold *= TILE_DIM;
    }
    else {
        edge_threshold = 999.0; // arbitrarily high to prevent edge draws
    }
    if(!vec4Equals(vec4(0.0), histogram)) {
        var max = 0.0;
        if(histogram.r > max) {
            max = histogram.r;
            res = 1000.0;
        }
        if(histogram.g > max) {
            max = histogram.g;
            res = 2000.0;
        }
        if(histogram.b > max) {
            max = histogram.b;
            res = 3000.0;
        }
        if(histogram.a > max) {
            max = histogram.a;
            res = 4000.0;
        }
        
        if(max < edge_threshold) {
            res = 0.0;
        }
    }

    // if no edges drawn, then calculate average brightness
    if(res == 0.0) {

        var sum = 0.0;
        for(var i = 0; i < wg_x; i++) {
            for(var j = 0; j < wg_y; j++) {
                sum += tile[i][j].luma;
            }
        }
        res = quantize(sum/TILE_DIM);
        
    }
 
    var coords_f = vec2(f32(coords.x), f32(coords.y));

    var f_id = vec2(f32(wg_id.x), f32(wg_id.y));

    var index = f_id.x + (f_id.y * ceil(u_res.resolution.x/x));

    storage_buffer[i32(index)] = u32(res);
}