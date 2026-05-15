[[vk::binding(0, 0)]] Texture2D<float4> unsmoothed : register(t0);
[[vk::binding(1, 0)]] SamplerState smp : register(s0);


float3 AcesApprox(float3 v)
{
    v *= 0.6f;
    float a = 2.51f;
    float b = 0.03f;
    float c = 2.43f;
    float d = 0.59f;
    float e = 0.14f;
    return clamp((v*(a*v+b))/(v*(c*v+d)+e), 0.0f, 1.0f);
}

struct VOut
{
    float4 clip_pos : SV_POSITION;
    float4 texcoord : TEXCOORD0;
};

[shader("fragment")]
float4 fs_main(VOut input) : SV_Target
{   
    let color = unsmoothed.Sample(smp, input.texcoord.xy);
    return float4(AcesApprox(color.xyz), color.w);
}