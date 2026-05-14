@binding(0) @group(1) var<storage, read> kernel_0 : array<f32>;

struct _MatrixStorage_float4x4std140_0
{
    @align(16) data_0 : array<vec4<f32>, i32(4)>,
};

struct Globals_std140_0
{
    @align(16) projection_0 : _MatrixStorage_float4x4std140_0,
    @align(16) camera_0 : _MatrixStorage_float4x4std140_0,
    @align(16) viewport_size_0 : vec2<u32>,
};

@binding(0) @group(2) var<uniform> g_0 : Globals_std140_0;
@binding(0) @group(0) var unsmoothed_0 : texture_2d<f32>;

@binding(1) @group(0) var smp_0 : sampler;

var<private> ARRAY_LEN_0 : i32;

var<private> DIM_LEN_0 : i32;

var<private> SIDE_0 : i32;

var<private> CENTER_0 : vec2<i32>;

var<private> dx_0 : vec2<f32>;

var<private> dh_0 : vec2<f32>;

fn at_0( i_0 : vec2<i32>) -> f32
{
    return kernel_0[i_0.x + i_0.y * DIM_LEN_0];
}

struct FOut_0
{
    @builtin(frag_depth) depth_0 : f32,
    @location(0) norm_0 : vec4<f32>,
};

struct pixelInput_0
{
    @location(0) texcoord_0 : vec4<f32>,
};

@fragment
fn fs_main( _S1 : pixelInput_0, @builtin(position) clip_pos_0 : vec4<f32>) -> FOut_0
{
    ARRAY_LEN_0 = i32(0);
    var _S2 : vec2<u32> = vec2<u32>(arrayLength(&kernel_0), 4);
    var _S3 : i32 = i32(_S2.x);
    ARRAY_LEN_0 = _S3;
    var _S4 : i32 = i32(sqrt(f32(_S3)));
    DIM_LEN_0 = _S4;
    var _S5 : i32 = (_S4 - i32(1)) / i32(2);
    SIDE_0 = _S5;
    CENTER_0 = vec2<i32>(_S5, _S5);
    var o_0 : FOut_0;
    o_0.depth_0 = 0.0f;
    o_0.norm_0 = vec4<f32>(0.0f, 0.0f, 0.0f, 0.0f);
    var _S6 : f32 = f32(g_0.viewport_size_0.y);
    var _S7 : vec2<f32> = vec2<f32>(1.0f / f32(g_0.viewport_size_0.x), 0.0f);
    dx_0 = _S7;
    dh_0 = _S7 + vec2<f32>(0.0f, 1.0f / _S6);
    var _S8 : f32 = f32(- _S5);
    var px_0 : vec2<f32> = vec2<f32>(_S8, _S8);
    for(;;)
    {
        if((px_0.x) < f32(SIDE_0))
        {
        }
        else
        {
            break;
        }
        px_0[i32(1)] = f32(- SIDE_0);
        for(;;)
        {
            if((px_0.y) < f32(SIDE_0))
            {
            }
            else
            {
                break;
            }
            o_0.norm_0 = o_0.norm_0 + (textureSample((unsmoothed_0), (smp_0), (_S1.texcoord_0.xy + dh_0 * px_0))).xyzw * vec4<f32>(at_0(vec2<i32>(px_0 + vec2<f32>(CENTER_0))));
            px_0[i32(1)] = px_0[i32(1)] + 1.0f;
        }
        px_0[i32(0)] = px_0[i32(0)] + 1.0f;
    }
    return o_0;
}

