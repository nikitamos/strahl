use std::sync::Arc;

use crate::{
  geometry::Geometry,
  material::Material,
};

pub struct BodyPipeline {
  pipeline: wgpu::RenderPipeline,
}

impl Default for BodyPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl BodyPipeline {
  pub fn new() -> Self { todo!() }
}

pub struct Body {
  geometry: Geometry,
  material: Material,
  pipeline: BodyPipeline,
}

impl Body {
  pub fn new(geometry: Geometry, material: Material) -> Self {
    Self {
      geometry,
      material,
      pipeline: BodyPipeline::new(),
    }
  }
}

pub struct Scene {
  bodies: Vec<Arc<Body>>,
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
  pub fn new() -> Self { Self { bodies: vec![] } }
  pub fn add_body(&mut self, geometry: Geometry, material: Material) -> Arc<Body> {
    Arc::new(Body::new(geometry, material))
  }
  pub(crate) fn bodies(&self) -> &[Arc<Body>] { &self.bodies }
}
