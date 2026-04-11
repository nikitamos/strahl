use crate::{Spectrum, material::Material};

#[repr(transparent)]
#[derive(Debug)]
pub struct Lambertian {
  pub s: Spectrum,
}

impl Material for Lambertian {}
