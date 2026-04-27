use glam::Vec3;

use crate::{Castable, PointGlobal, Spectrum, VecGlobal};

/// Represents a ray with an origin and direction.
#[derive(Debug, Clone, Default)]
pub struct CameraRay {
  pub origin:    PointGlobal,
  /// Current direction (in global coordinates?)
  // TODO: replace with VecGlobal
  pub direction: Vec3,
  /// Recorded color
  pub color:     Spectrum,
  camera_dir:    VecGlobal,
}

impl CameraRay {
  pub fn reset_direction(&mut self) { self.direction = self.camera_dir.into(); }
  pub fn new(origin: PointGlobal, direction: VecGlobal) -> Self {
    Self {
      origin,
      direction: direction.into(),
      color: Default::default(),
      camera_dir: direction.into(),
    }
  }
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

  pub fn write_image(&self, img: &mut image::Rgb32FImage) {
    for x in 0..self.resolution.x {
      for y in 0..self.resolution.y {
        let color = self.rays[self.resolution.x * y + x].color;
        img.put_pixel(x as u32, y as u32, [color.x, color.y, color.z].into());
      }
    }
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
          origin:     point.into(),
          direction:  ray_direction,
          camera_dir: ray_direction.into(),
          ..Default::default()
        });
      }
    }

    &mut self.rays
  }

  pub fn resolution(&self) -> glam::USizeVec2 { self.resolution }
}
