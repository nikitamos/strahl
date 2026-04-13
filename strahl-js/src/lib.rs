#![deny(clippy::all)]

use napi::{Env, bindgen_prelude::ArrayBuffer};
use napi_derive::napi;

#[napi]
pub struct Strahl {
  backend: (),
}

#[napi]
pub struct Material {}

#[napi]
pub struct Geometry {}

#[napi]
pub struct Scene {}

#[napi]
pub struct Body {
  material: Material,
  geometry: Geometry,
}

#[napi(object)]
pub struct BodyCreateInfo<'env> {
  pub position: ArrayBuffer<'env>,
}

#[napi]
impl Scene {
  #[napi]
  pub fn add_body<'env>(
    &mut self,
    _env: &'env Env,
    material: &'env Material,
    geometry: &'env Geometry,
    info: BodyCreateInfo<'env>,
  ) {
    todo!()
  }
}

#[napi]
impl Strahl {
  #[napi(constructor)]
  pub fn new(info: StrahlCreateInfo) -> Self {
    todo!()
  }

  pub fn create_scene(&self) -> Scene {
    todo!()
  }

  #[napi]
  pub fn load_material(&self, path: String) -> Material {
    Material {}
  }

  #[napi]
  pub fn load_model(&self, path: String) -> Geometry {
    Geometry {}
  }
}

#[napi]
pub enum StrahlBackend {
  Rasterize,
}

#[napi(object)]
pub struct StrahlCreateInfo {
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
