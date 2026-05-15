use std::{cell::RefCell, num::NonZero};

use glam::Vec2;
use wgpu::{BlendState, util::DeviceExt};
use zerocopy::{Immutable, IntoBytes};

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
  device:               wgpu::Device,
  #[deprecated]
  kernel_buffer:        wgpu::Buffer,
  kernel_layout:        wgpu::BindGroupLayout,
  kernel_group:         RefCell<Option<wgpu::BindGroup>>,
  bright_regions:       TextureProvider,
  brightness_extractor: TextureDrawer,
  blur_horizontal:      TextureDrawer,
  blur_vertical:        TextureDrawer,
  temp_texture:         TextureProvider,
}

pub struct BloomCreateInfo {
  pub kernel_depth: u32,
  pub s:            f32,
}

#[repr(C)]
#[derive(Clone, Copy, IntoBytes, Immutable)]
struct PushConstants {
  horizontal: u32,
}

impl PostProcessStep for BloomPostProcess {
  type CreateInfo = BloomCreateInfo;

  fn create(ci: Self::CreateInfo, base_info: PostProcessCreateInfoBase) -> Self
  where Self: Sized {
    let blur = GaussianBlur {
      s:    ci.s,
      side: ci.kernel_depth as usize,
      dh:   Vec2::ONE,
    };
    let shader = base_info
      .device
      .create_shader_module(wgpu::include_wgsl!("../shaders/blur.wgsl"));
    let (kernel_buffer, kernel_layout, blur_horizontal, blur_vertical) =
      prepare_blur(&base_info, blur, &shader);

    let bright_regions =
      TextureProvider::new(&base_info.device, crate::limne::TextureProviderDescriptor {
        label:           Some("bright regions".to_string()),
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

    // Temporary texture for intermediate blur pass
    let temp_texture =
      TextureProvider::new(&base_info.device, crate::limne::TextureProviderDescriptor {
        label:           Some("bloom temp".to_string()),
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
      device: base_info.device,
      kernel_buffer,
      kernel_layout,
      kernel_group: RefCell::new(None),
      bright_regions,
      brightness_extractor,
      blur_horizontal,
      blur_vertical,
      temp_texture,
    }
  }

  fn apply(
    &self,
    origin: &wgpu::TextureView,
    target: &wgpu::TextureView,
    info: PostProcessInfo,
  ) -> wgpu::CommandBuffer {
    log::trace!("applying bloom");
    self.ensure_kernel_group_created(origin);

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
        immediates: &[]
      });
    drop(pass);

    let kernel_group = self.kernel_group.borrow();
    let bind_groups = [kernel_group.as_ref().unwrap(), info.uniform.bind_group()];

    // 2. Horizontal blur pass (bright_regions -> temp_texture)

    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("bloom horizontal blur"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view:           &self.temp_texture,
        depth_slice:    None,
        resolve_target: None,
        ops:            wgpu::Operations {
          load:  wgpu::LoadOp::Clear(wgpu::Color::BLACK),
          store: wgpu::StoreOp::Store,
        },
      })],
      depth_stencil_attachment: None,
      timestamp_writes: None,
      occlusion_query_set: None,
      multiview_mask: None,
    });

    self
      .blur_horizontal
      .render_into_pass(&mut pass, &TextureDrawerResources {
        src:         &self.bright_regions,
        bind_groups: &bind_groups,
        device:      &self.device,
        immediates:  PushConstants { horizontal: 1 }.as_bytes(),
      });
    drop(pass);

    // 3. Vertical blur pass (temp_texture -> target)
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("bloom vertical blur"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view:           target,
        depth_slice:    None,
        resolve_target: None,
        ops:            wgpu::Operations {
          load:  wgpu::LoadOp::Clear(wgpu::Color::WHITE),
          store: wgpu::StoreOp::Store,
        },
      })],
      depth_stencil_attachment: None,
      timestamp_writes: None,
      occlusion_query_set: None,
      multiview_mask: None,
    });

    self
      .blur_vertical
      .render_into_pass(&mut pass, &TextureDrawerResources {
        src:         &self.temp_texture,
        bind_groups: &bind_groups,
        device:      &self.device,
        immediates:  PushConstants { horizontal: 0 }.as_bytes(),
      });

    drop(pass);

    encoder.finish()
  }
}

impl BloomPostProcess {
  fn ensure_kernel_group_created(&self, origin: &wgpu::TextureView) {
    let mut group = self.kernel_group.borrow_mut();
    if group.is_none() {
      *group = Some(kernel_group(
        origin,
        &self.device,
        &self.kernel_buffer,
        &self.kernel_layout,
      ));
    }
  }
}

fn prepare_blur<B: Blur>(
  base_info: &PostProcessCreateInfoBase<'_>,
  blur: B,
  shader: &wgpu::ShaderModule,
) -> (
  wgpu::Buffer,
  wgpu::BindGroupLayout,
  TextureDrawer,
  TextureDrawer,
) {
  let kernel = blur.full_kernel();
  let device = &base_info.device;
  let kernel_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label:    Some("bloom kernel"),
    contents: kernel.as_bytes(),
    usage:    wgpu::BufferUsages::STORAGE,
  });

  // Bind group layout for kernel and origin texture
  let kernel_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label:   Some("bloom kernel"),
    entries: &[
      wgpu::BindGroupLayoutEntry {
        binding:    0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty:         wgpu::BindingType::Texture {
          sample_type:    wgpu::TextureSampleType::Float { filterable: true },
          view_dimension: wgpu::TextureViewDimension::D2,
          multisampled:   false,
        },
        count:      None,
      },
    ],
  });

  // Bind group layouts for the blur passes
  let bind_group_layouts = [
    kernel_layout.clone(),
    base_info.uniform.bind_group_layout().clone(),
  ];

  let blur_horizontal = TextureDrawer::new(
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
      layout:          &bind_group_layouts, // Pass the bind group layouts
      unclipped_depth: false,
      blend:           None,
      immediate_size:  std::mem::size_of::<PushConstants>() as u32, // Size of push constants
    },
  );

  let blur_vertical = TextureDrawer::new(
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
          blend:      Some(wgpu::BlendState::REPLACE),
          write_mask: Default::default(),
        })],
      }),
      layout:          &bind_group_layouts, // Pass the bind group layouts
      unclipped_depth: false,
      blend:           None,
      immediate_size:  std::mem::size_of::<PushConstants>() as u32, // Size of push constants
    },
  );

  (kernel_buffer, kernel_layout, blur_horizontal, blur_vertical)
}

fn kernel_group(
  origin: &wgpu::TextureView,
  device: &wgpu::Device,
  kernel_buffer: &wgpu::Buffer,
  kernel_layout: &wgpu::BindGroupLayout,
) -> wgpu::BindGroup {
  device.create_bind_group(&wgpu::BindGroupDescriptor {
    label:   Some("bloom kernel bg"),
    layout:  kernel_layout,
    entries: &[
      wgpu::BindGroupEntry {
        binding:  0,
        resource: wgpu::BindingResource::TextureView(origin),
      },
    ],
  })
}
