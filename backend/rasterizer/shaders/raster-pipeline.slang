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
    float3 center_world;
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
    float3 light_dir: TEXCOORD2;
    float3 halfway_dir: TEXCOORD3;
    float3 camera_dir: TEXCOORD4;
};

[shader("vertex")]
VertexOutput MeshGeometryVS(
    float3 vertex_local_pos: POSITION0,
    float3 normal: POSITION1,
    float2 uv: POSITION2
) {
    var transform = mul(global.camera, global.projection);
    var position = mul(float4(vertex_local_pos, 1.0),
                   transform
    );
    var up = float3(0.0, 1.0, 0.0);
    // TODO: a dirty hack that only works, if the mesh is a sphere
    if (abs(normal.y) > 0.999) {
        up = float3(1.0, 0.0, 0.0);
    }
    float3 bitangent = normalize(cross(up, normal));
    float3 tangent = normalize(cross(normal, bitangent));
    float3x3 TBN = float3x3(tangent, cross(normal, tangent), normal);
    TBN = transpose(TBN);

    float3 light_dir = float3(0.0, 0.0, 1.0);
    float3 camera_pos = mul(global.camera, float4(0.0, 0.0, 0.0, 1.0)).xyz;
    float3 view_dir = normalize(camera_pos - position.xyz);
    float3 halfway_dir = normalize(light_dir + view_dir);

    return VertexOutput(position, uv,   
        mul(TBN, light_dir),
        mul(TBN, halfway_dir),
        mul(TBN, camera_pos - position.xyz)
    );
}

[shader("fragment")]
float4 RasterizerPbrFS(VertexOutput in) : SV_Target {
    float3 normal = GetNormal(in.uv).rgb * 2.0 - 1.0;
    float3 albedo = GetDiffuse(in.uv).rgb;
    float shininess = GetGlossy(in.uv).r;
    float diffuseness = saturate(dot(normal, in.light_dir));
    float glossiness = pow(max(dot(normal, in.halfway_dir), 0.0), shininess);

    float3 light_specular = float3(0.4, 0.4, 0.4);

    float3 diffuse = albedo * diffuseness;
    float3 specular = light_specular *
        GetSpecular(in.uv).rgb *
        pow(max(dot(in.camera_dir, in.light_dir), 0.0), shininess); // light is pure white
    float3 emission = GetEmission(in.uv).rgb;

    return float4(diffuse + specular + emission, 1.0);
}
