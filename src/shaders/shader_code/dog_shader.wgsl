struct DogBinding {
    resolution: vec2f,
}

@group(0) @binding(0) var u_texture: texture_2d<f32>;
@group(0) @binding(1) var u_sampler: sampler;
@group(0) @binding(2) var<uniform> u_res: DogBinding;

struct VertexShaderOutput {
    @builtin(position) position: vec4f,
    @location(0) frag_uv: vec2f,
    @location(1) frag_coord: vec2f,
};

@vertex fn v_main(
    @builtin(vertex_index) v_index : u32
) -> VertexShaderOutput {
    let pos = array(

        vec2f( -1.0,  -1.0),  // bottom right
        vec2f( -1.0,  1.0),  // top right
        vec2f( 1.0,  -1.0),  // bottom right

        vec2f( 1.0,  -1.0),  // bottom right
        vec2f( -1.0,  1.0),  // top left
        vec2f( 1.0,  1.0),  // top right
    );

    var v_output: VertexShaderOutput;
    let xy = pos[v_index];
    v_output.position = vec4f(xy.x, -xy.y, 0.0, 1.0);

    v_output.frag_uv = (xy + 1) / 2; // convert clip-space (-1 - 1) to UV (0 - 1)
    v_output.frag_coord = v_output.frag_uv * u_res.resolution;

    return v_output;
}

const MATRIX_SIZE : i32 = 11;
const KERNEL_SIZE : i32 = (MATRIX_SIZE - 1)/2;

fn desaturate(color: vec3<f32>) -> vec4<f32> {
    var lum = vec3(0.299, 0.587, 0.114);
    var gray = vec3(dot(lum, color));
    return vec4(vec3(gray), 1);
}

// normalized probability density function
fn normPdf(x: f32, sigma: f32) -> f32 {
    return 0.39894 * exp(-0.5 * x * x / (sigma*sigma)) / sigma;
}

fn blur(frag_coord: vec2<f32>, sigma: f32) -> vec3<f32> {
    var kSize = KERNEL_SIZE;
    var kernel = array<f32, MATRIX_SIZE>();

    // calculate kernel density
    for(var i = 0; i <= kSize; i++) {
        kernel[kSize + i] = normPdf(f32(i), sigma);
        kernel[kSize - i] = normPdf(f32(i), sigma);
    }

    // calculate sum of kernel
    var sum = 0.0;
    for(var i = 0; i < MATRIX_SIZE; i++) {
        sum += kernel[i];
    }

    // apply gaussian blur
    var blur = vec3(0.0);
    for(var i = -kSize; i <= kSize; i++) {
        for(var j = -kSize; j <= kSize; j++) {
            var texel = textureSample(u_texture, u_sampler, (frag_coord + vec2(f32(i), f32(j))) / u_res.resolution);

            blur += kernel[kSize + j] * kernel[kSize + i] * texel.rgb;
        }
    }

    return blur / (sum*sum);
}

@fragment fn f_main(f_input: VertexShaderOutput) -> @location(0) vec4f {
    var uv = f_input.frag_uv;
    var frag_coord = f_input.frag_coord;

    //
    // 2. DoG
    var sigmaBase = 3.0;
    var sigmaSubtract = sigmaBase + 10.0; // trust bro, trust

    var strongBlur = blur(frag_coord, sigmaSubtract);
    var weakBlur = blur(frag_coord, sigmaBase);

    // desaturate blurs
    var desaturatedSBlur = desaturate(strongBlur);
    var desaturatedWBlur = desaturate(weakBlur);

    // subtract the blurs
    var DoG = desaturatedWBlur - desaturatedSBlur;
    
    // quantize
    if(DoG.r < 0.02) {
        DoG = vec4(0.0, 0.0, 0.0, 1.0);
    }
    else {
        DoG = vec4(1.0,1.0,1.0,1.0);
    }
    
    return DoG;
}