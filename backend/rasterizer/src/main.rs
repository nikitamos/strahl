use std::f32::consts::TAU;

use anyhow::anyhow;
use glam::Mat4;
use image::Rgba;
use rasterizer::presenter::PresentationResult;
pub use rasterizer::*;

// pub mod renderdoc {
//   use std::{
//     os::raw::c_void,
//     ptr::{self, NonNull},
//   };

//   #[repr(transparent)]
//   pub struct Api {
//     handle: ptr::NonNull<c_void>,
//   }

//   unsafe extern "C" {
//     fn create_renderdoc_api() -> *mut c_void;
//     fn destroy_renderdoc_api(api: *mut c_void);
//     fn renderdoc_start_capture(api: *const c_void);
//     fn renderdoc_end_capture(api: *const c_void);
//   }
//   impl Api {
//     pub fn new() -> Option<Self> {
//       unsafe {
//         let handle = create_renderdoc_api();
//         if handle.is_null() {
//           None
//         } else {
//           Some(Self {
//             handle: NonNull::new_unchecked(handle),
//           })
//         }
//       }
//     }
//     pub fn start_frame_capture(&self) {
//       unsafe {
//         renderdoc_start_capture(self.handle.as_ptr());
//       }
//     }
//     pub fn end_frame_capture(&self) {
//       unsafe {
//         renderdoc_end_capture(self.handle.as_ptr());
//       }
//     }
//   }
//   impl Drop for Api {
//     fn drop(&mut self) {
//       unsafe {
//         destroy_renderdoc_api(self.handle.as_mut());
//       }
//     }
//   }
// }

pub fn main() -> anyhow::Result<()> {
  env_logger::Builder::new()
    .filter_level(log::LevelFilter::Info)
    .filter_module("rasterizer", log::LevelFilter::Trace)
    .format_timestamp(None)
    .parse_default_env()
    .init();

  tokio::runtime::Builder::new_current_thread()
    .build()?
    .block_on(true_main())?;
  Ok(())
}

pub async fn true_main() -> anyhow::Result<()> {
  // let mut rdoc = renderdoc::Api::new().unwrap();
  let mut s = String::new();
  std::io::stdin().read_line(&mut s)?;

  let size = glam::uvec2(800, 600);
  let mut strahl = Rasterizer::new(RasterizerCreateInfo {
    state: RasterizerStateInfo { viewport: size },
    wgpu:  RasterizerWgpuInfo {
      wgpu_setup: WgpuSetup::Managed,
      target:     PresentTarget::ManagedMappedRam,
    },
  })
  .await?;
  let loader = strahl.asset_loader();
  // let material = strahl.load_material("../../../strahl-import/assets/gas.zip")?;
  // let geometry = strahl.load_mesh("../../../strahl-import/assets/lava/Lava.gltf")?;
  let mut scene = strahl.create_scene();
  let skybox = loader.load_skybox("/HDD/home/nikita/BSUIR/ExoplanetsCatalog/assets/skybox")?;
  scene.set_skybox(skybox);
  // let _body = scene.add_body(geometry, material);
  let aspect = (size.x as f32) / (size.y as f32);
  const POINTS: usize = 10;
  for i in 0..POINTS {
    // rdoc.start_frame_capture();
    log::trace!("rendering image {i}, of {POINTS}");
    let phi = i as f32 / TAU;

    let eye = glam::vec3(0.0, phi.cos(), phi.sin());
    let camera = Camera {
      projection: dbg!(Mat4::orthographic_lh(
        -1.0,
        1.0,
        -1.0 / aspect,
        1.0 / aspect,
        0.0,
        3.0
      )),
      camera:     dbg!(Mat4::look_at_lh(
        eye,
        glam::Vec3::ZERO,
        eye.cross(glam::Vec3::X),
      )),
    };

    let PresentationResult::Mapped(test) = strahl.render(&scene, &camera) else {
      unreachable!()
    };
    // rdoc.end_frame_capture();
    let buf =
      image::ImageBuffer::<Rgba<u8>, _>::from_raw(size.x, size.y, test).ok_or_else(|| {
        log::error!("failed to import image");
        anyhow!("failed to import image")
      })?;
    buf.save(format!("out{i}.png"))?;
  }

  Ok(())
}
