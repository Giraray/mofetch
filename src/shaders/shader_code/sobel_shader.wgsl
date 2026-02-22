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

const PI : f32 = 3.141592653589793238;
const RADIAL_DIV : f32 = 1.0/16.0; // radial divider

// honestly not sure why we're getting the luminance of a binary texel
fn getFragLuma(offsetUV: vec2<f32>) -> f32 {
    var targetColor = textureSample(u_texture, u_sampler, offsetUV);
    var fragLuma = targetColor.r * 0.2126 + targetColor.g * 0.7152 + targetColor.b * 0.0722;
    return fragLuma;
}

@fragment fn f_main(f_input: VertexShaderOutput) -> @location(0) vec4f {
    var uv = f_input.frag_uv;
    //
    // 3. sobel gradients
    var stepValue = 1.0;
    var stepx = stepValue / u_res.resolution.x;
    var stepy = stepValue / u_res.resolution.y;

    var kernel1 = 1.0;
    var kernel2 = 2.0;

    var horizontalSobelMatrix = array<f32, 9>(
        -kernel1, 0.0, kernel1,
        -kernel2, 0.0, kernel2,
        -kernel1, 0.0, kernel1
    );

    var verticalSobelMatrix = array<f32, 9>(
        kernel1, kernel2, kernel1,
        0.0, 0.0, 0.0,
        -kernel1, -kernel2, -kernel1
    );

    var offsets = array<vec2<f32>, 9>(
        vec2(uv.x - stepx, uv.y + stepy), // 1
        vec2(uv.x, uv.y + stepy),
        vec2(uv.x + stepx, uv.y + stepy), // 3
        vec2(uv.x - stepx, uv.y),
        vec2(uv.x, uv.y), // 5
        vec2(uv.x, uv.y),
        vec2(uv.x - stepx, uv.y - stepy), // 7
        vec2(uv.x, uv.y - stepy),
        vec2(uv.x + stepx, uv.y - stepy) // 9
    );

    var gx = 0.0;
    gx += horizontalSobelMatrix[0] * getFragLuma(offsets[0]);
    gx += horizontalSobelMatrix[1] * getFragLuma(offsets[1]);
    gx += horizontalSobelMatrix[2] * getFragLuma(offsets[2]);
    gx += horizontalSobelMatrix[3] * getFragLuma(offsets[3]);
    gx += horizontalSobelMatrix[4] * getFragLuma(offsets[4]);
    gx += horizontalSobelMatrix[5] * getFragLuma(offsets[5]);
    gx += horizontalSobelMatrix[6] * getFragLuma(offsets[6]);
    gx += horizontalSobelMatrix[7] * getFragLuma(offsets[7]);
    gx += horizontalSobelMatrix[8] * getFragLuma(offsets[8]);


    var gy = 0.0;
    gy += verticalSobelMatrix[0] * getFragLuma(offsets[0]);
    gy += verticalSobelMatrix[1] * getFragLuma(offsets[1]);
    gy += verticalSobelMatrix[2] * getFragLuma(offsets[2]);
    gy += verticalSobelMatrix[3] * getFragLuma(offsets[3]);
    gy += verticalSobelMatrix[4] * getFragLuma(offsets[4]);
    gy += verticalSobelMatrix[5] * getFragLuma(offsets[5]);
    gy += verticalSobelMatrix[6] * getFragLuma(offsets[6]);
    gy += verticalSobelMatrix[7] * getFragLuma(offsets[7]);
    gy += verticalSobelMatrix[8] * getFragLuma(offsets[8]);

    var g = sqrt((pow(gx,2.0) + pow(gy,2.0))); // aggregates all sides

    var color = vec3(0.0);
    var red = vec3(1.0,0.0,0.0);
    var green = vec3(0.0,1.0,0.0);
    var blue = vec3(0.0,0.0,1.0);
    var yellow = vec3(1.0,1.0,0.0);
    var white = vec3(1.0,1.0,1.0);

    var div = 1.0/8.0;

    if(g > 0.0) {
        // get gradient vector
        var t = atan2(gy, gx); // theta
        t = (t/PI) * 0.5 + 0.5; // normalize theta
        var s = RADIAL_DIV;

        // quantize gradient vector to 4 different types
        // green
        if((t >= s && t <= 3.0*s) || (t >= 0.5 + s && t <= 0.5 + 3.0*s)) {
            color = green;
        }

        // yellow
        if((t >= 0.25 + s && t <= 0.25 + 3.0*s) || (t >= 0.75 + s && t <= 0.75 + 3.0*s)) {
            color = yellow;
        }

        // red
        if((t >= 1.0 - s || t <= s) || (t >= 0.5 - s && t <= 0.5 + s)) {
            color = red;
        }

        // blue
        if((t >= 0.25 - s && t <= 0.25 + s) || (t >= 0.75 - s && t <= 0.75 + s)) {
            color = blue;
        }
    }

    return vec4(color, 1.0);
}