use std::{cell::Cell, pin::Pin, slice};

use crate::{
  gpu_alloc::Allocator,
  limne::{self, RenderTarget},
};

use ash::vk::{self};
use glam::{UVec2, UVec4};
use wgpu::hal::vulkan as wgvk;

pub(crate) trait Presenter: Send {
  // type CreateInfo: Into<Self>
  // where Self: Sized;
  // const IMMEDIATE_PRESENT: bool; // TODO
  fn post_submit(&self) {}

  fn get_wgpu_capabilities(&self) -> (wgpu::Limits, wgpu::Features);
  #[deprecated]
  fn texture_view(&self) -> &wgpu::TextureView;
  #[deprecated]
  fn present_texture(&self) -> &wgpu::Texture;
  fn present(
    &self,
    backbuffer: &wgpu::Texture,
    encoder: &mut wgpu::CommandEncoder,
    tex_dim: UVec2,
    viewport: UVec4,
  ) -> PresentationResult<'_>;

  fn reconfigure(&self, _viewport: UVec2) {}
}

#[derive(Debug)]
pub enum PresentationResult<'a> {
  Mapped(&'a [u8]),
  Stored(&'a wgpu::Texture),
  Submitted,
  Ignored,
  ReconfigurationRequired(Option<&'a wgpu::Texture>),
}

pub(crate) struct MappedPresenter {
  pub(crate) present_texture: wgpu::Texture,
  pub(crate) mapped:          &'static [u8], // this is bad and unsound. must be rewritten somehow
}

pub(crate) struct RawMappedPresenter {
  pub(crate) present_texture: wgvk::Texture,
  pub(crate) wgpu_tex_desc:   wgpu::TextureDescriptor<'static>,
  pub(crate) mapped:          &'static [u8], // this is bad and unsound. must be rewritten somehow
}

impl RawMappedPresenter {
  pub fn from_hal(self, dev: &wgpu::Device) -> MappedPresenter {
    MappedPresenter {
      present_texture: unsafe {
        dev.create_texture_from_hal::<wgvk::Api>(self.present_texture, &self.wgpu_tex_desc)
      },
      mapped:          self.mapped,
    }
  }
}

impl Presenter for MappedPresenter {
  fn get_wgpu_capabilities(&self) -> (wgpu::Limits, wgpu::Features) { Default::default() }

  fn texture_view(&self) -> &wgpu::TextureView { todo!() }

  fn present_texture(&self) -> &wgpu::Texture { &self.present_texture }

  fn present(
    &self,
    backbuffer: &wgpu::Texture,
    encoder: &mut wgpu::CommandEncoder,
    tex_dim: UVec2,
    _viewport: UVec4,
  ) -> PresentationResult<'_> {
    encoder.copy_texture_to_texture(
      wgpu::TexelCopyTextureInfo {
        texture:   backbuffer,
        mip_level: 0,
        origin:    wgpu::Origin3d { x: 0, y: 0, z: 0 },
        aspect:    wgpu::TextureAspect::All,
      },
      wgpu::TexelCopyTextureInfo {
        texture:   self.present_texture(),
        mip_level: 0,
        origin:    wgpu::Origin3d { x: 0, y: 0, z: 0 },
        aspect:    wgpu::TextureAspect::All,
      },
      wgpu::Extent3d {
        width:                 tex_dim.x,
        height:                tex_dim.y,
        depth_or_array_layers: 1,
      },
    );
    PresentationResult::Mapped(self.mapped)
  }
}

// TODO: replace unwraps with proper error handling
pub(crate) async unsafe fn raw_wgpu_setup(
  vk_instance: &wgvk::InstanceShared,
  dq: &wgpu::hal::OpenDevice<wgvk::Api>,
  phy: vk::PhysicalDevice,
  width: u32,
  height: u32,
) -> RawMappedPresenter {
  let extent = vk::Extent3D {
    width,
    height,
    depth: 1,
  };
  let alloc = Allocator::new(phy, dq.device.raw_device(), vk_instance.raw_instance());
  unsafe {
    let img = dq
      .device
      .raw_device()
      .create_image(
        &vk::ImageCreateInfo::default()
          .array_layers(1) // Vulkan implementation must support at least 256 array layers
          .extent(extent)
          .flags(vk::ImageCreateFlags::empty())
          .format(vk::Format::R8G8B8A8_UNORM)
          .image_type(vk::ImageType::TYPE_2D)
          .initial_layout(vk::ImageLayout::UNDEFINED)
          .mip_levels(1)
          .queue_family_indices(&[dq.device.queue_family_index()])
          .samples(vk::SampleCountFlags::TYPE_1) // That's for multisampling, we don't use it (now)
          .sharing_mode(vk::SharingMode::EXCLUSIVE)
          .tiling(vk::ImageTiling::LINEAR) // Linear tiling for predictable memory layout
          .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::COLOR_ATTACHMENT),
        None,
      )
      .unwrap();
    // TODO: proper offset/size calculation
    let reqs = dq.device.raw_device().get_image_memory_requirements(img);
    if width as u64 * height as u64 * 4 < (reqs.size) {
      log::warn!(
        "The driver requires allocation of size {}, while the size of linear texture is {} ({width}x{height})",
        reqs.size,
        width as u64 * height as u64 * 4,
      );
      log::warn!("This may lead to unexpected results");
    }
    log::trace!("Required alignment: {}", reqs.alignment);
    let allocation = alloc
      .allocate(
        reqs.size,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_CACHED,
        reqs.memory_type_bits,
        None::<&mut vk::DedicatedAllocationMemoryAllocateInfoNV>,
      )
      .unwrap(); // TODO: fallback to manual flushing
    let mapped = dq
      .device
      .raw_device()
      .map_memory(allocation, 0, reqs.size, vk::MemoryMapFlags::empty())
      .unwrap()
      .cast();

    let mapped = slice::from_raw_parts(mapped, reqs.size as usize);

    dq.device
      .raw_device()
      .bind_image_memory(img, allocation, 0)
      .unwrap();
    let vk_device = dq.device.raw_device().clone();
    let vk_tex_desc = wgpu::hal::TextureDescriptor {
      label:           Some("framebuffer"),
      size:            wgpu::Extent3d {
        width:                 extent.width,
        height:                extent.height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count:    1,
      dimension:       wgpu::TextureDimension::D2,
      format:          wgpu::TextureFormat::Rgba8Unorm,
      // TODO: Is it initial usage or all allowed usages
      usage:           wgpu::TextureUses::COPY_DST
        | wgpu::TextureUses::UNINITIALIZED
        | wgpu::TextureUses::COLOR_TARGET,
      view_formats:    vec![],
      memory_flags:    wgpu::hal::MemoryFlags::PREFER_COHERENT, // I don't know what it exactly means, but it seems to be right
    };
    let wgpu_tex_desc = wgpu::TextureDescriptor {
      label:           Some("framebuffer"),
      size:            wgpu::Extent3d {
        width:                 extent.width,
        height:                extent.height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count:    1,
      dimension:       wgpu::TextureDimension::D2,
      format:          wgpu::TextureFormat::Rgba8Unorm,
      usage:           wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
      view_formats:    &[],
    };
    let swapchain = dq.device.texture_from_raw(
      img,
      &vk_tex_desc,
      Some(Box::new(move || {
        vk_device.unmap_memory(allocation);
        vk_device.destroy_image(img, None);
        vk_device.free_memory(allocation, None);
      })),
      wgvk::TextureMemory::External,
    );
    RawMappedPresenter {
      present_texture: swapchain,
      wgpu_tex_desc,
      mapped,
    }
  }
}

pub(crate) struct TexturePresenter {
  texture: wgpu::Texture,
}

impl TexturePresenter {
  pub fn new(texture: wgpu::Texture) -> Self { Self { texture } }
}

impl Presenter for TexturePresenter {
  fn get_wgpu_capabilities(&self) -> (wgpu::Limits, wgpu::Features) { Default::default() }

  fn texture_view(&self) -> &wgpu::TextureView { todo!() }

  fn present_texture(&self) -> &wgpu::Texture { &self.texture }

  fn present(
    &self,
    backbuffer: &wgpu::Texture,
    encoder: &mut wgpu::CommandEncoder,
    tex_dim: UVec2,
    _viewport: UVec4,
  ) -> PresentationResult<'_> {
    encoder.copy_texture_to_texture(
      wgpu::TexelCopyTextureInfo {
        texture:   backbuffer,
        mip_level: 0,
        origin:    wgpu::Origin3d { x: 0, y: 0, z: 0 },
        aspect:    wgpu::TextureAspect::All,
      },
      wgpu::TexelCopyTextureInfo {
        texture:   self.present_texture(),
        mip_level: 0,
        origin:    wgpu::Origin3d { x: 0, y: 0, z: 0 },
        aspect:    wgpu::TextureAspect::All,
      },
      wgpu::Extent3d {
        width:                 tex_dim.x,
        height:                tex_dim.y,
        depth_or_array_layers: 1,
      },
    );
    PresentationResult::Stored(&self.texture)
  }
}

pub(crate) struct SurfacePresenter<'window> {
  surface: wgpu::Surface<'window>,
  drawer:  limne::TextureDrawer,
  device:  wgpu::Device,
  texture: Cell<Option<wgpu::SurfaceTexture>>,
}

impl<'w> SurfacePresenter<'w> {
  pub fn new(surface: wgpu::Surface<'w>, device: wgpu::Device) -> Self {
    let drawer = limne::TextureDrawer::new(
      &device,
      &wgpu::TextureFormat::Bgra8Unorm,
      limne::TextureDrawerInitRes {
        blend: Some(wgpu::BlendState {
          color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::SrcAlpha,
            dst_factor: wgpu::BlendFactor::DstAlpha,
            operation:  wgpu::BlendOperation::Add,
          },
          alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation:  wgpu::BlendOperation::Add,
          },
        }),
        ..Default::default()
      },
    );
    Self {
      surface,
      device,
      texture: Cell::new(None),
      drawer,
    }
  }

  fn copy_texture(
    &self,
    backbuffer: &wgpu::Texture,
    encoder: &mut wgpu::CommandEncoder,
    tex: &wgpu::SurfaceTexture,
  ) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("present pass"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view:           &tex.texture.create_view(&Default::default()),
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
    // pass.set_scissor_rect(viewport.x, viewport.y, viewport.z, viewport.w);
    self
      .drawer
      .render_into_pass(&mut pass, &limne::TextureDrawerResources {
        src:         &backbuffer.create_view(&Default::default()),
        bind_groups: &[],
        device:      &self.device,
        immediates:  &[],
      });
  }
}

impl<'w> Presenter for SurfacePresenter<'w> {
  fn get_wgpu_capabilities(&self) -> (wgpu::Limits, wgpu::Features) { Default::default() }

  fn texture_view(&self) -> &wgpu::TextureView { todo!() }

  fn present_texture(&self) -> &wgpu::Texture { todo!() }

  fn present(
    &self,
    backbuffer: &wgpu::Texture,
    encoder: &mut wgpu::CommandEncoder,
    _tex_dim: UVec2,
    _viewport: UVec4,
  ) -> PresentationResult<'_> {
    match self.surface.get_current_texture() {
      wgpu::CurrentSurfaceTexture::Timeout => {
        log::warn!("Swapchain timed out?");
        return PresentationResult::Ignored;
      }
      wgpu::CurrentSurfaceTexture::Occluded => {
        log::warn!("The window is occluded");
        return PresentationResult::ReconfigurationRequired(None);
      }
      wgpu::CurrentSurfaceTexture::Outdated => {
        log::warn!("Surface outdated");
        return PresentationResult::ReconfigurationRequired(None);
      }
      wgpu::CurrentSurfaceTexture::Lost => {
        log::warn!("Surface lost");
        return PresentationResult::ReconfigurationRequired(None);
      }
      wgpu::CurrentSurfaceTexture::Validation => {
        log::warn!("Surface validation error");
        return PresentationResult::Ignored;
      }
      wgpu::CurrentSurfaceTexture::Suboptimal(tex) => {
        log::warn!("Surface is suboptimal");
        self.copy_texture(backbuffer, encoder, &tex);
        self.texture.set(Some(tex));
        PresentationResult::ReconfigurationRequired(None)
      }
      wgpu::CurrentSurfaceTexture::Success(tex) => {
        self.copy_texture(backbuffer, encoder, &tex);
        self.texture.set(Some(tex));
        PresentationResult::Submitted
      }
    }
  }
  fn post_submit(&self) {
    if let Some(t) = self.texture.take() {
      t.present()
    }
  }
  fn reconfigure(&self, viewport: UVec2) {
    self
      .surface
      .configure(&self.device, &wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
        format: wgpu::TextureFormat::Bgra8Unorm,
        width: viewport.x,
        height: viewport.y,
        present_mode: wgpu::PresentMode::AutoVsync,
        desired_maximum_frame_latency: 2,
        alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        view_formats: vec![wgpu::TextureFormat::Bgra8Unorm],
      });
  }
}
