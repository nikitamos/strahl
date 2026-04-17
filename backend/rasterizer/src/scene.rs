use std::sync::{Arc, RwLock};

use crate::{geometry::Geometry, material::Material};

#[repr(C)]
#[derive(Clone, zerocopy::IntoBytes, zerocopy::KnownLayout, zerocopy::Immutable)]
pub struct BodyData {
  pub rotation:    glam::Quat,
  pub mat:         glam::Mat4,
  pub translation: glam::Vec3,
  pub scale:       f32,
}

pub struct Body {
  geometry: Arc<Geometry>,
  material: Arc<Material>,
  // TODO: determine which RwLock should be used hare
  data:     RwLock<BodyData>,
}

impl Body {
  pub fn new(geometry: Arc<Geometry>, material: Arc<Material>) -> Self {
    Self {
      geometry,
      material,
      data: RwLock::new(BodyData {
        scale:       1.0,
        mat:         glam::Mat4::IDENTITY,
        rotation:    glam::Quat::IDENTITY,
        translation: glam::Vec3::ZERO,
      }),
    }
  }

  pub fn rotation(&self) -> glam::Quat { self.data.read().unwrap().rotation }
  pub fn set_rotation(&self, rotation: glam::Quat) {
    self.data.write().unwrap().rotation = rotation;
  }
  pub fn translation(&self) -> glam::Vec3 { self.data.read().unwrap().translation }
  pub fn set_translation(&self, translation: glam::Vec3) {
    self.data.write().unwrap().translation = translation;
  }
  pub fn scale(&self) -> f32 { self.data.read().unwrap().scale }
  pub fn set_scale(&self, scale: f32) { self.data.write().unwrap().scale = scale; }

  pub fn material(&self) -> &Material { &self.material }
  pub fn geometry(&self) -> &Geometry { &self.geometry }
}

pub struct Scene {
  bodies: Vec<Arc<Body>>,
}

impl Default for Scene {
  fn default() -> Self { Self::new() }
}

impl Scene {
  pub fn new() -> Self { Self { bodies: vec![] } }
  pub fn add_body(&mut self, geometry: Arc<Geometry>, material: Arc<Material>) -> Arc<Body> {
    let res = Arc::new(Body::new(geometry, material));
    self.bodies.push(res.clone());
    res
  }
  pub(crate) fn bodies(&self) -> &[Arc<Body>] { &self.bodies }
}
