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
@binding(0) @group(1) var skybox_0 : texture_cube<f32>;

@binding(1) @group(1) var sampler_0 : sampler;

const COORDS_0 : array<vec3<f32>, i32(14)> = array<vec3<f32>, i32(14)>( vec3<f32>(-1.0f, 1.0f, 1.0f), vec3<f32>(-1.0f, -1.0f, 1.0f), vec3<f32>(1.0f, 1.0f, 1.0f), vec3<f32>(1.0f, -1.0f, 1.0f), vec3<f32>(1.0f, -1.0f, -1.0f), vec3<f32>(1.0f, 1.0f, 1.0f), vec3<f32>(1.0f, 1.0f, -1.0f), vec3<f32>(-1.0f, 1.0f, 1.0f), vec3<f32>(-1.0f, 1.0f, -1.0f), vec3<f32>(-1.0f, -1.0f, 1.0f), vec3<f32>(-1.0f, -1.0f, -1.0f), vec3<f32>(1.0f, -1.0f, -1.0f), vec3<f32>(-1.0f, 1.0f, -1.0f), vec3<f32>(1.0f, 1.0f, -1.0f) );
struct VSOut_0
{
    @builtin(position) pos_0 : vec4<f32>,
    @location(0) vec_0 : vec3<f32>,
};

fn VSOut_x24init_0( pos_1 : vec4<f32>,  vec_1 : vec3<f32>) -> VSOut_0
{
    var _S1 : VSOut_0;
    _S1.pos_0 = pos_1;
    _S1.vec_0 = vec_1;
    return _S1;
}

@vertex
fn SkyboxVertex(@builtin(vertex_index) i_0 : u32) -> VSOut_0
{
    var i_1 : i32 = i32(i_0);
    var _S2 : mat4x4<f32> = mat4x4<f32>(mat4x4<f32>(global_0.camera_0.data_0[i32(0)][i32(0)], global_0.camera_0.data_0[i32(0)][i32(1)], global_0.camera_0.data_0[i32(0)][i32(2)], global_0.camera_0.data_0[i32(0)][i32(3)], global_0.camera_0.data_0[i32(1)][i32(0)], global_0.camera_0.data_0[i32(1)][i32(1)], global_0.camera_0.data_0[i32(1)][i32(2)], global_0.camera_0.data_0[i32(1)][i32(3)], global_0.camera_0.data_0[i32(2)][i32(0)], global_0.camera_0.data_0[i32(2)][i32(1)], global_0.camera_0.data_0[i32(2)][i32(2)], global_0.camera_0.data_0[i32(2)][i32(3)], global_0.camera_0.data_0[i32(3)][i32(0)], global_0.camera_0.data_0[i32(3)][i32(1)], global_0.camera_0.data_0[i32(3)][i32(2)], global_0.camera_0.data_0[i32(3)][i32(3)]));
    return VSOut_x24init_0((((mat4x4<f32>(mat4x4<f32>(global_0.projection_0.data_0[i32(0)][i32(0)], global_0.projection_0.data_0[i32(0)][i32(1)], global_0.projection_0.data_0[i32(0)][i32(2)], global_0.projection_0.data_0[i32(0)][i32(3)], global_0.projection_0.data_0[i32(1)][i32(0)], global_0.projection_0.data_0[i32(1)][i32(1)], global_0.projection_0.data_0[i32(1)][i32(2)], global_0.projection_0.data_0[i32(1)][i32(3)], global_0.projection_0.data_0[i32(2)][i32(0)], global_0.projection_0.data_0[i32(2)][i32(1)], global_0.projection_0.data_0[i32(2)][i32(2)], global_0.projection_0.data_0[i32(2)][i32(3)], global_0.projection_0.data_0[i32(3)][i32(0)], global_0.projection_0.data_0[i32(3)][i32(1)], global_0.projection_0.data_0[i32(3)][i32(2)], global_0.projection_0.data_0[i32(3)][i32(3)]))) * (vec4<f32>((((mat3x3<f32>(_S2[i32(0)].xyz, _S2[i32(1)].xyz, _S2[i32(2)].xyz)) * (COORDS_0[i_1]))), 1.0f)))).xyww, COORDS_0[i_1].xyz);
}

struct pixelOutput_0
{
    @location(0) output_0 : vec4<f32>,
};

struct pixelInput_0
{
    @location(0) vec_2 : vec3<f32>,
};

@fragment
fn SkyboxFragment( _S3 : pixelInput_0, @builtin(position) pos_2 : vec4<f32>) -> pixelOutput_0
{
    var _S4 : pixelOutput_0 = pixelOutput_0( (textureSample((skybox_0), (sampler_0), (_S3.vec_2))) );
    return _S4;
}

