pub struct SampleState {
  pub u:  f32,
  pub uc: glam::Vec2,
}

pub struct Sample<T, M = ()> {
  pub prob:     f32,
  pub sample:   T,
  pub metadata: M,
}

impl<T, M> Sample<T, M> {
  pub fn map<U>(self, mapper: impl Fn(T) -> U) -> Sample<U, M> {
    Sample {
      prob:     self.prob,
      sample:   mapper(self.sample),
      metadata: self.metadata,
    }
  }
}
