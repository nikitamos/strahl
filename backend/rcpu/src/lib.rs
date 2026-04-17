#![feature(assert_matches)]

use std::{marker::PhantomData, sync::Arc};

mod points;
use glam::{Mat4, Quat, USizeVec2, Vec3};
pub use points::*;
mod sampling;

pub use sampling::*;
mod geometry;
pub use geometry::*;

use crate::{
  light::LightSource,
  material::{ConcreteMaterial, bsdf::lambertian::Lambertian, medium::UniformMedium},
};
pub mod camera;

pub mod material;

#[cfg(test)]
mod test;

#[macro_export]
/// # Partial update of struct
/// This macro allows partial update of structures, i.e. mutating
/// values of one or more of its fields while keeping others unchanged.
///
/// Note that macro consumes the original value.
/// ```
/// #![feature(assert_matches)]
/// use std::assert_matches::assert_matches;
/// use rcpu::with;
///
/// #[derive(Debug)]
/// struct Point {
///   pub x: f32,
///   pub y: f32,
/// }
///
/// #[derive(Debug)]
/// struct Rectangle {
///   pub top_left:     Point,
///   pub bottom_right: Point,
/// }
///
/// let point = Point { x: 12.0, y: -0.3 };
/// let point = with!(point: x = 5.0); // set x to be 5.0
/// assert_matches!(point, Point {x: 5.0, y: -0.3});
///
/// let rect = Rectangle {top_left: point, bottom_right: Point {x: 6.0, y: -1.0} };
/// // Mutating nested fields is done with the `=>` syntax.
/// let rect = with!(rect => bottom_right.y = 0.0, top_left.y = 2.0);
/// assert_matches!(rect, Rectangle { top_left: Point {x: 5.0, y: 2.0 }, bottom_right: Point{x: 6.0, y: 0.0}});
/// ```
macro_rules! with {
  ($x:ident: $($($fields:ident).* = $val: expr), *) => {
      {
        let mut y = $x;
        $(y$(.$fields)* = $val;)*
        y
      }
  };
  ($x:expr => $($($fields:ident).* = $val: expr), *) => {
      {
        let mut y = $x;
        // TODO: Reuse arm #0
        $(y$(.$fields)* = $val;)*
        y
      }
  };
}

pub struct RayTracer {}

impl Default for RayTracer {
  fn default() -> Self { Self::new() }
}

impl RayTracer {
  pub fn new() -> Self { Self {} }
  pub fn create_scene(&self) -> Scene { Scene::new() }
  pub fn create_solver(&self) -> solver::Solver { solver::Solver::new() }
  pub fn create_sphere(&self, radius: f32) -> Arc<Sphere> { Arc::new(Sphere { radius }) }
}

mod solver;

type Spectrum = Vec3;

/// Represents a castable ray, usually originating from camera or light source.
pub trait Castable {
  /// Current position
  fn pos(&self) -> PointGlobal;
  /// Current direction
  fn direction(&self) -> VecGlobal;
}
#[derive(Debug)]
pub struct SurfaceHit {
  pub point:        PointLocal,
  /// Surface normal in local coordinates
  pub normal:       Vec3,
  pub ray_distance: f32,
  /// Transform from global coordinates to the coordinates of the hit body.
  /// Usually is taken from [`IntersectionContext::transform`]
  pub transform:    Transform,
}

impl SurfaceHit {
  pub fn point_global(&self) -> PointGlobal { self.transform.p2world(self.point) }
}

pub struct Interaction<'a> {
  pub hit:      SurfaceHit,
  pub body:     &'a Body,
  /// Normalized ray vector direction, pointing to the surface
  pub incoming: VecLocal,
}

pub struct IntersectionContext {
  transform: Transform,
}

/// Transformation of coordinates
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Transform {
  /// Transformation of world coordinates to local
  pub w2l: Mat4,
  /// Transformation of local coordinates to local
  pub l2w: Mat4,
}

impl Transform {
  /// Constructs Self from world-to-local transformation
  pub fn from_w2l(w2l: Mat4) -> Self {
    Self {
      w2l,
      l2w: w2l.inverse(),
    }
  }
  pub fn p2world(&self, l: PointLocal) -> PointGlobal { self.l2w.transform_point3(l.into()).into() }
  pub fn v2world(&self, l: VecLocal) -> VecGlobal { self.l2w.transform_vector3(l.into()).into() }
  pub fn p2local(&self, g: PointGlobal) -> PointLocal { self.w2l.transform_point3(g.into()).into() }
  pub fn v2local(&self, g: VecGlobal) -> VecLocal { self.w2l.transform_vector3(g.into()).into() }
}

// #[derive(Default)]
pub struct Body {
  geometry: Arc<dyn Geometry>,
  material: Arc<dyn material::Material>,
  pos:      PointGlobal,
  rotation: glam::Quat,
}

impl Body {
  /// Returns matrix representing transform from world coordinates to local
  pub fn w2l_matrix(&self) -> Mat4 {
    let mut res = Mat4::from_quat(self.rotation);
    res.w_axis = self.pos.into();
    res.w_axis *= -1f32;
    res.w_axis.w = 1.0;
    res
  }
  /// Returns matrix representing transform from local coordinates to world
  pub fn l2w_matrix(&self) -> Mat4 { self.w2l_matrix().inverse() }
  /// Returns [`Transform`] that maps between local and world coordinates
  pub fn transform(&self) -> Transform {
    Transform {
      w2l: self.w2l_matrix(),
      l2w: self.l2w_matrix(),
    }
  }
}

pub mod light;

pub struct Scene {
  // TODO: RWLock/mutex on body or lights?
  pub(crate) bodies: Vec<Body>,
  pub(crate) lights: Vec<LightSource>,
  // Why does scene at all stores its cameras?
  cameras:           Vec<Arc<camera::Camera>>,
}

impl Default for Scene {
  fn default() -> Self { Self::new() }
}

impl Scene {
  pub fn new() -> Self {
    Scene {
      bodies:  vec![],
      cameras: vec![],
      lights:  vec![],
    }
  }

  pub fn create_camera(
    &mut self,
    resolution: USizeVec2,
    direction: Vec3,
    right: Vec3,
    pos: PointGlobal,
    cam_type: camera::CameraType,
  ) -> camera::Camera {
    camera::Camera::new(resolution, direction, right, pos, cam_type)
  }
  pub fn add_sphere(&mut self, radius: f32) -> &Body {
    self.bodies.push(Body {
      geometry: Arc::new(Sphere { radius }),
      material: Arc::new(ConcreteMaterial {
        bsdf:   Lambertian { s: Vec3::X },
        medium: UniformMedium { ior: 1.0 },
      }),
      pos:      Default::default(),
      rotation: Quat::IDENTITY,
    });
    self.bodies.last().unwrap()
  }
  pub fn add_light(
    &mut self,
    geometry: Arc<dyn Geometry>,
    spectrum: SurfaceProperty<Spectrum>,
    dir: light::LightEmissionDirection,
  ) -> &LightSource {
    self.lights.push(LightSource::new(
      geometry,
      spectrum,
      2.0 * glam::vec3(1.0, 1.0, 0.4),
      dir,
    ));
    self.lights.last().unwrap()
  }
  /// Samples a light source present on the scene.
  /// For now the sampling is uniform, that is, each light source has
  /// equal probability to be sampled.
  pub fn sample_light_source(&self, sampler: &Sampler, _dest: PointGlobal) -> Sample<&LightSource> {
    sampler.sample_element(&self.lights)
  }
}

pub struct Texture<T>(PhantomData<T>);
pub enum SurfaceProperty<T> {
  /// The same value for all points of the surface
  Uniform(T),
  /// The value for each point is read from the texture
  Texture(Texture<T>),
}

impl<T> SurfaceProperty<T>
where T: Clone
{
  pub fn get(&self) -> T {
    match self {
      SurfaceProperty::Uniform(x) => x.clone(),
      SurfaceProperty::Texture(_texture) => unimplemented!(),
    }
  }
}
