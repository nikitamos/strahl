use glam::Vec3;

use crate::{Castable, PointGlobal, VecGlobal};

/// Represents a ray with an origin and direction.
#[derive(Debug, Clone, Default)]
pub struct CameraRay {
  origin:    PointGlobal,
  /// Current direction
  direction: Vec3,
}

impl Castable for CameraRay {
  fn pos(&self) -> PointGlobal { self.origin }

  fn direction(&self) -> VecGlobal { self.direction.into() }
}

/// Camera type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraType {
  Perspective,
  Orthographic,
}

pub struct Camera {
  resolution:  glam::USizeVec2,
  cam_type:    CameraType,
  right:       Vec3,
  direction:   Vec3,
  translation: PointGlobal,
  rays:        Vec<CameraRay>, // invariant: if non-empty, rays are valid for cam_type and resolution
}

impl Camera {
  pub fn new(
    resolution: glam::USizeVec2,
    direction: Vec3,
    right: Vec3,
    pos: PointGlobal,
    cam_type: CameraType,
  ) -> Self {
    assert!(
      resolution.x != 0 && resolution.y != 0,
      "resolution can't be zero"
    );
    Camera {
      resolution,
      cam_type,
      right,
      direction,
      translation: pos,
      rays: Vec::new(),
    }
  }

  /// Copies internally stored image to provided memory
  pub fn acquire_image(&mut self, _image: &mut [Vec3]) {
    unimplemented!("acquireImage is not defined in the original C++ code")
  }

  /// Initializes the ray cache if empty and returns a slice to the rays.
  pub(crate) fn init_rays(&mut self) -> &mut [CameraRay] {
    if !self.rays.is_empty() {
      return &mut self.rays;
    }

    let (width, height) = self.resolution.into();
    self.rays.reserve(width * height);

    let cam_pos: Vec3 = self.translation.into();
    let screen_center = cam_pos + self.direction;
    let screen_up = (self.right.cross(self.direction).normalize())
      * self.right.length()
      * (height as f32 / width as f32);

    let top_left = screen_center + screen_up - self.right;
    let x_step = 2.0 / width as f32 * self.right;
    let y_step = -2.0 / height as f32 * screen_up;

    for j in 0..height {
      for i in 0..width {
        let point = top_left + (i as f32) * x_step + (j as f32) * y_step;
        let ray_direction = (point - cam_pos).normalize();
        self.rays.push(CameraRay {
          origin:    point.into(),
          direction: ray_direction,
        });
      }
    }

    &mut self.rays
  }
}
