[[vk::binding(0, 0)]] Texture2D<float4> unsmoothed : register(t0);
[[vk::binding(1, 0)]] SamplerState smp : register(s0);

import tonemapping;

struct VOut
{
    float4 clip_pos : SV_POSITION;
    float4 texcoord : TEXCOORD0;
};

[shader("fragment")]
float4 fs_main(VOut input) : SV_Target
{   
    let color = unsmoothed.Sample(smp, input.texcoord.xy);
    return float4(tonemapping::AcesApprox(color.xyz), color.w);
}