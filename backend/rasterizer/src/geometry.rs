use std::{mem::MaybeUninit, sync::Arc};

use strahl_import::reader::{GltfBufferView, GltfGeometry};
use wgpu::{
  PipelineCompilationOptions, VertexBufferLayout,
  util::{BufferInitDescriptor, DeviceExt},
  vertex_attr_array,
};

// #[derive(Clone)]
#[non_exhaustive]
pub enum Geometry {
  #[non_exhaustive]
  Mesh {
    buf:        wgpu::Buffer,
    gltf:       GltfGeometry,
    attributes: [wgpu::VertexAttribute; 3],
  },
}

impl Geometry {
  pub fn from_gltf(dev: &wgpu::Device, queue: &wgpu::Queue, gltf: GltfGeometry) -> Self {
    let buf = dev.create_buffer_init(&BufferInitDescriptor {
      label:    None,
      contents: &gltf.buffer,
      usage:    wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX,
    });

    let res = Self::Mesh {
      buf,
      gltf,
      attributes: unsafe { MaybeUninit::uninit().assume_init() },
    };
    res
  }

  pub fn setup_attributes(&self, pass: &mut wgpu::RenderPass) {
    match self {
      Self::Mesh { buf, gltf, .. } => {
        pass.set_index_buffer(buf.slice(gltf.indices), todo!());
        pass.set_vertex_buffer(0, buf.slice(gltf));
        pass.set_vertex_buffer(1, buf.slice(gltf));
        pass.set_vertex_buffer(2, buf.slice(gltf));
      }
    }
  }
  pub fn vertex_state(&self) -> wgpu::VertexState<'_> {
    wgpu::VertexState {
      module:              todo!(),
      entry_point:         todo!(),
      compilation_options: PipelineCompilationOptions {
        constants: &[],
        zero_initialize_workgroup_memory: false,
      },
      buffers:             &self.buffer_layout(),
    }
  }
  pub fn buffer_layout<'a>(&'a mut self) -> [wgpu::VertexBufferLayout<'a>; 3] {
    let Geometry::Mesh {
      buf: _buf,
      gltf,
      attributes: attrs,
    } = self;
    let (first, cons) = attrs.split_at_mut(1);
    let (second, third) = cons.split_at_mut(1);
    [
      Self::layout_for(&gltf.position, 0, wgpu::VertexFormat::Float32x3, first),
      Self::layout_for(&gltf.normals, 1, wgpu::VertexFormat::Float32x3, second),
      Self::layout_for(&gltf.uv, 2, wgpu::VertexFormat::Float32x2, third),
    ]
  }
  fn layout_for<'a>(
    gltf_buf: &'a GltfBufferView,
    loc: wgpu::ShaderLocation,
    fmt: wgpu::VertexFormat,
    attrs: &'a mut [wgpu::VertexAttribute],
  ) -> wgpu::VertexBufferLayout<'a> {
    attrs[0] = wgpu::VertexAttribute {
      format:          fmt,
      offset:          0,
      shader_location: loc,
    };
    VertexBufferLayout {
      array_stride: gltf_buf.stride as wgpu::BufferAddress,
      step_mode:    wgpu::VertexStepMode::Vertex,
      attributes:   attrs,
    }
  }
}

fn get_gltf_index_format(gltf: &GltfGeometry) -> Option<wgpu::IndexFormat> {
  match gltf.index_size {
    16 => Some(wgpu::IndexFormat::Uint16),
    32 => Some(wgpu::IndexFormat::Uint32),
    _ => None,
  }
}
