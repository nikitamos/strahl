[vk::binding(0,1)]
uniform TextureCube skybox;
[vk::binding(1,1)]
uniform SamplerState sampler;

struct VSOut {
    float4 pos: SV_Position;
    float2 tex: TEXCOORD0;
}

static const float4 COORDS[4] = {
    {-1., -1., 1., 1.},
    {-1.,  1., 1., 1.},
    { 1., -1., 1., 1.},
    { 1.,  1., 1., 1.}
};

[shader("vertex")]
VSOut SkyboxVertex(int i: SV_VertexID) {
    return {COORDS[i], COORDS[i].xy*0.5 + 1.0};
}


[shader("fragment")]
float4 SkyboxFragment(VSOut in) : SV_Target {
    return skybox.Sample(sampler, normalize(float3(in.tex, 1.0)));
}