module globals;

public struct Globals {
    public row_major float4x4 projection;
    public row_major float4x4 camera;
    public uint32_t2 viewport_size;
};

[vk::binding(0,0)]
public ConstantBuffer<Globals> global;