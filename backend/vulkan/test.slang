ConstantBuffer<int[8]> in;
RWStructuredBuffer<int> out;

[shader("compute")]
void some_calc(uint3 id: SV_DispatchThreadID) {
    out[id.x] = in[id.x];
}


