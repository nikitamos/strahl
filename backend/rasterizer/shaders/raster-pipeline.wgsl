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
    @location(0) body_position_0 : vec3<f32>,
    @location(1) uv_2 : vec2<f32>,
    @location(2) normal_1 : vec3<f32>,
};

@vertex
fn VertexShader( _S2 : vertexInput_0) -> VertexOutput_0
{
    return VertexOutput_x24init_0(vec4<f32>(_S2.body_position_0, 1.0f), _S2.uv_2, _S2.normal_1);
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
fn FragmentShader( _S3 : pixelInput_0, @builtin(position) position_2 : vec4<f32>) -> pixelOutput_0
{
    var _S4 : pixelOutput_0 = pixelOutput_0( GetDiffuse_0(_S3.uv_4) );
    return _S4;
}

