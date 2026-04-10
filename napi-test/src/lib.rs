#![deny(clippy::all)]
#![feature(try_blocks)]
#![allow(dead_code)]

use core::slice;
use std::{
  mem::ManuallyDrop,
  sync::atomic::{AtomicU32, Ordering},
};

use ash::vk;
use napi::bindgen_prelude::Uint8ArraySlice;
use napi_derive::napi;
use wgpu::hal::vulkan as wgvk;

use crate::gpu_alloc::Allocator;

pub(crate) mod gpu_alloc;

#[napi]
pub struct StrahlState {
  i: wgpu::Instance,
  dev: wgpu::Device,
  queue: wgpu::Queue,
  raw_state: ManuallyDrop<RawState>,
}

#[napi]
impl Drop for StrahlState {
  fn drop(&mut self) {
    unsafe {
      ManuallyDrop::drop(&mut self.raw_state);
    }
  }
}

#[napi]
pub async fn wgpu_init() -> napi::Result<StrahlState> {
  let r: anyhow::Result<StrahlState> = try {
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
    StrahlState {
      i: instance,
      raw_state: ManuallyDrop::new(raw),
      dev,
      queue,
    }
  };
  r.map_err(|x| napi::Error::from_reason(x.to_string()))
}

struct RawState {
  swapchain: wgvk::Texture,
  mapped: &'static [u8], // this is bad and unsound. must be rewritten somehow
}

// TODO: replace unwraps with proper error handling
async unsafe fn raw_wgpu_setup(
  instance: &wgvk::InstanceShared,
  dq: &wgpu::hal::OpenDevice<wgvk::Api>,
  phy: vk::PhysicalDevice,
) -> RawState {
  let extent = vk::Extent3D {
    width: 1024,
    height: 1024,
    depth: 1,
  };
  let alloc = Allocator::new(phy, dq.device.raw_device(), instance.raw_instance());
  let img = dq
    .device
    .raw_device()
    .create_image(
      &vk::ImageCreateInfo::default()
        .array_layers(1) // Vulkan implementation must support at least 256 array layers
        .extent(extent)
        .flags(vk::ImageCreateFlags::ALIAS)
        .format(vk::Format::R8G8B8A8_UINT)
        .image_type(vk::ImageType::TYPE_2D)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .mip_levels(1)
        .queue_family_indices(&[dq.device.queue_family_index()])
        .samples(vk::SampleCountFlags::TYPE_1) // That's for multisampling, we don't use it (now)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .tiling(vk::ImageTiling::LINEAR) // Linear tiling for predictable memory layout
        .usage(vk::ImageUsageFlags::TRANSFER_DST),
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
  let swapchain = dq.device.texture_from_raw(
    img,
    &wgpu::hal::TextureDescriptor {
      label: Some("framebuffer"),
      size: wgpu::Extent3d {
        width: extent.width,
        height: extent.height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8Uint,
      usage: wgpu::TextureUses::COPY_DST | wgpu::TextureUses::UNINITIALIZED,
      view_formats: vec![],
      memory_flags: wgpu::hal::MemoryFlags::PREFER_COHERENT, // I don't know what it exactly means, but it seems to be right
    },
    Some(Box::new(move || {
      println!("drop callback!");
      vk_device.unmap_memory(allocation);
      vk_device.destroy_image(img, None);
      vk_device.free_memory(allocation, None);
    })),
    wgvk::TextureMemory::External,
  );
  RawState { swapchain, mapped }
}

#[napi]
struct CpuSwapchain {
  buf: Vec<u8>,
  used_blocks: AtomicU32,
  pub width: u32,
  pub height: u32,
  pub channels: u32,
}

#[napi]
impl CpuSwapchain {
  #[napi(constructor)]
  pub fn new() -> napi::Result<Self> {
    let r: anyhow::Result<Self> = try {
      let w = 20;
      let h = 20;
      let n = 1;
      let c = 4;
      let buf = vec![0xFF000088u32.to_be_bytes(); c * h * w * n].into_flattened();
      Self {
        buf,
        used_blocks: AtomicU32::new(0),
        width: w as u32,
        height: h as u32,
        channels: c as u32,
      }
    };
    r.map_err(|err| napi::Error::from_reason(err.to_string()))
  }
  #[napi]
  pub fn acquire_next_texture<'env>(
    &'env mut self,
    env: &'env napi::Env,
  ) -> napi::Result<Uint8ArraySlice<'env>> {
    if self.used_blocks.load(Ordering::Relaxed).count_ones() > 0 {
      Err(napi::Error::from_reason("no free texture in the swapchain"))
    } else {
      self.used_blocks.fetch_and(0x01, Ordering::Acquire);
      let tex_size = (self.width * self.height * self.channels) as usize;
      let offset = 0 * tex_size;
      let res = &mut self.buf[offset..(offset + tex_size)];
      let ptr = res.as_ptr();

      // SAFETY: for now, at most one such reference is allowed
      unsafe {
        Uint8ArraySlice::from_external(
          env,
          res.as_mut_ptr(),
          res.len(),
          (&self.used_blocks, 1),
          |_, (blocks, blkid)| {
            println!("slice freed {}", ptr.add(1).read());
            blocks.fetch_and(!(0x01 << blkid), Ordering::Release);
          },
        )
      }
    }
  }
}
