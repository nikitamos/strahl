use std::ops::{Deref, DerefMut, Neg, Sub};

use glam::Vec3;

macro_rules! vec_wrapper {
  {
   $(#[$outer:meta])*
   $name:ident => $underlying:ty $(; $($conv:ty,)*)?
  } => {
    #[repr(transparent)]
    #[derive(Debug, Default, Clone, Copy, PartialEq)]
    $(#[$outer])*
    pub struct $name($underlying);
    impl Deref for $name {
      type Target = $underlying;
      fn deref(&self) -> &Self::Target { &self.0 }
    }
    impl DerefMut for $name {
      fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
    }
    impl $name {
      pub const fn new(value: $underlying) -> Self { Self(value) }
    }
    impl From<$name> for $underlying {
      fn from(value: $name) -> $underlying { value.0 }
    }
    impl From<$underlying> for $name {
      fn from(value: $underlying) -> $name { Self(value) }
    }
    impl Neg for $name {
      type Output = $name;

      fn neg(self) -> Self::Output { (-*self).into() }
    }
    $($(
      impl From<$conv> for $name {
        fn from(value: $conv) -> Self{
          Self(value.into())
        }
      }
    )*)?
  };
}

vec_wrapper! {
  /// Type-safe wrapper around point in geometry space
  PointLocal => glam::Vec3
}

vec_wrapper! {
  /// Type-safe wrapper around point in world coordinates
  PointGlobal => glam::Vec3
}

vec_wrapper! {
  /// Type-safe wrapper around vector in geometry coordinates
  VecLocal => Vec3; glam::Vec3A,
}

vec_wrapper! {
  /// Type-safe wrapper around vector in world coordinates
  VecGlobal => Vec3; glam::Vec3A,
}

vec_wrapper! {
  /// Type-safe wrapper around vector in hit space
  VecHit => Vec3; glam::Vec3A,
}

impl Sub for PointGlobal {
  type Output = VecGlobal;

  fn sub(self, rhs: Self) -> Self::Output { (*self - *rhs).into() }
}
