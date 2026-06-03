use std::sync::Arc;

use bsdf::BSDF;
use medium::Medium;

pub mod bsdf;
pub mod medium;

pub trait Material: Send + Sync {
  fn medium(&self) -> &Medium;
  fn bsdf(&self) -> &dyn BSDF;
}

pub struct TypeErasedMaterial {
  bsdf:   Arc<dyn BSDF>,
  medium: Arc<Medium>,
}

impl TypeErasedMaterial {
  pub fn new(bsdf: Arc<dyn BSDF>, medium: Arc<Medium>) -> Self { Self { bsdf, medium } }
}

impl Material for TypeErasedMaterial {
  fn medium(&self) -> &Medium { self.medium.as_ref() }

  fn bsdf(&self) -> &dyn BSDF { self.bsdf.as_ref() }
}

pub trait TypedMaterial {
  type FixedBSDF: BSDF;

  fn medium(&self) -> &Medium;

  fn bsdf(&self) -> &Self::FixedBSDF
  where Self: Sized;
}

pub struct ConcreteMaterial<B>
where B: BSDF + Send + Sync
{
  pub medium: Medium,
  pub bsdf:   B,
}

impl<B> TypedMaterial for ConcreteMaterial<B>
where B: BSDF + Send + Sync
{
  type FixedBSDF = B;

  fn medium(&self) -> &Medium {
    &self.medium
  }

  fn bsdf(&self) -> &Self::FixedBSDF
  where Self: Sized {
    &self.bsdf
  }
}

impl<B> Material for ConcreteMaterial<B>
where B: bsdf::BSDF + Send + Sync
{
  fn medium(&self) -> &Medium { &self.medium }
  fn bsdf(&self) -> &dyn BSDF { &self.bsdf }
}
