module globals;

public struct Globals {
    public float4x4 camera;
};

[vk::binding(0,0)]
public ConstantBuffer<Globals> global;