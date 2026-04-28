#![deny(clippy::all)]
#![allow(dead_code)]
#![feature(associated_type_defaults)]
#![feature(duration_millis_float)]
#![feature(rwlock_downgrade)]
#![deny(clippy::all)]
#![allow(dead_code)]

use std::{ffi::CStr, path::Path, sync::Arc, time::SystemTime};

use anyhow::anyhow;
use ash::vk;
use glam::UVec2;
use material::Material;
use wgpu::{Backends, hal::vulkan as wgvk, naga::back, wgt::WgpuHasDisplayHandle};

use crate::{
  geometry::Geometry,
  presenter::{PresentationResult, Presenter, SurfacePresenter, TexturePresenter},
  scene::Scene,
  shader_manager::ShaderManager,
};

pub(crate) mod gpu_alloc;

pub struct Rasterizer {
  i:         wgpu::Instance,
  queue:     wgpu::Queue,
  dev:       wgpu::Device,
  presenter: Box<dyn Presenter>,
  target:    limne::TextureProvider,
  drawer:    Option<limne::TextureDrawer>,
  manager:   Arc<ShaderManager>,
  info:      RasterizerStateInfo,
  depth:     limne::TextureProvider,
}

pub mod geometry;
mod limne;
pub mod material;
pub mod scene;
pub mod uniform;

pub enum WgpuSetup {
  External(wgpu::Instance, wgpu::Device, wgpu::Queue),
  Managed,
}

pub enum PresentTarget {
  ExternalTexture(wgpu::Texture),
  ManagedTexture,
  ManagedMappedRam,
  ManagedWindow(
    Box<dyn wgpu::wgt::WgpuHasDisplayHandle>,
    Box<dyn wgpu::WindowHandle>,
  ),
}

pub struct RasterizerCreateInfo {
  pub state: RasterizerStateInfo,
  pub wgpu:  RasterizerWgpuInfo,
}

pub struct RasterizerStateInfo {
  pub viewport: UVec2,
}

pub struct RasterizerWgpuInfo {
  pub wgpu_setup: WgpuSetup,
  pub target:     PresentTarget,
}

#[derive(zerocopy::KnownLayout, zerocopy::IntoBytes, zerocopy::Immutable, Default, Clone)]
#[repr(C)]
pub struct Camera {
  pub projection: glam::Mat4,
  pub camera:     glam::Mat4,
}

struct WgpuState {
  instance:  wgpu::Instance,
  device:    wgpu::Device,
  queue:     wgpu::Queue,
  presenter: Box<dyn Presenter>,
}

impl Rasterizer {
  pub async fn new(
    RasterizerCreateInfo {
      state: info,
      wgpu: wgpu_info,
    }: RasterizerCreateInfo,
  ) -> anyhow::Result<Rasterizer> {
    log::info!("Creating Rasterizer backend?");

    let WgpuState {
      instance,
      device: dev,
      queue,
      presenter,
    } = Self::create_wgpu_state(wgpu_info, &info).await?;

    let target = limne::TextureProvider::new(&dev, limne::TextureProviderDescriptor {
      label:           Some(" a kind of render target".to_string()),
      size:            wgpu::Extent3d {
        width:                 info.viewport.x,
        height:                info.viewport.y,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count:    1,
      dimension:       wgpu::TextureDimension::D2,
      format:          wgpu::TextureFormat::Rgba8Unorm,
      usage:           wgpu::TextureUsages::RENDER_ATTACHMENT
        | wgpu::TextureUsages::COPY_SRC
        | wgpu::TextureUsages::TEXTURE_BINDING,
      view_formats:    vec![wgpu::TextureFormat::Rgba8Unorm],
    });
    let depth = limne::TextureProvider::new(&dev, limne::TextureProviderDescriptor {
      label:           Some("depth".to_string()),
      size:            wgpu::Extent3d {
        width:                 info.viewport.x,
        height:                info.viewport.y,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count:    1,
      dimension:       wgpu::TextureDimension::D2,
      format:          wgpu::TextureFormat::Depth16Unorm,
      usage:           wgpu::TextureUsages::RENDER_ATTACHMENT,
      view_formats:    vec![wgpu::TextureFormat::Depth16Unorm],
    });

    let manager = Arc::new(ShaderManager::new(
      dev.clone(),
      wgpu::TextureFormat::Depth16Unorm,
    ));

    // On wgpu shutdown device is dropped earlier than callback is called for some reason
    Ok(Rasterizer {
      i: instance,
      presenter,
      queue,
      drawer: None,
      dev,
      target,
      manager,
      info,
      depth,
    })
  }

  async fn create_wgpu_state(
    info: RasterizerWgpuInfo,
    state: &RasterizerStateInfo,
  ) -> anyhow::Result<WgpuState> {
    match info.wgpu_setup {
      WgpuSetup::External(instance, device, queue) => {
        Self::create_external_wgpu(instance, device, queue, info.target)
      }
      WgpuSetup::Managed => Self::create_managed_wgpu(info, state).await,
    }
  }

  async fn create_managed_wgpu(
    info: RasterizerWgpuInfo,
    state: &RasterizerStateInfo,
  ) -> anyhow::Result<WgpuState> {
    match info.target {
      PresentTarget::ExternalTexture(_) => Err(anyhow::anyhow!(
        "External texture must be used with external wgpu setup"
      )),
      PresentTarget::ManagedTexture => todo!(),
      PresentTarget::ManagedMappedRam => Self::create_some_ram_presenter(state).await,
      PresentTarget::ManagedWindow(display, window) => {
        let (instance, adapter, desc) =
          instantiate_wgpu(Backends::PRIMARY, Some(display), vec![]).await?;
        let surface = instance.create_surface(wgpu::SurfaceTarget::Window(window))?;
        let (device, queue) = adapter.request_device(&desc).await?;

        surface.configure(&device, &wgpu::SurfaceConfiguration {
          usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
          format: wgpu::TextureFormat::Bgra8Unorm,
          width: state.viewport.x,
          height: state.viewport.y,
          present_mode: wgpu::PresentMode::AutoVsync,
          desired_maximum_frame_latency: 2,
          alpha_mode: wgpu::CompositeAlphaMode::Inherit,
          view_formats: vec![wgpu::TextureFormat::Bgra8Unorm],
        });

        Ok(WgpuState {
          presenter: Box::new(SurfacePresenter::new(surface, device.clone())),
          instance,
          device,
          queue,
        })
      }
    }
  }
  fn create_external_wgpu(
    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
    target: PresentTarget,
  ) -> anyhow::Result<WgpuState> {
    match target {
      PresentTarget::ExternalTexture(texture) => Ok(WgpuState {
        instance: instance,
        device,
        queue,
        presenter: Box::new(TexturePresenter::new(texture)),
      }),
      _ => Err(anyhow!(
        "Only external texture may be used with external wgpu setup"
      )),
    }
  }

  pub fn create_scene(&self) -> Scene { Scene::new(Arc::clone(&self.manager)) }
  pub fn load_material(&self, path: impl AsRef<Path>) -> anyhow::Result<Arc<Material>> {
    let imported = strahl_import::reader::Material::read(path)?;
    Ok(Arc::new(Material::from_imported(
      &self.dev,
      &self.queue,
      imported,
    )))
  }
  pub fn load_mesh(&self, path: impl AsRef<Path>) -> anyhow::Result<Arc<Geometry>> {
    let gltf = strahl_import::reader::GltfGeometry::import_validate(path)?;
    Geometry::from_gltf(&self.dev, gltf).map(Arc::new)
  }
  pub fn render(&mut self, scene: &Scene, camera: &Camera) -> PresentationResult<'_> {
    let begin = SystemTime::now();
    self.manager.uniforms_mut().camera = camera.clone();
    self.manager.uniforms_mut().viewport_size = self.info.viewport;
    self.manager.uniforms().upload(&self.queue);
    let mut encoder = self
      .dev
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zbuf_smoothing"),
      });
    {
      let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("zbuf_smoothing"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view:           &self.target,
          resolve_target: None,
          ops:            wgpu::Operations {
            load:  wgpu::LoadOp::Clear(wgpu::Color {
              r: 1.0,
              g: 1.0,
              b: 1.0,
              a: 1.0,
            }),
            store: wgpu::StoreOp::Store,
          },
          depth_slice:    None,
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
          view:        &self.depth,
          depth_ops:   Some(wgpu::Operations {
            load:  wgpu::LoadOp::Clear(1.0),
            store: wgpu::StoreOp::Store,
          }),
          stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
      });
      // pass.set_viewport(200.0, 200.0, 200.0, 150.0, 0.0, 1.0);
      pass.set_bind_group(0, self.manager.uniforms().bind_group(), &[]);
      for body in scene.bodies() {
        body.draw(&mut pass);
      }
    }
    let res = self.copy_to_presenter(&mut encoder);
    let index = self.queue.submit(std::iter::once(encoder.finish()));
    self.presenter.post_submit();
    log::trace!("work submitted to the GPU");
    // TODO: wait asynchronously
    let _ = self.dev.poll(wgpu::wgt::PollType::Wait {
      submission_index: Some(index),
      timeout:          None,
    });
    let end = SystemTime::now();
    let dur = end.duration_since(begin).unwrap().as_millis_f32();
    log::trace!("finished in {dur:.2}ms, ~{:.0} draw/second", 1000.0 / dur);
    res
  }

  fn copy_to_presenter(&self, encoder: &mut wgpu::CommandEncoder) -> PresentationResult<'_> {
    self.presenter.present(
      self.target.texture(),
      encoder,
      self.info.viewport,
      glam::uvec4(200, 200, 200, 150),
    )
  }

  pub fn required_features() -> wgpu::Features {
    wgpu::Features {
      features_wgpu:   wgpu::FeaturesWGPU::empty(),
      features_webgpu: wgpu::FeaturesWebGPU::IMMEDIATES,
    }
  }
  pub fn required_limits() -> wgpu::Limits {
    wgpu::Limits {
      max_immediate_size: 256,
      ..Default::default()
    }
  }
  async fn create_some_ram_presenter(state: &RasterizerStateInfo) -> anyhow::Result<WgpuState> {
    let (instance, adapter, dev_desc) =
      instantiate_wgpu(wgpu::Backends::VULKAN, None, vec![]).await?;
    let (raw, (dev, queue)) =
      unsafe { create_presenter_dev_queue(&instance, adapter, dev_desc, state).await? };
    // Ok((instance, raw.from_hal(&dev), dev, queue))
    Ok(WgpuState {
      instance,
      queue,
      presenter: Box::new(raw.from_hal(&dev)),
      device: dev,
    })
  }
}

/// Creates a WGPU instance, optionally enabling Vulkan instance extensions
///
/// # Panics
/// Panics if backends is not equal to [`wgpu::Backends::VULKAN`] and `vk_extensions`
/// is not empty
async fn instantiate_wgpu(
  backends: wgpu::Backends,
  display: Option<Box<dyn WgpuHasDisplayHandle>>,
  mut vk_extensions: Vec<&'static CStr>,
) -> Result<
  (
    wgpu::Instance,
    wgpu::Adapter,
    wgpu::wgt::DeviceDescriptor<Option<&'static str>>,
  ),
  anyhow::Error,
> {
  let mut desc = wgpu::InstanceDescriptor {
    backends,
    display,
    ..wgpu::InstanceDescriptor::new_without_display_handle_from_env()
  };
  desc.backends = backends;
  // So far no instance extensions requested

  let instance = if backends == wgpu::Backends::VULKAN && vk_extensions.len() > 0 {
    log::debug!(
      "Creating WGPU instance as Vulkan HAL ({} extensions requested)",
      vk_extensions.len()
    );
    unsafe {
      wgpu::Instance::from_hal::<wgvk::Api>(wgvk::Instance::init_with_callback(
        &wgpu::hal::InstanceDescriptor {
          name: "A?",
          flags: desc.flags,
          memory_budget_thresholds: desc.memory_budget_thresholds,
          backend_options: desc.backend_options,
          telemetry: None, // May be required on DX12
          display: desc.display.as_ref().and_then(|dh| {
            dh.display_handle()
              .inspect_err(|e| log::error!("Failed to retrieve display handle: {e}"))
              .ok()
          }),
        },
        Some(Box::new(|opts| {
          opts.extensions.append(&mut vk_extensions);
        })),
      )?)
    }
  } else if vk_extensions.len() > 0 {
    panic!("Requested possibly non-Vulkan backend with Vulkan extensions")
  } else {
    wgpu::Instance::new(desc)
  };

  let adapter = instance
    .request_adapter(&wgpu::RequestAdapterOptions {
      power_preference: wgpu::PowerPreference::HighPerformance,
      ..Default::default()
    })
    .await?;
  let dev_desc = wgpu::DeviceDescriptor {
    required_features: Rasterizer::required_features(),
    required_limits: Rasterizer::required_limits(),
    ..Default::default()
  };
  Ok((instance, adapter, dev_desc))
}

pub mod presenter;

async unsafe fn create_presenter_dev_queue(
  instance: &wgpu::Instance,
  adapter: wgpu::Adapter,
  dev_desc: wgpu::wgt::DeviceDescriptor<Option<&str>>,
  info: &RasterizerStateInfo,
) -> Result<(presenter::RawMappedPresenter, (wgpu::Device, wgpu::Queue)), anyhow::Error> {
  unsafe {
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
    Ok((
      presenter::raw_wgpu_setup(
        i.shared_instance(),
        &dq,
        phy,
        info.viewport.x,
        info.viewport.y,
      )
      .await,
      adapter.create_device_from_hal(dq, &dev_desc)?,
    ))
  }
}

pub mod shader_manager;
