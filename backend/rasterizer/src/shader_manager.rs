use crate::{geometry::Geometry, skybox::Skybox, uniform::GlobalUniformsWrapper};
use std::sync::{Arc, RwLock};
use wgpu::include_wgsl;

pub(crate) struct ShaderEntryPoint {
  pub module:      wgpu::ShaderModule,
  pub entry_point: Option<String>,
}

pub(crate) struct ShaderManager {
  mesh_vert:    ShaderEntryPoint,
  pbr_frag:     ShaderEntryPoint,
  skybox_vert:  ShaderEntryPoint,
  skybox_frag:  ShaderEntryPoint,
  dev:          wgpu::Device,
  uniforms:     RwLock<GlobalUniformsWrapper>,
  depth_format: wgpu::TextureFormat,
}

impl ShaderManager {
  const EMPTY_ATTRIBUTES: [wgpu::VertexAttribute; 3] = [wgpu::VertexAttribute {
    format:          wgpu::VertexFormat::Float16,
    offset:          0,
    shader_location: 0,
  }; 3];
  const EMPTY_BUFFER_LAYOUT: [wgpu::VertexBufferLayout<'_>; 3] = [
    wgpu::VertexBufferLayout {
      array_stride: 0,
      step_mode:    wgpu::VertexStepMode::Instance,
      attributes:   &[],
    },
    wgpu::VertexBufferLayout {
      array_stride: 0,
      step_mode:    wgpu::VertexStepMode::Instance,
      attributes:   &[],
    },
    wgpu::VertexBufferLayout {
      array_stride: 0,
      step_mode:    wgpu::VertexStepMode::Instance,
      attributes:   &[],
    },
  ];
  pub fn new(dev: wgpu::Device, depth_format: wgpu::TextureFormat) -> Self {
    let raster = dev.create_shader_module(include_wgsl!("../shaders/raster-pipeline.wgsl"));
    let skybox = dev.create_shader_module(include_wgsl!("../shaders/skybox.wgsl"));
    let mesh_vert = ShaderEntryPoint {
      module:      raster.clone(),
      entry_point: Some("MeshGeometryVS".to_string()),
    };
    let pbr_frag = ShaderEntryPoint {
      module:      raster,
      entry_point: Some("RasterizerPbrFS".to_string()),
    };
    let skybox_frag = ShaderEntryPoint {
      module:      skybox.clone(),
      entry_point: Some("SkyboxFragment".to_string()),
    };
    let skybox_vert = ShaderEntryPoint {
      module:      skybox,
      entry_point: Some("SkyboxVertex".to_string()),
    };
    Self {
      uniforms: RwLock::new(GlobalUniformsWrapper::new(&dev)),
      mesh_vert,
      pbr_frag,
      dev,
      depth_format,
      skybox_frag,
      skybox_vert,
    }
  }
  pub fn mesh_vertex(&self) -> &ShaderEntryPoint { &self.mesh_vert }
  pub fn pbr_fragment(&self) -> &ShaderEntryPoint { &self.pbr_frag }
  pub fn create_pipeline_for_mesh_geometry(
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
          Some(self.uniforms.read().unwrap().bind_group_layout()),
          Some(material.bind_group_layout()),
          geometry.bind_group_layout(),
        ],
        immediate_size:     256,
      });

    let mut attrs: [wgpu::VertexAttribute; 3] = Self::EMPTY_ATTRIBUTES;
    let mut attr_layout: [wgpu::VertexBufferLayout<'_>; 3] = Self::EMPTY_BUFFER_LAYOUT;

    let desc = wgpu::RenderPipelineDescriptor {
      label:          Some("material"),
      layout:         Some(&layout),
      vertex:         geometry.vertex_state(&mut attrs, &mut attr_layout, self),
      primitive:      wgpu::PrimitiveState {
        topology:           wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face:         wgpu::FrontFace::Ccw,
        cull_mode:          Some(wgpu::Face::Back),
        unclipped_depth:    false,
        polygon_mode:       wgpu::PolygonMode::Fill,
        conservative:       false,
      },
      depth_stencil:  Some(wgpu::DepthStencilState {
        format:              self.depth_format,
        depth_write_enabled: Some(true),
        depth_compare:       Some(wgpu::CompareFunction::Less),
        stencil:             wgpu::StencilState::default(),
        bias:                wgpu::DepthBiasState::default(),
      }),
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

  pub fn create_pipeline_for_skybox(&self, skybox: &Skybox) -> wgpu::RenderPipeline {
    let layout = self
      .dev
      .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label:              Some("skybox pl"),
        bind_group_layouts: &[
          Some(self.uniforms.read().unwrap().bind_group_layout()),
          Some(skybox.bind_group_layout()),
        ],
        immediate_size:     0,
      });
    self
      .dev
      .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label:          Some("Skybox pipeline"),
        layout:         Some(&layout),
        vertex:         skybox.vertex_state(&self.skybox_vert),
        primitive:      wgpu::PrimitiveState {
          topology:           wgpu::PrimitiveTopology::TriangleStrip,
          strip_index_format: None,
          front_face:         wgpu::FrontFace::Ccw,
          cull_mode:          None,
          unclipped_depth:    false,
          polygon_mode:       wgpu::PolygonMode::Fill,
          conservative:       false,
        },
        depth_stencil:  Some(wgpu::DepthStencilState {
          format:              self.depth_format,
          depth_write_enabled: Some(true),
          depth_compare:       Some(wgpu::CompareFunction::LessEqual),
          stencil:             wgpu::StencilState::default(),
          bias:                wgpu::DepthBiasState::default(),
        }),
        multisample:    wgpu::MultisampleState::default(),
        fragment:       Some(skybox.fragment_state(&self.skybox_frag)),
        multiview_mask: None,
        cache:          None,
      })
  }

  pub(crate) fn uniforms(&self) -> std::sync::RwLockReadGuard<'_, GlobalUniformsWrapper> {
    self.uniforms.read().unwrap()
  }
  pub(crate) fn uniforms_mut(&self) -> std::sync::RwLockWriteGuard<'_, GlobalUniformsWrapper> {
    self.uniforms.write().unwrap()
  }
}
