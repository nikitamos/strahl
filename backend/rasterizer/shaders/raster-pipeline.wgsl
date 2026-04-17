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
@binding(4) @group(1) var diffuseTex_0 : texture_2d<f32>;

@binding(0) @group(1) var sampler_0 : sampler;

struct VertexOutput_0
{
    @builtin(position) position_0 : vec4<f32>,
    @location(0) uv_0 : vec2<f32>,
    @location(1) vertex_normal_0 : vec3<f32>,
};

fn VertexOutput_x24init_0( position_1 : vec4<f32>,  uv_1 : vec2<f32>,  vertex_normal_1 : vec3<f32>) -> VertexOutput_0
{
    var _S1 : VertexOutput_0;
    _S1.position_0 = position_1;
    _S1.uv_0 = uv_1;
    _S1.vertex_normal_0 = vertex_normal_1;
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
    return VertexOutput_x24init_0(((((((mat4x4<f32>(mat4x4<f32>(global_0.projection_0.data_0[i32(0)][i32(0)], global_0.projection_0.data_0[i32(0)][i32(1)], global_0.projection_0.data_0[i32(0)][i32(2)], global_0.projection_0.data_0[i32(0)][i32(3)], global_0.projection_0.data_0[i32(1)][i32(0)], global_0.projection_0.data_0[i32(1)][i32(1)], global_0.projection_0.data_0[i32(1)][i32(2)], global_0.projection_0.data_0[i32(1)][i32(3)], global_0.projection_0.data_0[i32(2)][i32(0)], global_0.projection_0.data_0[i32(2)][i32(1)], global_0.projection_0.data_0[i32(2)][i32(2)], global_0.projection_0.data_0[i32(2)][i32(3)], global_0.projection_0.data_0[i32(3)][i32(0)], global_0.projection_0.data_0[i32(3)][i32(1)], global_0.projection_0.data_0[i32(3)][i32(2)], global_0.projection_0.data_0[i32(3)][i32(3)]))) * (mat4x4<f32>(mat4x4<f32>(global_0.camera_0.data_0[i32(0)][i32(0)], global_0.camera_0.data_0[i32(0)][i32(1)], global_0.camera_0.data_0[i32(0)][i32(2)], global_0.camera_0.data_0[i32(0)][i32(3)], global_0.camera_0.data_0[i32(1)][i32(0)], global_0.camera_0.data_0[i32(1)][i32(1)], global_0.camera_0.data_0[i32(1)][i32(2)], global_0.camera_0.data_0[i32(1)][i32(3)], global_0.camera_0.data_0[i32(2)][i32(0)], global_0.camera_0.data_0[i32(2)][i32(1)], global_0.camera_0.data_0[i32(2)][i32(2)], global_0.camera_0.data_0[i32(2)][i32(3)], global_0.camera_0.data_0[i32(3)][i32(0)], global_0.camera_0.data_0[i32(3)][i32(1)], global_0.camera_0.data_0[i32(3)][i32(2)], global_0.camera_0.data_0[i32(3)][i32(3)])))))) * (vec4<f32>(_S2.vertex_local_pos_0, 1.0f)))), _S2.uv_2, _S2.normal_1);
}

@id(0) override COLORS_0 : u32;

fn GetDiffuse_0( uv_3 : vec2<f32>) -> vec4<f32>
{
    if(bool((COLORS_0 & (u32(8)))))
    {
        return colors_0.diffuse_0;
    }
    return (textureSample((diffuseTex_0), (sampler_0), (uv_3)));
}

struct pixelOutput_0
{
    @location(0) output_0 : vec4<f32>,
};

struct pixelInput_0
{
    @location(0) uv_4 : vec2<f32>,
    @location(1) vertex_normal_2 : vec3<f32>,
};

@fragment
fn RasterizerPbrFS( _S3 : pixelInput_0, @builtin(position) position_2 : vec4<f32>) -> pixelOutput_0
{
    var _S4 : pixelOutput_0 = pixelOutput_0( GetDiffuse_0(_S3.uv_4) );
    return _S4;
}

