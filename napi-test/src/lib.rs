#![deny(clippy::all)]
#![feature(try_blocks)]
#![allow(dead_code)]

use std::{
  io::{Seek, Write},
  os::{fd::AsRawFd, raw::c_uint},
};

use ash::vk;
use napi_derive::napi;
use nix::{
  fcntl::OFlag,
  sys::{
    memfd::{memfd_create, MFdFlags},
    mman,
    stat::Mode,
  },
};
use wgpu::hal::vulkan as wgvk;

#[napi(object)]
pub struct Inner {
  pub y: u32,
}

#[napi(object)]
pub struct Outer {
  pub x: u32,
  pub i: Inner,
}

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

#[napi]
struct TexWrapper {
  fd: std::os::fd::OwnedFd,
  pub width: i32,
  pub height: i32,
}

#[napi]
impl TexWrapper {
  #[napi(constructor)]
  pub fn new() -> napi::Result<TexWrapper> {
    let r: anyhow::Result<TexWrapper> = try {
      println!(
        "Rust PID: {} {}",
        nix::unistd::gettid(),
        nix::unistd::getpid()
      );
      let fd = memfd_create("name", MFdFlags::empty())?;
      let mut f = std::fs::File::from(fd);
      f.write(&[0xFF000088u32.to_be_bytes(); 400].as_flattened())?;
      f.flush()?;
      f.rewind()?;
      TexWrapper {
        fd: f.into(),
        width: 20,
        height: 20,
      }
    };
    r.map_err(|err| napi::Error::from_reason(err.to_string()))
  }
  #[napi]
  pub fn fd(&self) -> i32 {
    self.fd.as_raw_fd()
  }
}

#[napi]
fn new_tex_wrapper() -> napi::Result<TexWrapper> {
  TexWrapper::new()
}
