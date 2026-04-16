#[derive(Clone)]
pub enum Geometry {
  Mesh {
    buf:   wgpu::Buffer,
    group: wgpu::BindGroup,
  },
}