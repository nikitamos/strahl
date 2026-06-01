use std::sync::Arc;

use bsdf::BSDF;
use medium::Medium;

pub mod bsdf;
pub mod medium;

pub trait Material: Send + Sync {
  fn medium(&self) -> &dyn Medium;
  fn bsdf(&self) -> &dyn BSDF;
}

pub struct TypeErasedMaterial {
  bsdf:   Arc<dyn BSDF>,
  medium: Arc<dyn Medium>,
}

impl TypeErasedMaterial {
  pub fn new(bsdf: Arc<dyn BSDF>, medium: Arc<dyn Medium>) -> Self { Self { bsdf, medium } }
}

impl Material for TypeErasedMaterial {
  fn medium(&self) -> &dyn Medium { self.medium.as_ref() }

  fn bsdf(&self) -> &dyn BSDF { self.bsdf.as_ref() }
}

pub trait TypedMaterial {
  type FixedMedium: Medium;
  type FixedBSDF: BSDF;

  fn medium(&self) -> &Self::FixedMedium
  where Self: Sized;

  fn bsdf(&self) -> &Self::FixedBSDF
  where Self: Sized;
}

pub struct ConcreteMaterial<M, B>
where
  M: Medium + Send + Sync,
  B: BSDF + Send + Sync,
{
  pub medium: M,
  pub bsdf:   B,
}

impl<M, B> TypedMaterial for ConcreteMaterial<M, B>
where
  M: Medium + Send + Sync,
  B: BSDF + Send + Sync,
{
  type FixedMedium = M;
  type FixedBSDF = B;

  fn medium(&self) -> &Self::FixedMedium
  where Self: Sized {
    &self.medium
  }

  fn bsdf(&self) -> &Self::FixedBSDF
  where Self: Sized {
    &self.bsdf
  }
}

impl<M, B> Material for ConcreteMaterial<M, B>
where
  B: bsdf::BSDF + Send + Sync,
  M: Medium + Send + Sync,
{
  fn medium(&self) -> &dyn Medium { &self.medium }

  fn bsdf(&self) -> &dyn BSDF { &self.bsdf }
}
