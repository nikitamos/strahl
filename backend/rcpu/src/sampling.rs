use rand::seq::IndexedRandom;
use rand_distr::{Distribution, Uniform};

use crate::VecHit;
use std::f32::consts::{FRAC_1_PI, TAU};

pub struct Sampler {
  x: Uniform<f32>,
  y: Uniform<f32>,
  c: Uniform<f32>,
}

impl Default for Sampler {
  fn default() -> Self { Self::new() }
}

impl Sampler {
  pub fn new() -> Self {
    Self {
      x: Uniform::new(0.0, 1.0).unwrap(),
      y: Uniform::new(0.0, 1.0).unwrap(),
      c: Uniform::new(0.0, 1.0).unwrap(),
    }
  }
  pub fn sample(&'_ self) -> SampleState<'_> {
    // TODO: better generation of vectors
    let mut rng = rand::rng();
    SampleState {
      uniform_1d: self.c.sample(&mut rng),
      uniform_2d: glam::vec2(self.x.sample(&mut rng), self.y.sample(&mut rng)),
      producer:   self,
    }
  }
  /// Uniformly samples an element from non-empty slice.
  /// # Panics
  /// Panics if the `slice` is empty.
  pub fn sample_element<'a, T>(&self, slice: &'a [T]) -> Sample<&'a T> {
    let mut rng = rand::rng();
    Sample {
      sample:   slice.choose(&mut rng).unwrap(),
      prob:     1.0 / (slice.len() as f32),
      metadata: (),
    }
  }
}

pub struct SampleState<'a> {
  pub uniform_1d: f32,
  pub uniform_2d: glam::Vec2,
  pub producer:   &'a Sampler,
}

// TODO: evaluate the possibility of extracting each distribution into a separate
// class allowing to combine and invert them
impl<'a> SampleState<'a> {
  /// Samples a uniformly distributed point within the unit disk $B^2$.
  pub fn disk_uniform(self) -> Sample<glam::Vec2, &'a Sampler> {
    let phi = TAU * self.uniform_2d.y;
    let r = self.uniform_2d.x.sqrt();
    self.make_sample(r * glam::vec2(phi.cos(), phi.sin()), FRAC_1_PI)
  }
  /// Samples a uniformly distributed point within the upper hemisphere $\mathcal{H}^2$.
  pub fn hemisphere_uniform(self) -> Sample<VecHit, &'a Sampler> {
    let z = self.uniform_2d.x;
    let r = (1.0 - z * z).sqrt();
    let phi = TAU * self.uniform_2d.y;
    self.make_sample(
      glam::vec3(r * phi.cos(), r * phi.sin(), z).into(),
      1.0 / TAU,
    )
  }
  /// Samples a point distributed within the upper hemisphere $\mathcal{H}^2$
  /// according to the cosine-weighted distribution, i.e.
  /// $$ p(X=(\theta, \phi)) = \frac{1}{\pi} \cos\theta \sin\theta $$
  pub fn hemisphere_cosine(self) -> Sample<VecHit, &'a Sampler> {
    self
      .disk_uniform()
      .map(|d| d.extend((1.0 - d.length_squared()).sqrt()).into())
  }
  /// Samples a uniformly distributed point within the $\mathcal{S}^2$.
  pub fn sphere_uniform(self) -> Sample<VecHit, &'a Sampler> {
    let z = 1.0 - 2.0 * self.uniform_2d.x;
    let r = (1.0 - z * z).sqrt();
    let phi = TAU * self.uniform_2d.y;
    self.make_sample(
      glam::vec3(r * phi.cos(), r * phi.sin(), z).into(),
      0.5 / TAU,
    )
  }
  /// Returns a sample of given value with given probability, additionally
  /// injecting the [`Self::producer`] as metadata.
  fn make_sample<T>(self, sample: T, prob: f32) -> Sample<T, &'a Sampler> {
    Sample {
      prob,
      sample,
      metadata: self.producer,
    }
  }
}

/// Result of sampling operation
pub struct Sample<T, M = ()> {
  /// Probability of sampling the `sample` value
  pub prob:     f32,
  /// The sampled value
  pub sample:   T,
  /// The data associated with the sampling result. It may be used to store
  /// operation-specific information.
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
  pub fn map_all<U, N>(self, mapper: impl Fn(T, M) -> (U, N)) -> Sample<U, N> {
    let (sample, metadata) = mapper(self.sample, self.metadata);
    Sample {
      prob: self.prob,
      sample,
      metadata,
    }
  }
  
  pub fn and_then<U, N>(self, mapper: impl FnOnce(T, M) -> Option<Sample<U, N>>) -> Option<Sample<U, N>> {
    let mut m = mapper(self.sample, self.metadata)?;
    m.prob *= self.prob;
    Some(m)
  }
  pub fn replace<U>(self, replacement: U) -> Sample<U, M> {
    Sample {
      prob:     self.prob,
      sample:   replacement,
      metadata: self.metadata,
    }
  }
  /// Discards the metadata and replaces it with `()`
  pub fn discard_metadata(self) -> Sample<T, ()> { self.with_metadata(()) }
  /// Discards metadata and replaces it with the value provided
  pub fn with_metadata<N>(self, metadata: N) -> Sample<T, N> {
    Sample {
      prob:     self.prob,
      sample:   self.sample,
      metadata,
    }
  }

  /// Consumes `self` and returns stored sample
  pub fn into_inner(self) -> T { self.sample }
}

// TODO: determine scope of the trait
trait Samplable<S> {
  fn sample<'a>(&self, ctx: SampleState<'a>) -> Sample<S>;
}
