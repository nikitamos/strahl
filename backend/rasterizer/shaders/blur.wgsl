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

struct PushConstants_std430_0
{
    @align(4) horizontal_0 : u32,
};

var<immediate> push_0 : PushConstants_std430_0;
@binding(0) @group(1) var origin_0 : texture_2d<f32>;

const weights_0 : array<f32, i32(5)> = array<f32, i32(5)>( 0.22702699899673462f, 0.194594606757164f, 0.12162160128355026f, 0.05405399948358536f, 0.01621600054204464f );
var<private> texelSize_0 : vec2<f32>;

struct FOut_0
{
    @location(0) norm_0 : vec4<f32>,
};

struct pixelInput_0
{
    @location(0) texcoord_0 : vec4<f32>,
};

@fragment
fn blur( _S1 : pixelInput_0, @builtin(position) clip_pos_0 : vec4<f32>) -> FOut_0
{
    var o_0 : FOut_0;
    o_0.norm_0 = vec4<f32>(0.0f, 0.0f, 0.0f, 1.0f);
    var _S2 : vec2<f32> = vec2<f32>(2.0f);
    texelSize_0 = _S2 * vec2<f32>(1.0f / f32(g_0.viewport_size_0.x), 1.0f / f32(g_0.viewport_size_0.y));
    var _S3 : vec2<f32> = _S1.texcoord_0.xy;
    var curSample_0 : vec4<f32> = (textureSample((unsmoothed_0), (smp_0), (_S3)));
    var _S4 : vec2<f32>;
    if((push_0.horizontal_0) == u32(1))
    {
        _S4 = vec2<f32>(texelSize_0.x, 0.0f);
    }
    else
    {
        _S4 = vec2<f32>(0.0f, texelSize_0.y);
    }
    var _S5 : vec2<f32> = _S2 * _S4;
    o_0.norm_0 = o_0.norm_0 + curSample_0 * vec4<f32>(0.22702699899673462f);
    var i_0 : i32 = i32(1);
    for(;;)
    {
        if(i_0 < i32(5))
        {
        }
        else
        {
            break;
        }
        var _S6 : vec2<f32> = _S5 * vec2<f32>(f32(i_0));
        o_0.norm_0 = o_0.norm_0 + ((textureSample((unsmoothed_0), (smp_0), (_S3 + _S6))) + (textureSample((unsmoothed_0), (smp_0), (_S3 - _S6)))) * vec4<f32>(weights_0[i_0]);
        i_0 = i_0 + i32(1);
    }
    o_0.norm_0[i32(3)] = curSample_0.w;
    return o_0;
}

struct pixelInput_1
{
    @location(0) texcoord_1 : vec4<f32>,
};

@fragment
fn bright( _S7 : pixelInput_1, @builtin(position) clip_pos_1 : vec4<f32>) -> FOut_0
{
    var o_1 : FOut_0;
    o_1.norm_0 = vec4<f32>(0.0f, 0.0f, 0.0f, 1.0f);
    var _S8 : vec3<f32> = (textureSample((unsmoothed_0), (smp_0), (_S7.texcoord_1.xy))).xyz;
    if((dot(_S8, _S8)) >= 0.60000002384185791f)
    {
        o_1.norm_0.x = _S8.x;
        o_1.norm_0.y = _S8.y;
        o_1.norm_0.z = _S8.z;
    }
    return o_1;
}

struct pixelInput_2
{
    @location(0) texcoord_2 : vec4<f32>,
};

@fragment
fn merge( _S9 : pixelInput_2, @builtin(position) clip_pos_2 : vec4<f32>) -> FOut_0
{
    var _S10 : vec2<f32> = _S9.texcoord_2.xy;
    var o_2 : FOut_0;
    var _S11 : vec3<f32> = tanh((textureSample((unsmoothed_0), (smp_0), (_S10))).xyz + (textureSample((origin_0), (smp_0), (_S10))).xyz);
    o_2.norm_0.x = _S11.x;
    o_2.norm_0.y = _S11.y;
    o_2.norm_0.z = _S11.z;
    o_2.norm_0[i32(3)] = 1.0f;
    return o_2;
}

