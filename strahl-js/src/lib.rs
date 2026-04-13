#![deny(clippy::all)]

mod erase;

use std::sync::Arc;

use napi::{
  Env,
  bindgen_prelude::{ArrayBuffer, Float32Array, Uint8Array},
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

/// The entry point to the `strahl`'s API.
/// This class provides methods for creating scenes, managing resources and rendering.
#[napi]
pub struct Strahl {
  backend: (),
}

/// Class wrapping a geometry. So far it doesn't have any methods exposed,
/// so it may thought of as a kind of opaque handler.
///
/// Geometry in `strahl` is a generic shape that may be used by bodies or light sources.
#[napi]
pub struct Geometry {}

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
pub struct Material(Arc<()>);

/// Anything that can be rendered
#[napi]
pub struct Body {
  material: Arc<()>,
  geometry: Arc<()>,
  position: [f32; 3],
}

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
    Float32Array::with_data_copied(self.position)
  }
  /// Sets the position of the body relative to the world origin. Expects an array of length 3.
  #[napi(setter)]
  pub fn set_position(&mut self, pos: &[f32]) -> napi::Result<()> {
    if pos.len() != 3 {
      Err(napi::Error::from_reason("Expected a slice of length 3"))
    } else {
      self.position.copy_from_slice(pos);
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
pub struct Scene {}

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
  ) -> napi::Result<Body> {
    napi_todo!()
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
  #[napi(constructor)]
  pub fn new(info: StrahlCreateInfo) -> Self {
    Self { backend: () }
  }
  /// Creates an empty scene
  #[napi]
  pub fn create_scene(&self) -> Scene {
    Scene {}
  }
  /// Loads material from given path
  #[napi]
  pub fn load_material(&self, path: String) -> napi::Result<Material> {
    napi_todo!()
  }
  /// Loads model geometry from given path
  #[napi]
  pub fn load_model(&self, path: String) -> napi::Result<Geometry> {
    napi_todo!()
  }
  #[napi]
  pub fn create_point_geometry(&self) -> napi::Result<Geometry> {
    napi_todo!()
  }
  /// Renders the `scene` from `camera`'s point of view
  #[napi]
  pub async fn render(&self, scene: &Scene, cam: Camera) -> napi::Result<Uint8Array> {
    napi_todo!()
  }

  /// Creates an instance of `Strahl` with default settings
  #[napi(factory, js_name = "default")]
  pub fn default_node_workaround() -> Strahl {
    Self::default()
  }
}

impl Default for Strahl {
  fn default() -> Self {
    Strahl::new(StrahlCreateInfo::default())
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
}

#[napi]
impl Default for StrahlCreateInfo {
  fn default() -> Self {
    Self {
      backend: StrahlBackend::Rasterize,
    }
  }
}
