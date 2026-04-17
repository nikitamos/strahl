#![deny(clippy::all)]
#![feature(try_blocks)]

mod erase;

use std::{any, sync::Arc};

use glam::{Mat4, Vec3};
use rasterizer::Rasterizer;

use napi::{
  Env,
  bindgen_prelude::{ArrayBuffer, Float32Array, Uint8Array, Uint8ArraySlice},
  tokio::{self, sync::RwLock},
};
use napi_derive::napi;

macro_rules! napi_todo {
  ($reason:literal) => {
    return Err(napi::Error::from_reason($reason))
  };
  () => {
    napi_todo!("not implemented")
  };
}

pub trait AnyhowIntoNapi<T> {
  fn into_napi(self) -> napi::Result<T>;
}

impl<T> AnyhowIntoNapi<T> for anyhow::Result<T> {
  fn into_napi(self) -> napi::Result<T> {
    self.map_err(|e| napi::Error::from_reason(format!("{e}")))
  }
}

/// The entry point to the `strahl`'s API.
/// This class provides methods for creating scenes, managing resources and rendering.
#[napi]
pub struct Strahl {
  backend: Arc<RwLock<Rasterizer>>,
}

/// Class wrapping a geometry. So far it doesn't have any methods exposed,
/// so it may thought of as a kind of opaque handler.
///
/// Geometry in `strahl` is a generic shape that may be used by bodies or light sources.
#[napi]
pub struct Geometry(Arc<rasterizer::geometry::Geometry>);

#[napi]
impl Geometry {
  /// Tries to downcast self to the sphere
  #[napi]
  pub fn as_sphere(&self) -> napi::Result<Sphere> {
    napi_todo!()
  }
  /// Tries to downcast self to the mesh
  #[napi]
  pub fn as_mesh(&self) -> napi::Result<Mesh> {
    napi_todo!()
  }
}

/// Projection used by the camera. Two types of projection are supported:
/// * orthographic (https://registry.khronos.org/OpenGL-Refpages/gl2.1/xhtml/glOrtho.xml),
/// * perspective (https://registry.khronos.org/OpenGL-Refpages/gl2.1/xhtml/gluPerspective.xml)
#[napi]
pub enum Projection {
  /// Orthographic projection. See more at
  /// https://registry.khronos.org/OpenGL-Refpages/gl2.1/xhtml/glOrtho.xml
  Ortho {
    /// Left clipping plane
    left: f64,
    /// Right clipping plane
    right: f64,
    /// Bottom clipping plane
    bottom: f64,
    /// Top clipping plane
    top: f64,
    /// Nearer clipping plane
    near: f64,
    /// Farther clipping plane
    far: f64,
  },
  /// Perspective projection. See more at
  /// https://registry.khronos.org/OpenGL-Refpages/gl2.1/xhtml/gluPerspective.xml
  Perspective {
    /// Field of view angle in vertical plane, **unit is unspecified so far**
    fovy: f64,
    /// Aspect ratio, i.e. the ration of viewport width to its height
    aspect: f64,
    /// Nearer clipping plane
    z_near: f64,
    /// Farther clipping plane
    z_far: f64,
  },
}

#[napi(object)]
pub struct Camera {
  /// The projection.
  pub projection: Projection,
  /// The pointer being observed by the camera (array of length 3).
  pub look_at: Float32Array,
  /// The location of the camera (array of length 3)
  pub location: Float32Array,
}

/// Class wrapping a material. So far it doesn't have any methods exposed,
/// so it may thought of as a kind of opaque handler.
#[napi]
pub struct Material(Arc<rasterizer::material::Material>);

/// Anything that can be rendered
#[napi]
pub struct Body(Arc<rasterizer::scene::Body>);

/// Represents geometry of a perfect sphere
#[napi]
pub struct Sphere {}

/// Represents geometry of a mesh of triangles
#[napi]
pub struct Mesh {}

/// Represents geometry of a single point
#[napi]
pub struct Point {}

#[napi]
impl Body {
  #[napi(setter)]
  pub fn set_material(&mut self, m: &Material) -> napi::Result<()> {
    napi_todo!()
  }
  #[napi(getter)]
  pub fn material(&self) -> napi::Result<Material> {
    napi_todo!()
  }
  /// Gets the position of the body relative to the world origin. Returns the array of length 3.
  #[napi(getter)]
  pub fn position(&self) -> Float32Array {
    let rf = self.0.translation();
    Float32Array::with_data_copied(rf.as_ref())
  }
  /// Sets the position of the body relative to the world origin. Expects an array of length 3.
  #[napi(setter)]
  pub fn set_position(&mut self, pos: &[f32]) -> napi::Result<()> {
    if pos.len() != 3 {
      Err(napi::Error::from_reason("Expected a slice of length 3"))
    } else {
      self.0.set_translation(glam::Vec3::from_slice(pos));
      Ok(())
    }
  }
}

/// Object aggregating information required to create a [`Body`]
#[napi(object)]
pub struct BodyCreateInfo<'env> {
  /// Array of length 3 representing the position of the body
  /// relative to the world origin,
  /// i.e. the translation applied to the geometry.
  pub position: ArrayBuffer<'env>,
  /// Rotation applied to the geometry in its local coordinates,
  /// represented as Euler's angles.
  pub rotation: ArrayBuffer<'env>,
}

/// Class representing a light source
#[napi]
pub struct LightSource {}

/// A collection of objects (bodies and lights) that participate in rendering.
///
/// So far it doesn't have any methods exposed,
/// so it may thought of as a kind of opaque handler.
#[napi]
pub struct Scene(rasterizer::scene::Scene);

/// Object aggregating the information required to create [`LightSource`].
/// The only supported light type so far is omnidirectional: each point
/// of the source's surface emits equal amount of radiance in all direction.
#[napi]
pub enum LightCreateInfo {
  Omnidirectional {
    /// Radiance of the light source in red, green and blue wavelength of spectrum.
    color: Float32Array,
    /// Array of length 3 representing the position of the light source
    /// relative to the world origin,
    /// i.e. the translation applied to the geometry.
    position: Float32Array,
  },
}

#[napi]
impl Scene {
  #[napi]
  /// Adds a body to the scene
  pub fn add_body<'env>(
    &mut self,
    _env: &'env Env,
    info: BodyCreateInfo<'env>,
    material: &'env Material,
    geometry: &'env Geometry,
  ) -> Body {
    Body(
      self
        .0
        .add_body(Arc::clone(&geometry.0), Arc::clone(&material.0)),
    )
  }

  /// Adds a light source to the scene
  #[napi]
  pub fn add_light_source(
    &mut self,
    info: LightCreateInfo,
    geometry: &Geometry,
  ) -> napi::Result<LightSource> {
    napi_todo!()
  }
}

#[napi]
impl Strahl {
  /// Creates a new instance of `Strahl`
  #[napi(factory)]
  pub async fn create(info: StrahlCreateInfo) -> napi::Result<Self> {
    env_logger::builder()
      .default_format()
      .format_timestamp(None)
      .filter_module("strahl_import", log::LevelFilter::Warn)
      .filter_module("rasterizer", log::LevelFilter::Trace)
      .filter_module("strahl_js", log::LevelFilter::Trace)
      .parse_default_env()
      .init();

    Ok(Self {
      backend: Arc::new(RwLock::new(
        Rasterizer::new(rasterizer::RasterizerCreateInfo {
          viewport: glam::uvec2(info.width, info.height),
        })
        .await
        .into_napi()?,
      )),
    })
  }
  /// Creates an empty scene
  #[napi]
  pub async fn create_scene(&self) -> Scene {
    Scene(self.backend.read().await.create_scene())
  }
  /// Loads material from given path
  #[napi]
  pub async fn load_material(&self, path: String) -> napi::Result<Material> {
    let backend = Arc::clone(&self.backend);
    tokio::spawn(async move { backend.read().await.load_material(&path).map(Material) })
      .await
      .map_err(|e| e.into())
      .flatten()
      .into_napi()
  }
  /// Loads model geometry from given path
  #[napi]
  #[deprecated(note = "Use load_mesh instead")]
  pub async fn load_model(&self, path: String) -> napi::Result<Geometry> {
    self.load_mesh(path).await
  }
  /// Loads glTF mesh from file
  #[napi]
  pub async fn load_mesh(&self, path: String) -> napi::Result<Geometry> {
    let backend = Arc::clone(&self.backend);
    tokio::spawn(async move { backend.read().await.load_mesh(&path).map(Geometry) })
      .await
      .map_err(|e| e.into())
      .flatten()
      .into_napi()
  }
  #[napi]
  pub fn create_point_geometry(&self) -> napi::Result<Geometry> {
    napi_todo!()
  }
  /// Renders the `scene` from `camera`'s point of view
  /// Returns a memory-mapped region containing RGBA texture.
  #[napi]
  pub async fn render(&self, scene: &Scene, cam: Camera) -> napi::Result<Uint8Array> {
    let mut backend = self.backend.write().await;
    let projection = match cam.projection {
      Projection::Ortho {
        left,
        right,
        bottom,
        top,
        near,
        far,
      } => Mat4::orthographic_lh(
        left as f32,
        right as f32,
        bottom as f32,
        top as f32,
        near as f32,
        far as f32,
      ),
      Projection::Perspective {
        fovy,
        aspect,
        z_near,
        z_far,
      } => Mat4::perspective_lh(fovy as f32, aspect as f32, z_near as f32, z_far as f32),
    };
    let view = glam::Mat4::look_at_rh(
      Vec3::from_slice(&cam.location),
      Vec3::from_slice(&cam.look_at),
      (Vec3::Y),
    );
    let data = backend.render(
      &scene.0,
      &rasterizer::Camera {
        camera: view,
        projection,
      },
    );
    unsafe {
      Ok(Uint8Array::with_external_data(
        data.as_ptr().cast_mut(),
        data.len(),
        |_, _| log::info!("slice is utilized?"),
      ))
    }
  }

  /// Creates an instance of `Strahl` with default settings
  #[napi(factory, js_name = "default")]
  pub async fn default_node_workaround() -> napi::Result<Strahl> {
    Self::create(StrahlCreateInfo::default()).await
  }
}

/// The graphics backend implementation used by strahl
#[napi]
pub enum StrahlBackend {
  Rasterize,
}

/// Object aggregating the information required to create `Strahl`.
/// New fields (and will) be added in future.
#[napi(object)]
pub struct StrahlCreateInfo {
  /// Specifies the rendering backend
  pub backend: StrahlBackend,
  /// Specifies the width of the viewport (in pixels)
  pub width: u32,
  /// Specified the height of the viewport
  pub height: u32,
}

#[napi]
impl Default for StrahlCreateInfo {
  fn default() -> Self {
    Self {
      backend: StrahlBackend::Rasterize,
      width: 1024,
      height: 1024,
    }
  }
}
