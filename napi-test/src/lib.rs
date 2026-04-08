#![deny(clippy::all)]
#![feature(try_blocks)]

use std::ptr::NonNull;

use ash::vk;
use napi_derive::napi;
use wgpu::hal::vulkan as wgvk;

// mod texture_infos;

// #[cfg_attr(feature = "node", napi)]
#[napi]
pub struct Hello {
  pub x: u32,
  pub path: String,
}

#[napi]
impl Hello {
  // #[node_export(constructor)]
  #[napi(constructor)]
  pub fn new(x: u32) -> Self {
    Hello {
      x,
      path: "asdfasdf".to_string(),
    }
  }
}

#[napi]
impl Hello {
  #[napi]
  pub fn aaa(&self) {}
}

#[napi]
pub fn plus_100(input: u32) -> napi::Result<Hello> {
  Ok(Hello {
    x: input + 100,
    path: std::env::current_dir()
      .map_err(|a| napi::Error::from_reason(a.to_string()))?
      .into_os_string()
      .into_string()
      .map_err(|_| napi::Error::from_reason("failed to convert path"))?,
  })
}

#[napi]
pub struct StrahlState {
  i: wgpu::Instance,
}

pub trait From2<F, T> {
  fn from(value: F) -> T;
  // fn _inexistent() -> U
  // where
  //   Self: Sized;
}

#[napi]
impl Drop for StrahlState {
  fn drop(&mut self) {
    println!("strahl dies.");
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
            .map(|dh| dh.display_handle().ok())
            .flatten(),
        },
        Some(Box::new(|opts| {})),
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
    let (d, q) = unsafe {
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
      adapter.create_device_from_hal(dq, &dev_desc)?
    };

    StrahlState { i: instance }
  };
  r.map_err(|x| napi::Error::from_reason(x.to_string()))
}
