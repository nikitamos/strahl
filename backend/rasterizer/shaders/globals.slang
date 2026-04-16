module transforms;

struct Transforms {
    float4x4 world_to_body;
};

[vk::binding(0,0)]
ConstantBuffer<Transforms> transforms;