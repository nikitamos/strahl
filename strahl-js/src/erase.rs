#[macro_export]
macro_rules! eraser {
  ($name:ident($base_trait:path): $($types:ident,)+) => {
    enum $name {
      $($types (::std::boxed::Box<$types>),)*
    }
    $(
      impl From<$types> for $name {
      fn from(value: $types) -> Self {
        Self::$types(::std::boxed::Box::new(value))
      }
    })*
    impl ::std::ops::Deref for $name {
      type Target = dyn $base_trait;
      fn deref(&self) -> &Self::Target {
        match self {
          $(
          Self::$types(x) => x.as_ref(),
          )*
        }
      }
    }
    $(
    impl $name {
      pub fn $types(&self) -> Option<&$types> {
        self.into()
      }
    }

    impl From<$name> for Option<$types> {
      fn from(value: $name) -> Self {
        if let $name::$types(x) = value {
          Some(*x)
        } else {
          None
        }
      }
    }

    impl<'a> From<&'a $name> for Option<&'a $types> {
      fn from(value: &'a $name) -> Self {
        if let $name::$types(x) = value {
          Some(x.as_ref())
        } else {
          None
        }
      }
    }

    impl<'a> From<&'a mut $name> for Option<&'a mut $types> {
      fn from(value: &'a mut $name) -> Self {
        if let $name::$types(x) = value {
          Some(x.as_mut())
        } else {
          None
        }
      }
    }
  )*
  };
}
