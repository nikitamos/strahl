use strahl_import::reader::{GltfBufferView, GltfGeometry};
use wgpu::{
  PipelineCompilationOptions, VertexBufferLayout,
  util::{BufferInitDescriptor, DeviceExt},
};

use crate::shader_manager::ShaderManager;

// #[derive(Clone)]
#[non_exhaustive]
pub enum Geometry {
  #[non_exhaustive]
  Mesh {
    buf:  wgpu::Buffer,
    gltf: GltfGeometry,
  },
}

impl Geometry {
  pub fn from_gltf(dev: &wgpu::Device, gltf: GltfGeometry) -> anyhow::Result<Self> {
    log::trace!("Creating geometry from glTF");
    get_gltf_index_format(&gltf).ok_or_else(|| {
      log::error!(
        "bad index format in glTF: expected 2 or 4-byte indices, got {}",
        gltf.index_size
      );
      anyhow::anyhow!("bad index format: expected 2 or 4-byte indices")
    })?;
    let buf = dev.create_buffer_init(&BufferInitDescriptor {
      label:    None,
      contents: &gltf.buffer,
      usage:    wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX,
    });

    Ok(Self::Mesh { buf, gltf })
  }

  /// Sets vector attributes for rendering the mesh in given `RenderPass`
  pub fn setup_attributes(&self, pass: &mut wgpu::RenderPass) {
    match self {
      Self::Mesh { buf, gltf, .. } => {
        pass.set_index_buffer(
          buf.slice(&gltf.indices),
          get_gltf_index_format(gltf).unwrap(),
        );
        pass.set_vertex_buffer(0, buf.slice(&gltf.position));
        pass.set_vertex_buffer(1, buf.slice(&gltf.normals));
        pass.set_vertex_buffer(2, buf.slice(&gltf.uv));
      }
    }
  }

  pub fn dispatch_draw(&self, pass: &mut wgpu::RenderPass) {
    if let Self::Mesh { gltf, .. } = self {
      pass.draw_indexed(0..(gltf.indices.count as u32), 0, 0..1);
    } else {
      unreachable!()
    }
  }

  pub(crate) fn vertex_state<'b>(
    &self,
    attrs: &'b mut [wgpu::VertexAttribute; 3],
    layout: &'b mut [wgpu::VertexBufferLayout<'b>; 3],
    mgr: &'b ShaderManager,
  ) -> wgpu::VertexState<'b> {
    if let Self::Mesh { buf: _buf, gltf } = self {
      *layout = Self::buffer_layout(gltf, attrs);
      let shader = mgr.mesh_vertex();
      wgpu::VertexState {
        module:              &shader.module,
        entry_point:         shader.entry_point.as_deref(),
        compilation_options: PipelineCompilationOptions {
          constants: &[],
          zero_initialize_workgroup_memory: false,
        },
        buffers:             layout,
      }
    } else {
      unreachable!()
    }
  }

  pub fn buffer_layout<'a>(
    gltf: &GltfGeometry,
    attrs: &'a mut [wgpu::VertexAttribute; 3],
  ) -> [wgpu::VertexBufferLayout<'a>; 3] {
    let (first, cons) = attrs.split_at_mut(1);
    let (second, third) = cons.split_at_mut(1);
    [
      Self::layout_for(&gltf.position, 0, wgpu::VertexFormat::Float32x3, first),
      Self::layout_for(&gltf.normals, 1, wgpu::VertexFormat::Float32x3, second),
      Self::layout_for(&gltf.uv, 2, wgpu::VertexFormat::Float32x2, third),
    ]
  }
  fn layout_for<'a>(
    gltf_buf: &GltfBufferView,
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

  pub fn bind_group_layout(&self) -> Option<&wgpu::BindGroupLayout> { None }
  pub fn bind_group(&self) -> Option<&wgpu::BindGroup> { None }
}

fn get_gltf_index_format(gltf: &GltfGeometry) -> Option<wgpu::IndexFormat> {
  match gltf.index_size {
    2 => Some(wgpu::IndexFormat::Uint16),
    4 => Some(wgpu::IndexFormat::Uint32),
    _ => None,
  }
}
