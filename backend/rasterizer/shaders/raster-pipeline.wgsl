struct _MatrixStorage_float4x4std430_0
{
    @align(16) data_0 : array<vec4<f32>, i32(4)>,
};

struct Body_std430_0
{
    @align(16) body2world_0 : _MatrixStorage_float4x4std430_0,
    @align(16) center_world_0 : vec3<f32>,
};

var<immediate> body_0 : Body_std430_0;
struct _MatrixStorage_float4x4std140_0
{
    @align(16) data_1 : array<vec4<f32>, i32(4)>,
};

struct Globals_std140_0
{
    @align(16) projection_0 : _MatrixStorage_float4x4std140_0,
    @align(16) camera_0 : _MatrixStorage_float4x4std140_0,
    @align(16) viewport_size_0 : vec2<u32>,
};

@binding(0) @group(0) var<uniform> global_0 : Globals_std140_0;
struct Colors_std140_0
{
    @align(16) roughness_0 : vec4<f32>,
    @align(16) specular_0 : vec4<f32>,
    @align(16) glossy_0 : vec4<f32>,
    @align(16) diffuse_0 : vec4<f32>,
    @align(16) emission_0 : vec4<f32>,
    @align(16) normal_0 : vec4<f32>,
};

@binding(7) @group(1) var<uniform> colors_0 : Colors_std140_0;
@binding(6) @group(1) var normalTex_0 : texture_2d<f32>;

@binding(0) @group(1) var sampler_0 : sampler;

@binding(3) @group(1) var glossyTex_0 : texture_2d<f32>;

@binding(5) @group(1) var emissionTex_0 : texture_2d<f32>;

@binding(4) @group(1) var diffuseTex_0 : texture_2d<f32>;

struct VertexOutput_0
{
    @builtin(position) position_0 : vec4<f32>,
    @location(0) uv_0 : vec2<f32>,
    @location(2) light_dir_0 : vec3<f32>,
    @location(3) halfway_dir_0 : vec3<f32>,
    @location(4) camera_dir_0 : vec3<f32>,
};

fn VertexOutput_x24init_0( position_1 : vec4<f32>,  uv_1 : vec2<f32>,  light_dir_1 : vec3<f32>,  halfway_dir_1 : vec3<f32>,  camera_dir_1 : vec3<f32>) -> VertexOutput_0
{
    var _S1 : VertexOutput_0;
    _S1.position_0 = position_1;
    _S1.uv_0 = uv_1;
    _S1.light_dir_0 = light_dir_1;
    _S1.halfway_dir_0 = halfway_dir_1;
    _S1.camera_dir_0 = camera_dir_1;
    return _S1;
}

struct vertexInput_0
{
    @location(0) vertex_local_pos_0 : vec3<f32>,
    @location(1) normal_1 : vec3<f32>,
    @location(2) uv_2 : vec2<f32>,
};

@vertex
fn MeshGeometryVS( _S2 : vertexInput_0) -> VertexOutput_0
{
    var transform_0 : mat4x4<f32> = ((((((mat4x4<f32>(mat4x4<f32>(global_0.projection_0.data_1[i32(0)][i32(0)], global_0.projection_0.data_1[i32(0)][i32(1)], global_0.projection_0.data_1[i32(0)][i32(2)], global_0.projection_0.data_1[i32(0)][i32(3)], global_0.projection_0.data_1[i32(1)][i32(0)], global_0.projection_0.data_1[i32(1)][i32(1)], global_0.projection_0.data_1[i32(1)][i32(2)], global_0.projection_0.data_1[i32(1)][i32(3)], global_0.projection_0.data_1[i32(2)][i32(0)], global_0.projection_0.data_1[i32(2)][i32(1)], global_0.projection_0.data_1[i32(2)][i32(2)], global_0.projection_0.data_1[i32(2)][i32(3)], global_0.projection_0.data_1[i32(3)][i32(0)], global_0.projection_0.data_1[i32(3)][i32(1)], global_0.projection_0.data_1[i32(3)][i32(2)], global_0.projection_0.data_1[i32(3)][i32(3)]))) * (mat4x4<f32>(mat4x4<f32>(global_0.camera_0.data_1[i32(0)][i32(0)], global_0.camera_0.data_1[i32(0)][i32(1)], global_0.camera_0.data_1[i32(0)][i32(2)], global_0.camera_0.data_1[i32(0)][i32(3)], global_0.camera_0.data_1[i32(1)][i32(0)], global_0.camera_0.data_1[i32(1)][i32(1)], global_0.camera_0.data_1[i32(1)][i32(2)], global_0.camera_0.data_1[i32(1)][i32(3)], global_0.camera_0.data_1[i32(2)][i32(0)], global_0.camera_0.data_1[i32(2)][i32(1)], global_0.camera_0.data_1[i32(2)][i32(2)], global_0.camera_0.data_1[i32(2)][i32(3)], global_0.camera_0.data_1[i32(3)][i32(0)], global_0.camera_0.data_1[i32(3)][i32(1)], global_0.camera_0.data_1[i32(3)][i32(2)], global_0.camera_0.data_1[i32(3)][i32(3)])))))) * (mat4x4<f32>(mat4x4<f32>(body_0.body2world_0.data_0[i32(0)][i32(0)], body_0.body2world_0.data_0[i32(0)][i32(1)], body_0.body2world_0.data_0[i32(0)][i32(2)], body_0.body2world_0.data_0[i32(0)][i32(3)], body_0.body2world_0.data_0[i32(1)][i32(0)], body_0.body2world_0.data_0[i32(1)][i32(1)], body_0.body2world_0.data_0[i32(1)][i32(2)], body_0.body2world_0.data_0[i32(1)][i32(3)], body_0.body2world_0.data_0[i32(2)][i32(0)], body_0.body2world_0.data_0[i32(2)][i32(1)], body_0.body2world_0.data_0[i32(2)][i32(2)], body_0.body2world_0.data_0[i32(2)][i32(3)], body_0.body2world_0.data_0[i32(3)][i32(0)], body_0.body2world_0.data_0[i32(3)][i32(1)], body_0.body2world_0.data_0[i32(3)][i32(2)], body_0.body2world_0.data_0[i32(3)][i32(3)])))));
    var position_2 : vec4<f32> = (((transform_0) * (vec4<f32>(_S2.vertex_local_pos_0, 1.0f))));
    const _S3 : vec3<f32> = vec3<f32>(0.0f, 1.0f, 0.0f);
    var up_0 : vec3<f32>;
    if((abs(_S2.normal_1.y)) > 0.99900001287460327f)
    {
        up_0 = vec3<f32>(1.0f, 0.0f, 0.0f);
    }
    else
    {
        up_0 = _S3;
    }
    var tangent_0 : vec3<f32> = normalize(cross(_S2.normal_1, normalize(cross(up_0, _S2.normal_1))));
    var TBN_0 : mat3x3<f32> = transpose(mat3x3<f32>(tangent_0, cross(_S2.normal_1, tangent_0), _S2.normal_1));
    var _S4 : vec3<f32> = position_2.xyz;
    var light_dir_2 : vec3<f32> = normalize(_S4 - (((transform_0) * (vec4<f32>(0.0f)))).xyz);
    var _S5 : vec3<f32> = (((vec4<f32>(0.0f, 0.0f, 0.0f, 1.0f)) * (mat4x4<f32>(mat4x4<f32>(global_0.camera_0.data_1[i32(0)][i32(0)], global_0.camera_0.data_1[i32(0)][i32(1)], global_0.camera_0.data_1[i32(0)][i32(2)], global_0.camera_0.data_1[i32(0)][i32(3)], global_0.camera_0.data_1[i32(1)][i32(0)], global_0.camera_0.data_1[i32(1)][i32(1)], global_0.camera_0.data_1[i32(1)][i32(2)], global_0.camera_0.data_1[i32(1)][i32(3)], global_0.camera_0.data_1[i32(2)][i32(0)], global_0.camera_0.data_1[i32(2)][i32(1)], global_0.camera_0.data_1[i32(2)][i32(2)], global_0.camera_0.data_1[i32(2)][i32(3)], global_0.camera_0.data_1[i32(3)][i32(0)], global_0.camera_0.data_1[i32(3)][i32(1)], global_0.camera_0.data_1[i32(3)][i32(2)], global_0.camera_0.data_1[i32(3)][i32(3)]))))).xyz - _S4;
    return VertexOutput_x24init_0(position_2, _S2.uv_2, (((light_dir_2) * (TBN_0))), (((normalize(light_dir_2 + normalize(_S5))) * (TBN_0))), (((_S5) * (TBN_0))));
}

@id(0) override COLORS_0 : u32;

fn GetNormal_0( uv_3 : vec2<f32>) -> vec4<f32>
{
    if(bool((COLORS_0 & (u32(32)))))
    {
        return colors_0.normal_0;
    }
    return (textureSample((normalTex_0), (sampler_0), (uv_3)));
}

fn GetGlossy_0( uv_4 : vec2<f32>) -> vec4<f32>
{
    if(bool((COLORS_0 & (u32(4)))))
    {
        return colors_0.glossy_0;
    }
    return (textureSample((glossyTex_0), (sampler_0), (uv_4)));
}

fn GetEmission_0( uv_5 : vec2<f32>) -> vec4<f32>
{
    if(bool((COLORS_0 & (u32(16)))))
    {
        return colors_0.emission_0;
    }
    return (textureSample((emissionTex_0), (sampler_0), (uv_5)));
}

fn GetDiffuse_0( uv_6 : vec2<f32>) -> vec4<f32>
{
    if(bool((COLORS_0 & (u32(8)))))
    {
        return colors_0.diffuse_0;
    }
    return (textureSample((diffuseTex_0), (sampler_0), (uv_6)));
}

struct pixelOutput_0
{
    @location(0) output_0 : vec4<f32>,
};

struct pixelInput_0
{
    @location(0) uv_7 : vec2<f32>,
    @location(2) light_dir_3 : vec3<f32>,
    @location(3) halfway_dir_2 : vec3<f32>,
    @location(4) camera_dir_2 : vec3<f32>,
};

@fragment
fn RasterizerPbrFS( _S6 : pixelInput_0, @builtin(position) position_3 : vec4<f32>) -> pixelOutput_0
{
    var normal_2 : vec3<f32> = GetNormal_0(_S6.uv_7).xyz * vec3<f32>(2.0f) - vec3<f32>(1.0f);
    var _S7 : pixelOutput_0 = pixelOutput_0( vec4<f32>(tanh((vec3<f32>(max(dot(normal_2, _S6.light_dir_3), 0.0f)) + GetGlossy_0(_S6.uv_7).xyz * vec3<f32>(pow(max(dot(normal_2, _S6.halfway_dir_2), 0.0f), 16.0f))) * GetDiffuse_0(_S6.uv_7).xyz + GetEmission_0(_S6.uv_7).xyz), 1.0f) );
    return _S7;
}

