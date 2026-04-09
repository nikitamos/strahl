use std::sync::{Arc, RwLock};

mod points;
use glam::{Mat4, USizeVec2, Vec3};
pub use points::*;
mod sampling;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
pub use sampling::*;
mod geometry;
pub use geometry::*;

use crate::camera::CameraRay;
pub mod camera;

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
    rays
      .into_par_iter()
      .for_each(|x| Self::try_intersect_ray(scene, x));
  }
  fn try_intersect_ray(scene: &Scene, r: &mut CameraRay) {
    for ((geom, material), w2l) in scene.bodies.iter().filter_map(|b| {
      b.geometry
        .as_ref()
        .zip(b.material.as_ref())
        .zip(Some(b.get_w2l_matrix()))
    }) {
      let ctx = IntersectionContext { mat: w2l };
      if let Some(isect) = geom.try_intersect(ctx, r) {

        // TODO
      }
    }
  }
}

trait Material: Send + Sync {}

pub trait Castable {}
/// TODO: determine scope
pub trait Samplable<S> {
  fn sample(&self, ctx: sampling::SampleState) -> sampling::Sample<S>;
}
pub struct SurfaceIntersection {
  pub point:  PointLocal,
  pub normal: Vec3,
}
pub struct IntersectionContext {
  mat: glam::Mat4,
}

#[derive(Default)]
pub struct Body {
  geometry: Option<Arc<dyn Geometry>>,
  material: Option<Arc<dyn Material>>,
  pos:      PointGlobal,
  rotation: glam::Quat,
}

impl Body {
  pub fn get_w2l_matrix(&self) -> Mat4 {
    let mut res = Mat4::from_quat(self.rotation);
    res.w_axis = self.pos.into();
    res.w_axis *= -1f32;
    res.w_axis.w = 1.0;
    res
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
  pub fn add_sphere(&mut self) -> &Body {
    self.bodies.push(Body::default());
    self.bodies.last().unwrap()
  }
}
