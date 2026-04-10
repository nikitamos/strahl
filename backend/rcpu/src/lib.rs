#![feature(assert_matches)]

use std::sync::{Arc, RwLock};

mod points;
use glam::{Mat4, Quat, USizeVec2, Vec3};
pub use points::*;
mod sampling;
use rayon::iter::{
  IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
};
pub use sampling::*;
mod geometry;
pub use geometry::*;

use crate::camera::CameraRay;
pub mod camera;

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
          intr.hit.point.into_global(intr.body.l2w_matrix())
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
          g2l: b.w2l_matrix(),
        };
        b.geometry
          .try_intersect(ctx, r)
          .map(|hit| Interaction { hit, body: b })
      })
      .min_by(|a, b| a.hit.ray_distance.partial_cmp(&b.hit.ray_distance).unwrap())
  }
}

trait Material: Send + Sync {}
type Spectrum = Vec3;
#[repr(transparent)]
struct Lambertian {
  pub s: Spectrum,
}
impl Material for Lambertian {}

/// Represents a castable ray, usually originating from camera or light source.
pub trait Castable {
  /// Current position in global coordinates
  fn pos(&self) -> PointGlobal;
  /// Current direction in **global** coordinates
  fn direction(&self) -> glam::Vec3;
}
#[derive(Debug)]
pub struct SurfaceHit {
  pub point:        PointLocal,
  pub normal:       Vec3,
  pub ray_distance: f32,
}

pub struct Interaction<'a> {
  pub hit:  SurfaceHit,
  pub body: &'a Body,
}

pub struct IntersectionContext {
  g2l: glam::Mat4,
}

// #[derive(Default)]
pub struct Body {
  geometry: Arc<dyn Geometry>,
  material: Arc<dyn Material>,
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
      material: Arc::new(Lambertian { s: Vec3::X }),
      pos:      Default::default(),
      rotation: Quat::IDENTITY,
    });
    self.bodies.last().unwrap()
  }
}
