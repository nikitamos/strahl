use std::num::NonZero;

use glam::Vec2;
use wgpu::{BlendState, util::DeviceExt};
use zerocopy::IntoBytes;

use crate::limne::{
  Blur, GaussianBlur, RenderTarget, TextureDrawer, TextureDrawerResources, TextureProvider,
};

pub struct PostProcessInfo<'a> {
  pub viewport:   glam::Vec2,
  pub uniform:    &'a crate::uniform::GlobalUniformsWrapper,
  pub dimensions: u32,
}

pub struct PostProcessCreateInfoBase<'a> {
  pub target_dim:     u32,
  pub texture_format: wgpu::TextureFormat,
  pub depth_format:   wgpu::TextureFormat,
  pub device:         wgpu::Device,
  pub uniform:        &'a crate::uniform::GlobalUniformsWrapper,
}

pub trait PostProcessStep: Send {
  type CreateInfo
  where Self: Sized;

  fn create(ci: Self::CreateInfo, base_info: PostProcessCreateInfoBase) -> Self
  where Self: Sized;
  #[must_use]
  fn apply(
    &self,
    origin: &wgpu::TextureView,
    target: &wgpu::TextureView,
    info: PostProcessInfo,
  ) -> wgpu::CommandBuffer;
}

pub struct BloomPostProcess {
  drawer:               TextureDrawer,
  device:               wgpu::Device,
  kernel_buffer:        wgpu::Buffer,
  kernel_group:         wgpu::BindGroup,
  bright_regions:       TextureProvider,
  brightness_extractor: TextureDrawer,
}

pub struct BloomCreateInfo {
  pub kernel_depth: u32,
  pub s:            f32,
}

impl PostProcessStep for BloomPostProcess {
  type CreateInfo = BloomCreateInfo;

  fn create(ci: Self::CreateInfo, base_info: PostProcessCreateInfoBase) -> Self
  where Self: Sized {
    let blur = GaussianBlur {
      s:    ci.s,
      side: ci.kernel_depth as usize,
      dh:   Vec2::ONE, // TODO
    };
    let shader = base_info
      .device
      .create_shader_module(wgpu::include_wgsl!("../shaders/blur.wgsl"));
    let (kernel_buffer, kernel_group, blur) = prepare_blur(&base_info, blur, &shader);
    let bright_regions =
      TextureProvider::new(&base_info.device, crate::limne::TextureProviderDescriptor {
        label:           Some("birght regions".to_string()),
        size:            wgpu::Extent3d {
          width:                 base_info.target_dim,
          height:                base_info.target_dim,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count:    1,
        dimension:       wgpu::TextureDimension::D2,
        format:          base_info.texture_format,
        usage:           wgpu::TextureUsages::RENDER_ATTACHMENT
          | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats:    vec![],
      });
    let brightness_extractor = TextureDrawer::new(
      &base_info.device,
      &bright_regions.format(),
      crate::limne::TextureDrawerInitRes {
        fragment: Some(wgpu::FragmentState {
          module:              &shader,
          entry_point:         Some("bright"),
          compilation_options: Default::default(),
          targets:             &[Some(wgpu::ColorTargetState {
            format:     bright_regions.format(),
            blend:      Some(BlendState::REPLACE),
            write_mask: Default::default(),
          })],
        }),
        ..Default::default()
      },
    );
    Self {
      drawer: blur,
      device: base_info.device,
      kernel_buffer,
      kernel_group,
      bright_regions,
      brightness_extractor,
    }
  }
  fn apply(
    &self,
    origin: &wgpu::TextureView,
    target: &wgpu::TextureView,
    info: PostProcessInfo,
  ) -> wgpu::CommandBuffer {
    log::trace!("applying bloom");
    let mut encoder = self
      .device
      .create_command_encoder(&wgpu::wgt::CommandEncoderDescriptor {
        label: Some("bloom encoder"),
      });

    // 1. Extract bright regions
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("bloom brightness"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view:           &self.bright_regions,
        depth_slice:    None,
        resolve_target: None,
        ops:            wgpu::Operations {
          load:  wgpu::LoadOp::Clear(wgpu::Color::BLACK),
          store: wgpu::StoreOp::Store,
        },
      })],
      ..Default::default()
    });
    self
      .brightness_extractor
      .render_into_pass(&mut pass, &TextureDrawerResources {
        src:         origin,
        bind_groups: &[],
        device:      &self.device,
      });
    drop(pass);
    encoder.copy_texture_to_texture(
      wgpu::TexelCopyTextureInfoBase {
        texture:   origin.texture(),
        mip_level: 0,
        origin:    wgpu::Origin3d::ZERO,
        aspect:    wgpu::TextureAspect::All,
      },
      wgpu::TexelCopyTextureInfoBase {
        texture:   target.texture(),
        mip_level: 0,
        origin:    wgpu::Origin3d::ZERO,
        aspect:    wgpu::TextureAspect::All,
      },
      wgpu::Extent3d {
        width:                 info.dimensions,
        height:                info.dimensions,
        depth_or_array_layers: 1,
      },
    );

    // 2. Apply blur
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("bloom pass"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view:           target,
        depth_slice:    None,
        resolve_target: None,
        ops:            wgpu::Operations {
          load:  wgpu::LoadOp::Load,
          store: wgpu::StoreOp::Store,
        },
      })],
      depth_stencil_attachment: None,
      timestamp_writes: None,
      occlusion_query_set: None,
      multiview_mask: None,
    });
    // pass.set_viewport(0.0, 0.0, info.viewport.x, info.viewport.y, 0.0, 1.0);
    self
      .drawer
      .render_into_pass(&mut pass, &TextureDrawerResources {
        src:         &self.bright_regions,
        bind_groups: &[&self.kernel_group, info.uniform.bind_group()],
        device:      &self.device,
      });

    drop(pass);

    encoder.finish()
  }
}

fn prepare_blur(
  base_info: &PostProcessCreateInfoBase<'_>,
  blur: GaussianBlur,
  shader: &wgpu::ShaderModule,
) -> (wgpu::Buffer, wgpu::BindGroup, TextureDrawer) {
  let kernel = blur.full_kernel();
  let device = &base_info.device;
  let kernel_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label:    Some("bloom kernel"),
    contents: kernel.as_bytes(), // TODO: does it do what I think
    usage:    wgpu::BufferUsages::STORAGE,
  });
  let kernel_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label:   Some("bloom kernel"),
    entries: &[wgpu::BindGroupLayoutEntry {
      binding:    0,
      visibility: wgpu::ShaderStages::FRAGMENT,
      ty:         wgpu::BindingType::Buffer {
        ty:                 wgpu::BufferBindingType::Storage { read_only: true },
        has_dynamic_offset: false,
        min_binding_size:   NonZero::new((kernel.len() * std::mem::size_of::<f32>()) as u64),
      },
      count:      None,
    }],
  });
  let kernel_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
    label:   Some("bloom kernel bg"),
    layout:  &kernel_layout,
    entries: &[wgpu::BindGroupEntry {
      binding:  0,
      resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
        buffer: &kernel_buffer,
        offset: 0,
        size:   None,
      }),
    }],
  });

  let drawer = TextureDrawer::new(
    &base_info.device,
    &base_info.texture_format,
    crate::limne::TextureDrawerInitRes {
      stencil:         None,
      fragment:        Some(wgpu::FragmentState {
        module:              shader,
        entry_point:         Some("blur"),
        compilation_options: Default::default(),
        targets:             &[Some(wgpu::ColorTargetState {
          format:     base_info.texture_format,
          blend:      Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
              src_factor: wgpu::BlendFactor::One,
              dst_factor: wgpu::BlendFactor::One,
              operation:  wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::REPLACE,
          }),
          write_mask: Default::default(),
        })],
      }),
      layout:          &[kernel_layout, base_info.uniform.bind_group_layout().clone()],
      unclipped_depth: false,
      blend:           None,
    },
  );
  (kernel_buffer, kernel_group, drawer)
}
