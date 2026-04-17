use crate::{geometry::Geometry, uniform::GlobalUniformsWrapper};
use std::{mem::MaybeUninit, sync::Arc};
use wgpu::include_wgsl;

pub(crate) struct ShaderEntryPoint {
  pub module:      Arc<wgpu::ShaderModule>,
  pub entry_point: Option<String>,
}

pub(crate) struct ShaderManager {
  mesh_vert: ShaderEntryPoint,
  pbr_frag:  ShaderEntryPoint,
  dev:       wgpu::Device,
  uniforms:  GlobalUniformsWrapper,
}

impl ShaderManager {
  pub fn new(dev: wgpu::Device) -> Self {
    let mesh_vert = ShaderEntryPoint {
      module:      Arc::new(
        dev.create_shader_module(include_wgsl!("../shaders/raster-pipeline.wgsl")),
      ),
      entry_point: Some("MeshGeometryVS".to_string()),
    };
    let pbr_frag = ShaderEntryPoint {
      module:      Arc::new(
        dev.create_shader_module(include_wgsl!("../shaders/raster-pipeline.wgsl")),
      ),
      entry_point: Some("RasterizerPbrFS".to_string()),
    };
    Self {
      uniforms: GlobalUniformsWrapper::new(&dev),
      mesh_vert,
      pbr_frag,
      dev,
    }
  }
  pub fn mesh_vertex(&self) -> &ShaderEntryPoint { &self.mesh_vert }
  pub fn pbr_fragment(&self) -> &ShaderEntryPoint { &self.pbr_frag }
  pub fn create_pipeline_for_mesh_geometry<'a>(
    &self,
    material: &crate::material::Material,
    geometry: &Geometry,
  ) -> wgpu::RenderPipeline {
    // We have the following layout:
    // (0) -> global (camera, time?) (or should it be a push constant?)
    // (1) -> material-specific
    // (2) -> geometry-specific
    // (vertex attributes) -> geometry-specific

    // (push consts) -> body-specific (transforms, etc.)

    let layout = self
      .dev
      .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label:              Some("material"),
        bind_group_layouts: &[
          Some(self.uniforms.bind_group_layout()),
          Some(material.bind_group_layout()),
          geometry.bind_group_layout(),
        ],
        immediate_size:     256,
      });

    let mut attrs: [wgpu::VertexAttribute; 3] = unsafe { MaybeUninit::uninit().assume_init() };
    let mut attr_layout: [wgpu::VertexBufferLayout<'_>; 3] =
      unsafe { MaybeUninit::uninit().assume_init() };

    let desc = wgpu::RenderPipelineDescriptor {
      label:          Some("material"),
      layout:         Some(&layout),
      vertex:         geometry.vertex_state(&mut attrs, &mut attr_layout, self),
      primitive:      wgpu::PrimitiveState {
        topology:           wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face:         wgpu::FrontFace::Cw, // TODO
        cull_mode:          None,
        unclipped_depth:    false,
        polygon_mode:       wgpu::PolygonMode::Fill,
        conservative:       false,
      },
      depth_stencil:  None,
      multisample:    wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      fragment:       Some(wgpu::FragmentState {
        module:              &self.pbr_fragment().module,
        entry_point:         self.pbr_fragment().entry_point.as_deref(),
        compilation_options: wgpu::PipelineCompilationOptions {
          constants: &[("0", material.color() as f64)],
          zero_initialize_workgroup_memory: false,
        },
        targets:             &[Some(wgpu::ColorTargetState {
          format:     wgpu::TextureFormat::Rgba8Unorm,
          blend:      None,
          write_mask: wgpu::ColorWrites::all(),
        })],
      }),
      multiview_mask: None,
      cache:          None,
    };
    // TODO: caching
    self.dev.create_render_pipeline(&desc)
  }

  pub(crate) fn uniforms(&self) -> &GlobalUniformsWrapper { &self.uniforms }
  pub(crate) fn uniforms_mut(&mut self) -> &mut GlobalUniformsWrapper { &mut self.uniforms }
}
