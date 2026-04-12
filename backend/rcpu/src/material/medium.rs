pub trait Medium {}

pub struct UniformMedium {
  pub ior: f32,
}

impl Medium for UniformMedium {}

pub struct MediumInterface<'e, 'i> {
  pub from: &'e dyn Medium,
  pub to:   &'i dyn Medium,
}
