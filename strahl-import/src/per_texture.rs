#[derive(serde::Deserialize, serde::Serialize)]
pub struct PerTexture<T> {
  pub roughness: Option<T>,
  pub specular:  Option<T>,
  pub glossy:    Option<T>,
  pub diffuse:   Option<T>,
  pub emission:  Option<T>,
  pub normal:    Option<T>,
}

impl<T> Default for PerTexture<T> {
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

impl<T> PerTexture<T> {
  pub fn map_all<U>(mut self, mapper: impl Fn(T) -> U) -> PerTexture<U> {
    PerTexture {
      roughness: self.roughness.take().map(&mapper),
      specular:  self.specular.take().map(&mapper),
      glossy:    self.glossy.take().map(&mapper),
      diffuse:   self.diffuse.take().map(&mapper),
      emission:  self.emission.take().map(&mapper),
      normal:    self.normal.take().map(&mapper),
    }
  }
  pub fn take(&mut self) -> Self {
    PerTexture {
      roughness: self.roughness.take(),
      specular:  self.specular.take(),
      glossy:    self.glossy.take(),
      diffuse:   self.diffuse.take(),
      emission:  self.emission.take(),
      normal:    self.normal.take(),
    }
  }
  pub fn map_named<U>(mut self, mut mapper: impl FnMut(&str, T) -> U) -> PerTexture<U> {
    PerTexture {
      roughness: self.roughness.take().map(|x| mapper("roughness", x)),
      specular:  self.specular.take().map(|x| mapper("specular", x)),
      glossy:    self.glossy.take().map(|x| mapper("glossy", x)),
      diffuse:   self.diffuse.take().map(|x| mapper("diffuse", x)),
      emission:  self.emission.take().map(|x| mapper("emission", x)),
      normal:    self.normal.take().map(|x| mapper("normal", x)),
    }
  }
  pub fn and_then<U>(mut self, mut mapper: impl FnMut(&str, T) -> Option<U>) -> PerTexture<U> {
    PerTexture {
      roughness: self.roughness.take().and_then(|x| mapper("roughness", x)),
      specular:  self.specular.take().and_then(|x| mapper("specular", x)),
      glossy:    self.glossy.take().and_then(|x| mapper("glossy", x)),
      diffuse:   self.diffuse.take().and_then(|x| mapper("diffuse", x)),
      emission:  self.emission.take().and_then(|x| mapper("emission", x)),
      normal:    self.normal.take().and_then(|x| mapper("normal", x)),
    }
  }
  pub fn or_else(mut self, mut fun: impl FnMut(&str) -> Option<T>) -> Self {
    PerTexture {
      roughness: self.roughness.take().or_else(|| fun("roughness")),
      specular:  self.specular.take().or_else(|| fun("specular")),
      glossy:    self.glossy.take().or_else(|| fun("glossy")),
      diffuse:   self.diffuse.take().or_else(|| fun("diffuse")),
      emission:  self.emission.take().or_else(|| fun("emission")),
      normal:    self.normal.take().or_else(|| fun("normal")),
    }
  }
}

impl<'a, T> IntoIterator for &'a PerTexture<T> {
  type Item = Option<&'a T>;

  type IntoIter = <[Self::Item; 6] as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    [
      self.roughness.as_ref(),
      self.specular.as_ref(),
      self.glossy.as_ref(),
      self.diffuse.as_ref(),
      self.emission.as_ref(),
      self.normal.as_ref(),
    ]
    .into_iter()
  }
}
