@binding(0) @group(1) var skybox_0 : texture_cube<f32>;

@binding(1) @group(1) var sampler_0 : sampler;

const COORDS_0 : array<vec4<f32>, i32(4)> = array<vec4<f32>, i32(4)>( vec4<f32>(-1.0f, -1.0f, 1.0f, 1.0f), vec4<f32>(-1.0f, 1.0f, 1.0f, 1.0f), vec4<f32>(1.0f, -1.0f, 1.0f, 1.0f), vec4<f32>(1.0f, 1.0f, 1.0f, 1.0f) );
struct VSOut_0
{
    @builtin(position) pos_0 : vec4<f32>,
    @location(0) tex_0 : vec2<f32>,
};

fn VSOut_x24init_0( pos_1 : vec4<f32>,  tex_1 : vec2<f32>) -> VSOut_0
{
    var _S1 : VSOut_0;
    _S1.pos_0 = pos_1;
    _S1.tex_0 = tex_1;
    return _S1;
}

@vertex
fn SkyboxVertex(@builtin(vertex_index) i_0 : u32) -> VSOut_0
{
    var i_1 : i32 = i32(i_0);
    return VSOut_x24init_0(COORDS_0[i_1], COORDS_0[i_1].xy * vec2<f32>(0.5f) + vec2<f32>(1.0f));
}

struct pixelOutput_0
{
    @location(0) output_0 : vec4<f32>,
};

struct pixelInput_0
{
    @location(0) tex_2 : vec2<f32>,
};

@fragment
fn SkyboxFragment( _S2 : pixelInput_0, @builtin(position) pos_2 : vec4<f32>) -> pixelOutput_0
{
    var _S3 : pixelOutput_0 = pixelOutput_0( (textureSample((skybox_0), (sampler_0), (normalize(vec3<f32>(_S2.tex_2, 1.0f))))) );
    return _S3;
}

