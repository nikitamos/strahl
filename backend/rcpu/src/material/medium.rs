pub trait Medium: Send + Sync {}

pub struct UniformMedium {
  pub ior: f32,
}

impl Medium for UniformMedium {}

#[derive(Clone, Copy)]
pub struct MediumInterface<'e, 'i> {
  pub from: &'e dyn Medium,
  pub to:   &'i dyn Medium,
}
