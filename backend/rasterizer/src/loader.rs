use crate::{geometry::Geometry, material::Material};

use std::{
  borrow::Cow,
  path::{Path, PathBuf},
  sync::Arc,
};

#[derive(Clone, Debug)]
pub struct AssetLoader {
  pub(crate) dev:   wgpu::Device,
  pub(crate) queue: wgpu::Queue,
  prefix:           Option<PathBuf>,
}

impl AssetLoader {
  pub(crate) fn new(dev: wgpu::Device, queue: wgpu::Queue) -> Self {
    Self {
      dev,
      queue,
      prefix: None,
    }
  }
  pub fn load_material(&self, path: impl AsRef<Path>) -> anyhow::Result<Arc<Material>> {
    let imported = strahl_import::reader::Material::read(self.get_path(path.as_ref()))?;
    Ok(Arc::new(Material::from_imported(
      &self.dev,
      &self.queue,
      imported,
    )))
  }
  pub fn load_mesh(&self, path: impl AsRef<Path>) -> anyhow::Result<Arc<Geometry>> {
    let gltf = strahl_import::reader::GltfGeometry::import_validate(self.get_path(path.as_ref()))?;
    Geometry::from_gltf(&self.dev, gltf).map(Arc::new)
  }

  pub fn set_prefix(&mut self, path: PathBuf) -> Option<PathBuf> { self.prefix.replace(path) }

  pub fn prefix(&self) -> Option<&Path> { self.prefix.as_deref() }

  fn get_path<'a>(&self, suffix: &'a Path) -> Cow<'a, Path> {
    if suffix.is_absolute() {
      Cow::Borrowed(suffix)
    } else if let Some(prefix) = self.prefix.as_deref() {
      Cow::Owned(prefix.join(suffix))
    } else {
      Cow::Borrowed(suffix)
    }
  }
}
