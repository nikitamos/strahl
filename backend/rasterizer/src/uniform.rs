use std::num::NonZero;

use wgpu::BindGroupDescriptor;
use zerocopy::IntoBytes;

#[derive(zerocopy::KnownLayout, zerocopy::IntoBytes, zerocopy::Immutable, Default)]
#[repr(C)]
pub struct GlobalUniforms {
  pub camera: glam::Mat4,
}

pub struct GlobalUniformsWrapper {
  pub uniform: GlobalUniforms,
  buffer:      wgpu::Buffer,
  bind_group:  wgpu::BindGroup,
  bg_layout:   wgpu::BindGroupLayout,
}

impl std::ops::DerefMut for GlobalUniformsWrapper {
  fn deref_mut(&mut self) -> &mut Self::Target { &mut self.uniform }
}

impl std::ops::Deref for GlobalUniformsWrapper {
  type Target = GlobalUniforms;

  fn deref(&self) -> &Self::Target { &self.uniform }
}

impl GlobalUniformsWrapper {
  const ENTRIES: [wgpu::BindGroupLayoutEntry; 1] = [wgpu::BindGroupLayoutEntry {
    binding:    0,
    visibility: wgpu::ShaderStages::all(),
    ty:         wgpu::BindingType::Buffer {
      ty:                 wgpu::BufferBindingType::Uniform,
      has_dynamic_offset: false,
      min_binding_size:   NonZero::new(std::mem::size_of::<GlobalUniforms>() as u64),
    },
    count:      None,
  }];

  pub fn new(device: &wgpu::Device) -> Self {
    let bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label:   Some("Uniforms BG layout"),
      entries: &Self::ENTRIES,
    });

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label:              Some("Uniform buffer"),
      size:               std::mem::size_of::<GlobalUniforms>() as u64,
      usage:              wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });

    let bind_group = device.create_bind_group(&BindGroupDescriptor {
      label:   Some("Uniforms BG"),
      layout:  &bg_layout,
      entries: &[wgpu::BindGroupEntry {
        binding:  0,
        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
          buffer: &buffer,
          offset: 0,
          size:   None,
        }),
      }],
    });
    Self {
      uniform: GlobalUniforms::default(),
      buffer,
      bind_group,
      bg_layout,
    }
  }
  pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout { &self.bg_layout }
  pub fn bind_group(&self) -> &wgpu::BindGroup { &self.bind_group }
  pub fn upload(&self, queue: &wgpu::Queue) {
    queue.write_buffer(&self.buffer, 0, self.uniform.as_bytes());
  }
}
