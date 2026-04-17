#![deny(clippy::all)]
#![feature(try_blocks)]
#![allow(dead_code)]

use core::slice;
use std::path::Path;

use ash::vk;
use material::Material;
use wgpu::{TextureUsages, hal::vulkan as wgvk};

use crate::{
  gpu_alloc::Allocator, scene::Scene,
};

pub(crate) mod gpu_alloc;

pub struct Rasterizer {
  i:         wgpu::Instance,
  queue:     wgpu::Queue,
  dev:       wgpu::Device,
  raw_state: RawState,
}

pub mod geometry;
pub mod material;
pub mod scene;
pub mod uniform;

impl Rasterizer {
  pub async fn fill_framebuffer(&self) {
    let mut encoder = self
      .dev
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("filler-encoder"),
      });
    let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("clear pass"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view:           &self
          .raw_state
          .swapchain
          .create_view(&wgpu::wgt::TextureViewDescriptor {
            label:             None,
            format:            None,
            dimension:         None,
            usage:             Some(TextureUsages::RENDER_ATTACHMENT),
            aspect:            wgpu::TextureAspect::All,
            base_mip_level:    0,
            mip_level_count:   Some(1),
            base_array_layer:  0,
            array_layer_count: Some(1),
          }),
        depth_slice:    None,
        resolve_target: None,
        ops:            wgpu::Operations {
          load:  wgpu::LoadOp::Clear(wgpu::Color {
            r: 1.0,
            g: 45.0,
            b: 0.0,
            a: 255.0,
          }),
          store: wgpu::StoreOp::Store,
        },
      })],
      depth_stencil_attachment: None,
      timestamp_writes: None,
      occlusion_query_set: None,
      multiview_mask: None,
    });
    drop(pass);
    let idx = self.queue.submit([encoder.finish()]);
    let _res = self.dev.poll(wgpu::wgt::PollType::Wait {
      submission_index: Some(idx),
      timeout:          None,
    });
    println!("{:?}", &self.raw_state.mapped[0..8]);
  }
}

impl Rasterizer {
  pub async fn new() -> anyhow::Result<Rasterizer> {
    let r: anyhow::Result<Rasterizer> = try {
      let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
      desc.backends = wgpu::Backends::VULKAN;
      let instance = unsafe {
        wgpu::Instance::from_hal::<wgvk::Api>(wgvk::Instance::init_with_callback(
          &wgpu::hal::InstanceDescriptor {
            name: "A?",
            flags: desc.flags,
            memory_budget_thresholds: desc.memory_budget_thresholds,
            backend_options: desc.backend_options,
            telemetry: None, // May be required on DX12
            display: desc
              .display
              .as_ref()
              .and_then(|dh| dh.display_handle().ok()),
          },
          Some(Box::new(|_opts| {})),
        )?)
      };

      let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
          power_preference: wgpu::PowerPreference::HighPerformance,
          ..Default::default()
        })
        .await?;
      let dev_desc = wgpu::DeviceDescriptor {
        ..Default::default()
      };
      let (raw, (dev, queue)) = unsafe {
        let i = instance.as_hal::<wgvk::Api>().unwrap();
        let hal_adapter = adapter.as_hal::<wgvk::Api>().unwrap();
        let phy = hal_adapter.raw_physical_device();
        let dev_version = i
          .shared_instance()
          .raw_instance()
          .get_physical_device_properties(phy)
          .api_version;
        println!(
          "Device's API version: {dev_version} ({}.{}.{}.{})",
          vk::api_version_major(dev_version),
          vk::api_version_minor(dev_version),
          vk::api_version_patch(dev_version),
          vk::api_version_variant(dev_version)
        );
        let dq = hal_adapter.open_with_callback(
          dev_desc.required_features,
          &dev_desc.required_limits,
          &dev_desc.memory_hints,
          Some(Box::new(|opts| {
            opts.extensions.push(vk::EXT_EXTERNAL_MEMORY_DMA_BUF_NAME);
          })),
        )?;
        (
          raw_wgpu_setup(i.shared_instance(), &dq, phy).await,
          adapter.create_device_from_hal(dq, &dev_desc)?,
        )
      };

      // On wgpu shutdown device is dropped earlier than callback is called for some reason
      Rasterizer {
        i: instance,
        raw_state: raw.into_hal(&dev),
        dev,
        queue,
      }
    };
    r.map_err(|x| anyhow::anyhow!(x))
  }

  pub fn create_scene(&self) -> Scene { Scene::new() }
  pub fn create_material(&self, path: impl AsRef<Path>) -> anyhow::Result<Material> {
    let imported = strahl_import::reader::Material::read(path)?;
    Ok(Material::from_imported(&self.dev, &self.queue, imported))
  }
  pub fn create_geometry(&self) {}
  pub fn render(&self, _scene: &Scene) -> &[u8] { todo!() }
}

struct RawState {
  swapchain: wgpu::Texture,
  mapped:    &'static [u8], // this is bad and unsound. must be rewritten somehow
}
struct RawStateVk {
  swapchain:     wgvk::Texture,
  wgpu_tex_desc: wgpu::TextureDescriptor<'static>,
  mapped:        &'static [u8], // this is bad and unsound. must be rewritten somehow
}

impl RawStateVk {
  pub fn into_hal(self, dev: &wgpu::Device) -> RawState {
    RawState {
      swapchain: unsafe {
        dev.create_texture_from_hal::<wgvk::Api>(self.swapchain, &self.wgpu_tex_desc)
      },
      mapped:    self.mapped,
    }
  }
}

// TODO: replace unwraps with proper error handling
async unsafe fn raw_wgpu_setup(
  vk_instance: &wgvk::InstanceShared,
  dq: &wgpu::hal::OpenDevice<wgvk::Api>,
  phy: vk::PhysicalDevice,
) -> RawStateVk {
  let extent = vk::Extent3D {
    width:  1024,
    height: 1024,
    depth:  1,
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
          .format(vk::Format::R8G8B8A8_UINT)
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
    let allocation = alloc
      .allocate(
        reqs.size,
        vk::MemoryPropertyFlags::HOST_COHERENT,
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
      format:          wgpu::TextureFormat::Rgba8Uint,
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
      format:          wgpu::TextureFormat::Rgba8Uint,
      usage:           wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
      view_formats:    &[],
    };
    let swapchain = dq.device.texture_from_raw(
      img,
      &vk_tex_desc,
      Some(Box::new(move || {
        println!("drop callback!");
        vk_device.unmap_memory(allocation);
        vk_device.destroy_image(img, None);
        vk_device.free_memory(allocation, None);
      })),
      wgvk::TextureMemory::External,
    );
    RawStateVk {
      swapchain,
      wgpu_tex_desc,
      mapped,
    }
  }
}

pub mod shader_manager;
