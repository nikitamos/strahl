#![feature(associated_type_defaults)]
#![feature(duration_millis_float)]
#![feature(sync_nonpoison)]
#![feature(nonpoison_rwlock)]
#![deny(clippy::all)]
#![allow(dead_code)]
#![allow(irrefutable_let_patterns)]

use std::{
  ffi::CStr,
  path::Path,
  sync::{
    Arc,
    nonpoison::{RwLock, RwLockReadGuard, RwLockWriteGuard},
  },
  time::SystemTime,
};

use anyhow::anyhow;
use ash::vk;
use glam::UVec2;
use material::Material;
use wgpu::{Backends, hal::vulkan as wgvk, wgt::WgpuHasDisplayHandle};

use crate::{
  geometry::Geometry,
  postprocessing::{
    BloomCreateInfo, BloomPostProcess, PostProcessCreateInfoBase, PostProcessInfo, PostProcessStep,
  },
  presenter::{PresentationResult, Presenter, SurfacePresenter, TexturePresenter},
  scene::Scene,
  shader_manager::ShaderManager,
};

pub(crate) mod gpu_alloc;
pub mod postprocessing;
pub mod skybox;

pub struct Rasterizer {
  i:           wgpu::Instance,
  queue:       wgpu::Queue,
  dev:         wgpu::Device,
  presenter:   Box<dyn Presenter>,
  postprocess: Vec<Box<dyn postprocessing::PostProcessStep>>,
  target:      limne::TextureProvider,
  target2:     limne::TextureProvider,
  drawer:      Option<limne::TextureDrawer>,
  manager:     Arc<ShaderManager>,
  info:        RwLock<RasterizerStateInfo>,
  loader:      loader::AssetLoader,
  depth:       limne::TextureProvider,
  buffer_side: u32,
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
  pub state:       RasterizerStateInfo,
  pub wgpu:        RasterizerWgpuInfo,
  pub buffer_side: u32,
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
  pub fn info(&self) -> RwLockReadGuard<'_, RasterizerStateInfo> { self.info.read() }
  pub fn info_mut(&self) -> RwLockWriteGuard<'_, RasterizerStateInfo> { self.info.write() }
  pub async fn new(
    RasterizerCreateInfo {
      state: info,
      wgpu: wgpu_info,
      buffer_side,
    }: RasterizerCreateInfo,
  ) -> anyhow::Result<Rasterizer> {
    log::info!("Creating Rasterizer backend?");

    let WgpuState {
      instance,
      device: dev,
      queue,
      presenter,
    } = Self::create_wgpu_state(wgpu_info, &info).await?;

    let target = create_target(buffer_side, &dev, "target 1");
    let target2 = create_target(buffer_side, &dev, "target 2");
    let depth = limne::TextureProvider::new(&dev, limne::TextureProviderDescriptor {
      label:           Some("depth".to_string()),
      size:            wgpu::Extent3d {
        width:                 buffer_side,
        height:                buffer_side,
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

    let pp = BloomPostProcess::create(
      BloomCreateInfo {
        kernel_depth: 12,
        s:            6.0,
        blur_iterations: 2
      },
      PostProcessCreateInfoBase {
        target_dim:     buffer_side,
        texture_format: target.format(),
        depth_format:   wgpu::TextureFormat::Depth16Unorm,
        device:         dev.clone(),
        uniform:        &manager.uniforms(),
      },
    );

    // On wgpu shutdown device is dropped earlier than callback is called for some reason
    Ok(Rasterizer {
      i: instance,
      presenter,
      loader: loader::AssetLoader::new(dev.clone(), queue.clone()),
      queue,
      drawer: None,
      dev,
      target,
      target2,
      manager,
      info: RwLock::new(info),
      depth,
      buffer_side,
      postprocess: vec![Box::new(pp)],
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
          alpha_mode: wgpu::CompositeAlphaMode::Opaque,
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
        instance,
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
  #[deprecated(note = "Use AssetLoader to load materials")]
  pub fn load_material(&self, path: impl AsRef<Path>) -> anyhow::Result<Arc<Material>> {
    self.loader.load_material(path)
  }
  #[deprecated(note = "Use AssetLoader to load meshes")]
  pub fn load_mesh(&self, path: impl AsRef<Path>) -> anyhow::Result<Arc<Geometry>> {
    self.loader.load_mesh(path)
  }
  pub fn asset_loader(&self) -> loader::AssetLoader { self.loader.clone() }
  pub fn render(&self, scene: &Scene, camera: &Camera) -> PresentationResult<'_> {
    let begin = SystemTime::now();
    let viewport = self.info.read().viewport;
    let viewport_size =
      viewport.clamp(UVec2::ZERO, glam::uvec2(self.buffer_side, self.buffer_side));
    if viewport_size != viewport {
      log::warn!("Viewport is too large; its size is clamped to texture dimensions");
    }

    self.manager.uniforms_mut().camera = camera.clone();
    self.manager.uniforms_mut().viewport_size = viewport_size;
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
      scene.draw_skybox(&mut pass);
    }
    let res = self.copy_to_presenter(&mut encoder);
    let postprocess = self.postprocess();
    let index = self
      .queue
      .submit(std::iter::once(encoder.finish()).chain(postprocess));
    self.presenter.post_submit();
    log::trace!("work submitted to the GPU");

    // TODO: wait asynchronously (?)
    let _ = self.dev.poll(wgpu::wgt::PollType::Wait {
      submission_index: Some(index),
      timeout:          None,
    });

    let end = SystemTime::now();
    let dur = end.duration_since(begin).unwrap().as_millis_f32();
    log::trace!("finished in {dur:.2}ms, ~{:.0} draw/second", 1000.0 / dur);
    if let PresentationResult::ReconfigurationRequired(_) = res {
      self.presenter.reconfigure(self.info().viewport);
    }
    res
  }

  fn copy_to_presenter(&self, encoder: &mut wgpu::CommandEncoder) -> PresentationResult<'_> {
    let viewport = self.info().viewport;
    self.presenter.present(
      self.get_presenting_target().tex(),
      encoder,
      glam::uvec2(self.buffer_side, self.buffer_side),
      glam::uvec4(0, 0, viewport.x, viewport.y),
    )
  }

  fn postprocess(&self) -> impl Iterator<Item = wgpu::CommandBuffer> {
    self.postprocess.iter().enumerate().map(|(i, step)| {
      let (origin, target) = if i % 2 == 0 {
        (&self.target, &self.target2)
      } else {
        (&self.target, &self.target2)
      };
      let viewport = self.info().viewport;
      let dimensions = self.buffer_side;
      step.apply(origin, target, PostProcessInfo {
        viewport: glam::vec2(viewport.x as f32, viewport.y as f32),
        uniform:  &self.manager.uniforms(),
        dimensions
      })
    })
  }

  fn get_presenting_target(&self) -> &limne::TextureProvider {
    if self.postprocess.len().is_multiple_of(2) {
      &self.target
    } else {
      &self.target2
    }
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

fn create_target(buffer_side: u32, dev: &wgpu::Device, label: &str) -> limne::TextureProvider {
  limne::TextureProvider::new(dev, limne::TextureProviderDescriptor {
    label:           Some(label.to_string()),
    size:            wgpu::Extent3d {
      width:                 buffer_side,
      height:                buffer_side,
      depth_or_array_layers: 1,
    },
    mip_level_count: 1,
    sample_count:    1,
    dimension:       wgpu::TextureDimension::D2,
    format:          wgpu::TextureFormat::Rgba8Unorm,
    usage:           wgpu::TextureUsages::RENDER_ATTACHMENT
      | wgpu::TextureUsages::COPY_SRC
      | wgpu::TextureUsages::COPY_DST
      | wgpu::TextureUsages::TEXTURE_BINDING,
    view_formats:    vec![wgpu::TextureFormat::Rgba8Unorm],
  })
}

pub mod loader;

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

  let instance = if backends == wgpu::Backends::VULKAN && !vk_extensions.is_empty() {
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
  } else if !vk_extensions.is_empty() {
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
