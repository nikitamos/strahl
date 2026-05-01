pub trait RtBackend {}

pub trait SceneNode {}

pub trait Camera {}

pub trait Body {}

pub trait Geometry {}

pub trait Material {}

pub trait Texture {}

pub trait Mesh {}

pub trait Scene {}

pub trait Renderer {}

pub mod rt;

#[macro_export]
macro_rules! with {
  ($x:ident: $($($fields:ident).* = $val: expr), *) => {
      {
        let mut y = $x;
        $(y$(.$fields)* = $val;)*
        y
      }
  };
  ($x:expr => $($($fields:ident).* = $val: expr), *) => {
      {
        let mut y = $x;
        // TODO: Reuse arm #0
        $(y$(.$fields)* = $val;)*
        y
      }
  };
}
