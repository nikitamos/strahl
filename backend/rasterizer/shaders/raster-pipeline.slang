import globals;


[SpecializationConstant]
uint32_t COLORS;

static const uint32_t ROUGHNESS = 0x01;
static const uint32_t SPECULAR = 0x02;
static const uint32_t GLOSSY = 0x04;
static const uint32_t DIFFUSE = 0x08;
static const uint32_t EMISSION = 0x10;
static const uint32_t NORMAL = 0x20;  // Added for completeness

struct Colors {
    float4 roughness;
    float4 specular;
    float4 glossy;
    float4 diffuse;
    float4 emission;
    float4 normal;
};

[vk::binding(0,1)]
uniform SamplerState sampler;
[vk::binding(1,1)]
uniform Texture2D roughnessTex;
[vk::binding(2,1)]
uniform Texture2D specularTex;
[vk::binding(3,1)]
uniform Texture2D glossyTex;
[vk::binding(4,1)]
uniform Texture2D diffuseTex;
[vk::binding(5,1)]
uniform Texture2D emissionTex;
[vk::binding(6,1)]
uniform Texture2D normalTex;
[vk::binding(7,1)]
ConstantBuffer<Colors> colors;

struct Body {
    float4x4 body2world;
};

[vk::push_constant]
ConstantBuffer<Body> body;

float4 GetRoughness(float2 uv) {
    if (COLORS & ROUGHNESS) {
        return colors.roughness;
    }
    return roughnessTex.Sample(sampler, uv);
}

float4 GetSpecular(float2 uv) {
    if (COLORS & SPECULAR) {
        return colors.specular;
    }
    return specularTex.Sample(sampler, uv);
}

float4 GetGlossy(float2 uv) {
    if (COLORS & GLOSSY) {
        return colors.glossy;
    }
    return glossyTex.Sample(sampler, uv);
}

float4 GetDiffuse(float2 uv) {
    if (COLORS & DIFFUSE) {
        return colors.diffuse;
    }
    return diffuseTex.Sample(sampler, uv);
}

float4 GetEmission(float2 uv) {
    if (COLORS & EMISSION) {
        return colors.emission;
    }
    return emissionTex.Sample(sampler, uv);
}

float4 GetNormal(float2 uv) {
    if (COLORS & NORMAL) {
        return colors.normal;
    }
    return normalTex.Sample(sampler, uv);
}

struct VertexOutput {
    float4 position: SV_Position;
    float2 uv: TEXCOORD0;
    float3 vertex_normal: TEXCOORD1;
};

[shader("vertex")]
VertexOutput MeshGeometryVS(
    float3 vertex_local_pos: POSITION0,
    float3 normal: POSITION1,
    float2 uv: POSITION2,
) {
    var transform = mul(global.camera, global.projection);
    var position = mul(float4(vertex_local_pos, 1.0),
                   transform
    );
    return VertexOutput(position, uv, normal);
}

[shader("fragment")]
float4 RasterizerPbrFS(VertexOutput in) : SV_Target {
    return GetDiffuse(in.uv);
}
