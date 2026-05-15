@binding(0) @group(0) var unsmoothed_0 : texture_2d<f32>;

@binding(1) @group(0) var smp_0 : sampler;

fn AcesApprox_0( v_0 : vec3<f32>) -> vec3<f32>
{
    var _S1 : vec3<f32> = v_0 * vec3<f32>(0.60000002384185791f);
    return clamp(_S1 * (vec3<f32>(2.50999999046325684f) * _S1 + vec3<f32>(0.02999999932944775f)) / (_S1 * (vec3<f32>(2.43000006675720215f) * _S1 + vec3<f32>(0.5899999737739563f)) + vec3<f32>(0.14000000059604645f)), vec3<f32>(0.0f), vec3<f32>(1.0f));
}

struct pixelOutput_0
{
    @location(0) output_0 : vec4<f32>,
};

struct pixelInput_0
{
    @location(0) texcoord_0 : vec4<f32>,
};

@fragment
fn fs_main( _S2 : pixelInput_0, @builtin(position) clip_pos_0 : vec4<f32>) -> pixelOutput_0
{
    var _S3 : vec4<f32> = (textureSample((unsmoothed_0), (smp_0), (_S2.texcoord_0.xy)));
    var _S4 : pixelOutput_0 = pixelOutput_0( vec4<f32>(AcesApprox_0(_S3.xyz), _S3.w) );
    return _S4;
}

