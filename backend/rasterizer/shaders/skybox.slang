import globals;

[vk::binding(0,1)]
uniform TextureCube skybox;
[vk::binding(1,1)]
uniform SamplerState sampler;

struct VSOut {
    float4 pos: SV_Position;
    float3 vec: TEXCOORD0;
}

static const float3 COORDS[14] = {
    {-1.f,  1.f,  1.f},  // Front-top-left
    {-1.f, -1.f,  1.f},  // Front-bottom-left
    { 1.f,  1.f,  1.f},  // Front-top-right
    { 1.f, -1.f,  1.f},  // Front-bottom-right
    { 1.f, -1.f, -1.f},  // Back-bottom-right
    { 1.f,  1.f,  1.f},  // Front-top-right
    { 1.f,  1.f, -1.f},  // Back-top-right
    {-1.f,  1.f,  1.f},  // Front-top-left
    {-1.f,  1.f, -1.f},  // Back-top-left
    {-1.f, -1.f,  1.f},  // Front-bottom-left
    {-1.f, -1.f, -1.f},  // Back-bottom-left
    { 1.f, -1.f, -1.f},  // Back-bottom-right
    {-1.f,  1.f, -1.f},  // Back-top-left
    { 1.f,  1.f, -1.f}   // Back-top-right
};

[shader("vertex")]
VSOut SkyboxVertex(int i: SV_VertexID) {
    let coord = COORDS[i];
    let camera = (float3x3(global.camera));

    let eye = float4(mul(coord, camera), 1.0);
    let position = mul(eye, global.projection);
    return {position.xyww, coord.xyz};
}


[shader("fragment")]
float4 SkyboxFragment(VSOut in) : SV_Target {
    return skybox.Sample(sampler, in.vec);
}