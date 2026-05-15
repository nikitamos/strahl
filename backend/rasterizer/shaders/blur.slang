struct Globals
{
    row_major float4x4 projection;
    row_major float4x4 camera;
    uint2 viewport_size;
};

import tonemapping;

[[vk::binding(0, 0)]] Texture2D<float4> unsmoothed;
[[vk::binding(1, 0)]] SamplerState smp;

[[vk::binding(0, 1)]] StructuredBuffer<float> kernel;
[[vk::binding(1, 1)]] Texture2D<float4> origin;

[[vk::binding(0, 2)]] ConstantBuffer<Globals> g;

static int SIDE;
static int2 CENTER;
static int DIM_LEN;
static int ARRAY_LEN;

static float2 dh;
static float2 dx;
static const float THRESHOLD = 0.6;

float at(int2 i)
{
    return kernel[i.x + i.y * DIM_LEN];
}

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
    ARRAY_LEN = 0;
    uint stride;
    kernel.GetDimensions(ARRAY_LEN, stride);
    
    DIM_LEN = int(sqrt(float(ARRAY_LEN)));
    SIDE = (DIM_LEN - 1) / 2;
    CENTER = int2(SIDE, SIDE);
    
    FOut o;
    o.norm = float4(0.0,0.0,0.0,1.0);

    let curSample = unsmoothed.Sample(smp, input.texcoord.xy);

    float2 size = float2(g.viewport_size.x, g.viewport_size.y);
    dx = float2(1.0f / size.x, 0.0f);
    float2 dy = float2(0.0f, 1.0f / size.y);
    dh = dx + dy;
    
    float2 px = float2(-SIDE, -SIDE);
    for (; px.x < SIDE; px.x += 1.0f)
    {
        for (px.y = -SIDE; px.y < SIDE; px.y += 1.0f)
        {
            float2 pos = float2(px);
            float2 samplePos = input.texcoord.xy + dh * pos; 
            float4 sample = unsmoothed.Sample(smp, samplePos);
            o.norm += sample * at(int2(px + CENTER));
        }
    }
    
    let originSample = origin.Sample(smp, input.texcoord.xy);
    o.norm.xyz += originSample.xyz;
    o.norm.xyz = tonemapping::AcesApprox(o.norm.xyz);
    o.norm.w = curSample.w;
    
    return o;
}

[shader("fragment")]
FOut bright(VOut input) {
    FOut o;
    o.norm = float4(0.0,0.0,0.0,1.0);

    let curSample = unsmoothed.Sample(smp, input.texcoord.xy);
    let brightness = dot(curSample.xyz, curSample.xyz);
    if (brightness > THRESHOLD) {
        o.norm.xyz = curSample.xyz;
    }
    return o;
}