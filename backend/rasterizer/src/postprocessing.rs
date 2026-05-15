use std::num::NonZero;

use glam::Vec2;
use wgpu::util::DeviceExt;
use zerocopy::IntoBytes;

use crate::limne::{Blur, GaussianBlur, RenderTarget, TextureDrawer, TextureDrawerResources};

pub struct PostProcessInfo<'a> {
  pub viewport: glam::Vec2,
  pub uniform:  &'a crate::uniform::GlobalUniformsWrapper,
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
  drawer:        TextureDrawer,
  device:        wgpu::Device,
  kernel_buffer: wgpu::Buffer,
  kernel_group:  wgpu::BindGroup,
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
          module:              &device
            .create_shader_module(wgpu::include_wgsl!("../shaders/blur.wgsl")),
          entry_point:         None,
          compilation_options: Default::default(),
          targets:             &[Some(wgpu::ColorTargetState {
            format:     base_info.texture_format,
            blend:      None,
            write_mask: Default::default(),
          })],
        }),
        layout:          &[kernel_layout, base_info.uniform.bind_group_layout().clone()],
        unclipped_depth: false,
        blend:           None,
      },
    );
    Self {
      drawer,
      device: base_info.device,
      kernel_buffer,
      kernel_group,
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

    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("bloom pass"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view:           target,
        depth_slice:    None,
        resolve_target: None,
        ops:            wgpu::Operations {
          load:  wgpu::LoadOp::DontCare(Default::default()),
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
        src:         origin,
        bind_groups: &[&self.kernel_group, info.uniform.bind_group()],
        device:      &self.device,
      });

    drop(pass);

    encoder.finish()
  }
}
