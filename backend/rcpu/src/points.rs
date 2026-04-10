use std::ops::Deref;

use glam::Vec4Swizzles;

#[repr(transparent)]
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct PointLocal(glam::Vec4);

impl Deref for PointLocal {
  type Target = glam::Vec4;

  fn deref(&self) -> &Self::Target { &self.0 }
}

impl PointLocal {
  pub const fn new(v: glam::Vec4) -> Self { Self(v) }
  pub fn into_global(self, l2g: glam::Mat4) -> PointGlobal { PointGlobal(l2g * self.0) }
}

impl From<PointLocal> for glam::Vec4 {
  fn from(value: PointLocal) -> Self { value.0 }
}

impl From<PointLocal> for glam::Vec3 {
  fn from(value: PointLocal) -> Self { value.0.xyz() }
}

impl From<glam::Vec3> for PointLocal {
  fn from(value: glam::Vec3) -> Self { PointLocal(glam::vec4(value.x, value.y, value.z, 1.0)) }
}

#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct PointGlobal(glam::Vec4);

impl Deref for PointGlobal {
  type Target = glam::Vec4;

  fn deref(&self) -> &Self::Target { &self.0 }
}

impl PointGlobal {
  pub const fn new(v: glam::Vec4) -> Self { Self(v) }
  pub fn into_local(self, g2l: glam::Mat4) -> PointLocal { PointLocal(g2l * self.0) }
}

impl From<PointGlobal> for glam::Vec4 {
  fn from(value: PointGlobal) -> Self { value.0 }
}

impl From<PointGlobal> for glam::Vec3 {
  fn from(value: PointGlobal) -> Self { value.0.xyz() }
}

impl From<glam::Vec3> for PointGlobal {
  fn from(value: glam::Vec3) -> Self { PointGlobal(glam::vec4(value.x, value.y, value.z, 1.0)) }
}
