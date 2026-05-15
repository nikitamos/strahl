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

fn tonemapping_AcesApprox_0( v_0 : vec3<f32>) -> vec3<f32>
{
    var _S1 : vec3<f32> = v_0 * vec3<f32>(0.60000002384185791f);
    return clamp(_S1 * (vec3<f32>(2.50999999046325684f) * _S1 + vec3<f32>(0.02999999932944775f)) / (_S1 * (vec3<f32>(2.43000006675720215f) * _S1 + vec3<f32>(0.5899999737739563f)) + vec3<f32>(0.14000000059604645f)), vec3<f32>(0.0f), vec3<f32>(1.0f));
}

struct FOut_0
{
    @location(0) norm_0 : vec4<f32>,
};

struct pixelInput_0
{
    @location(0) texcoord_0 : vec4<f32>,
};

@fragment
fn blur( _S2 : pixelInput_0, @builtin(position) clip_pos_0 : vec4<f32>) -> FOut_0
{
    var o_0 : FOut_0;
    const _S3 : vec4<f32> = vec4<f32>(0.0f, 0.0f, 0.0f, 1.0f);
    texelSize_0 = vec2<f32>(1.0f / f32(g_0.viewport_size_0.x), 1.0f / f32(g_0.viewport_size_0.y));
    var _S4 : vec2<f32> = _S2.texcoord_0.xy;
    var curSample_0 : vec4<f32> = (textureSample((unsmoothed_0), (smp_0), (_S4)));
    o_0.norm_0 = _S3 + curSample_0 * vec4<f32>(0.22702699899673462f);
    var i_0 : i32;
    if((push_0.horizontal_0) == u32(1))
    {
        i_0 = i32(1);
        for(;;)
        {
            if(i_0 < i32(5))
            {
            }
            else
            {
                break;
            }
            var offset_0 : vec2<f32> = vec2<f32>(texelSize_0.x * f32(i_0), 0.0f);
            o_0.norm_0 = o_0.norm_0 + ((textureSample((unsmoothed_0), (smp_0), (_S4 + offset_0))) + (textureSample((unsmoothed_0), (smp_0), (_S4 - offset_0)))) * vec4<f32>(weights_0[i_0]);
            i_0 = i_0 + i32(1);
        }
    }
    else
    {
        i_0 = i32(1);
        for(;;)
        {
            if(i_0 < i32(5))
            {
            }
            else
            {
                break;
            }
            var offset_1 : vec2<f32> = vec2<f32>(0.0f, texelSize_0.y * f32(i_0));
            o_0.norm_0 = o_0.norm_0 + ((textureSample((unsmoothed_0), (smp_0), (_S4 + offset_1))) + (textureSample((unsmoothed_0), (smp_0), (_S4 - offset_1)))) * vec4<f32>(weights_0[i_0]);
            i_0 = i_0 + i32(1);
        }
    }
    o_0.norm_0[i32(3)] = curSample_0.w;
    if((push_0.horizontal_0) == u32(0))
    {
        var _S5 : vec3<f32> = tonemapping_AcesApprox_0((textureSample((origin_0), (smp_0), (_S4))).xyz + o_0.norm_0.xyz);
        o_0.norm_0.x = _S5.x;
        o_0.norm_0.y = _S5.y;
        o_0.norm_0.z = _S5.z;
    }
    return o_0;
}

struct pixelInput_1
{
    @location(0) texcoord_1 : vec4<f32>,
};

@fragment
fn bright( _S6 : pixelInput_1, @builtin(position) clip_pos_1 : vec4<f32>) -> FOut_0
{
    var o_1 : FOut_0;
    o_1.norm_0 = vec4<f32>(0.0f, 0.0f, 0.0f, 1.0f);
    var _S7 : vec3<f32> = (textureSample((unsmoothed_0), (smp_0), (_S6.texcoord_1.xy))).xyz;
    if((dot(_S7, _S7)) > 0.60000002384185791f)
    {
        o_1.norm_0.x = _S7.x;
        o_1.norm_0.y = _S7.y;
        o_1.norm_0.z = _S7.z;
    }
    return o_1;
}

