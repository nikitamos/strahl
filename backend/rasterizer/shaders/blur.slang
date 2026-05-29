struct Globals
{
    row_major float4x4 projection;
    row_major float4x4 camera;
    uint2 viewport_size;
};

struct PushConstants
{
    uint32_t horizontal;
};

import tonemapping;

[[vk::binding(0, 0)]] Texture2D<float4> unsmoothed;
[[vk::binding(1, 0)]] SamplerState smp;

[[vk::binding(0, 1)]] Texture2D<float4> origin;

[[vk::binding(0, 2)]] ConstantBuffer<Globals> g;

[[vk::push_constant]] PushConstants push;

static float2 texelSize;
static const float THRESHOLD = 0.6;

#define WEIGHT_COUNT 5
static const float weights[WEIGHT_COUNT] = {0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216};

struct VOut
{
    float4 clip_pos : SV_POSITION;
    float4 texcoord : TEXCOORD0;
};

struct FOut
{
    float4 norm : SV_TARGET0;
};

[shader("fragment")]
FOut blur(VOut input)
{
    FOut o;
    o.norm = float4(0.0, 0.0, 0.0, 1.0);

    float2 size = float2(g.viewport_size.x, g.viewport_size.y);
    texelSize = 2*float2(1.0f / size.x, 1.0f / size.y); // textures are 0.5x scaled

    float4 curSample = unsmoothed.Sample(smp, input.texcoord.xy);
    float2 offset = 2*(push.horizontal == 1 ? float2(texelSize.x, 0.0) : float2(0.0, texelSize.y));
    
    o.norm += curSample * weights[0];
    for (int i = 1; i < WEIGHT_COUNT; i++)
    {
        float4 sample1 = unsmoothed.Sample(smp, input.texcoord.xy + offset * i);
        float4 sample2 = unsmoothed.Sample(smp, input.texcoord.xy - offset * i);
        o.norm += (sample1 + sample2) * weights[i];
    }
    
    o.norm.w = curSample.w;

    return o;
}

[shader("fragment")]
FOut bright(VOut input)
{
    FOut o;
    o.norm = float4(0.0, 0.0, 0.0, 1.0);

    let curSample = unsmoothed.Sample(smp, input.texcoord.xy);
    let brightness = dot(curSample.xyz, curSample.xyz);
    if (brightness >= THRESHOLD) {
        o.norm.xyz = curSample.xyz; // * (1.2 + smoothstep(THRESHOLD, 1.0, brightness));
    }
    return o;
}

[shader("fragment")]
FOut merge(VOut input) {
    FOut o;
    let unsmoothedSample = unsmoothed.Sample(smp, input.texcoord.xy);
    let originSample = origin.Sample(smp, input.texcoord.xy);
    o.norm.xyz = tanh(unsmoothedSample.xyz + originSample.xyz);
    o.norm.w = 1.0;
    return o;
}