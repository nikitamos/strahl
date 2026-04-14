#[derive(serde::Deserialize, serde::Serialize)]
pub struct MaterialTextures<T> {
  pub roughness: Option<T>,
  pub specular:  Option<T>,
  pub glossy:    Option<T>,
  pub diffuse:   Option<T>,
  pub emission:  Option<T>,
  pub normal:    Option<T>,
}

impl<T> Default for MaterialTextures<T> {
  fn default() -> Self {
    Self {
      roughness: Default::default(),
      specular:  Default::default(),
      glossy:    Default::default(),
      diffuse:   Default::default(),
      emission:  Default::default(),
      normal:    Default::default(),
    }
  }
}

impl<T> MaterialTextures<T> {
  pub fn map_all<U>(mut self, mapper: impl Fn(T) -> U) -> MaterialTextures<U> {
    MaterialTextures {
      roughness: self.roughness.take().map(&mapper),
      specular:  self.specular.take().map(&mapper),
      glossy:    self.glossy.take().map(&mapper),
      diffuse:   self.diffuse.take().map(&mapper),
      emission:  self.emission.take().map(&mapper),
      normal:    self.normal.take().map(&mapper),
    }
  }
  pub fn take(&mut self) -> Self {
    MaterialTextures {
      roughness: self.roughness.take(),
      specular:  self.specular.take(),
      glossy:    self.glossy.take(),
      diffuse:   self.diffuse.take(),
      emission:  self.emission.take(),
      normal:    self.normal.take(),
    }
  }
  pub fn map_named<U>(mut self, mut mapper: impl FnMut(&str, T) -> U) -> MaterialTextures<U> {
    dbg!(self.roughness.is_some());
    MaterialTextures {
      roughness: self.roughness.take().map(|x| {dbg!("rough map");mapper("roughness", x)}),
      specular:  self.specular.take().map(|x| mapper("specular", x)),
      glossy:    self.glossy.take().map(|x| mapper("glossy", x)),
      diffuse:   self.diffuse.take().map(|x| mapper("diffuse", x)),
      emission:  self.emission.take().map(|x| mapper("emission", x)),
      normal:    self.normal.take().map(|x| mapper("normal", x)),
    }
  }
  pub fn and_then<U>(mut self, mut mapper: impl FnMut(&str, T) -> Option<U>) -> MaterialTextures<U> {
    MaterialTextures {
      roughness: self.roughness.take().and_then(|x| mapper("roughness", x)),
      specular:  self.specular.take().and_then(|x| mapper("specular", x)),
      glossy:    self.glossy.take().and_then(|x| mapper("glossy", x)),
      diffuse:   self.diffuse.take().and_then(|x| mapper("diffuse", x)),
      emission:  self.emission.take().and_then(|x| mapper("emission", x)),
      normal:    self.normal.take().and_then(|x| mapper("normal", x)),
    }
  }
}
