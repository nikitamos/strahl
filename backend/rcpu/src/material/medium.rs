use std::marker::PhantomData;

pub trait Medium: Send + Sync {
  fn ior(&self) -> f32;
  fn interface<'a, 'b>(&'a self, next: &'b dyn Medium) -> MediumInterface<'a, 'b> {
    MediumInterface::with_relative_ior(self.ior() / next.ior())
  }
}

pub struct UniformMedium {
  pub ior: f32,
}

impl Medium for UniformMedium {
  fn ior(&self) -> f32 { self.ior }
}

#[derive(Clone, Copy)]
pub struct MediumInterface<'e, 'i> {
  pub relative_ior: f32,
  _p1:              PhantomData<&'e ()>,
  _p2:              PhantomData<&'i ()>,
}

impl<'e, 'i> MediumInterface<'e, 'i> {
  pub fn new(from: &'e dyn Medium, to: &'i dyn Medium) -> Self {
    Self::with_relative_ior(from.ior() / to.ior())
  }
  pub fn with_relative_ior(ior: f32) -> Self {
    MediumInterface {
      relative_ior: ior,
      _p1:          Default::default(),
      _p2:          Default::default(),
    }
  }
}
