#![feature(assert_matches)]

use std::sync::Arc;

mod points;
use glam::{Mat4, Quat, USizeVec2, Vec3};
pub use points::*;
mod sampling;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
pub use sampling::*;
mod geometry;
pub use geometry::*;

use crate::camera::CameraRay;
pub mod camera;

pub mod material;

#[cfg(test)]
mod test;

#[macro_export]
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

impl RayTracer {
  pub fn new() -> Self { Self {} }
  pub fn create_scene(&self) -> Scene { Scene::new() }
  pub fn create_solver(&self) -> Solver { Solver {} }
}

pub struct Solver {}
impl Solver {
  pub fn render(&self, scene: &Scene, cam: &mut camera::Camera) {
    let rays = cam.init_rays();
    rays.into_par_iter().enumerate().for_each(|(i, x)| {
      if let Some(intr) = Self::closest_hit(scene, x) {
        println!(
          "hit @ {:?}",
          intr.hit.transform.p2world(intr.hit.point)
        )
        // TODO
        // Sample BSDF
        // cast a new ray
      }
    });
  }
  fn closest_hit<'a>(scene: &'a Scene, r: &mut CameraRay) -> Option<Interaction<'a>> {
    scene
      .bodies
      .iter()
      .filter_map(|b| {
        let ctx = IntersectionContext {
            transform: b.transform(),
        };
        b.geometry
          .try_intersect(ctx, r)
          .map(|hit| Interaction { body: b, hit })
      })
      .min_by(|a, b| a.hit.ray_distance.partial_cmp(&b.hit.ray_distance).unwrap())
  }
}

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
  pub normal:       Vec3,
  pub ray_distance: f32,
  pub transform:    Transform,
}

impl SurfaceHit {
  pub fn point_global(&self) -> PointGlobal { self.transform.p2world(self.point) }
}

pub struct Interaction<'a> {
  pub hit:  SurfaceHit,
  pub body: &'a Body,
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

/// For now each point of the source emits light in direction normal
/// to the surface
pub struct LightSource {
  geometry: Option<Arc<dyn Geometry>>,
}

pub struct Scene {
  pub(crate) bodies: Vec<Body>,
  // Why does scene at all stores its cameras?
  cameras:           Vec<Arc<camera::Camera>>,
}

impl Scene {
  pub fn new() -> Self {
    Scene {
      bodies:  vec![],
      cameras: vec![],
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
      material: Arc::new(material::bsdf::lambertian::Lambertian { s: Vec3::X }),
      pos:      Default::default(),
      rotation: Quat::IDENTITY,
    });
    self.bodies.last().unwrap()
  }
}
