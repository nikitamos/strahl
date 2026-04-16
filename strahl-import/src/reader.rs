use std::{
  fs::File,
  io::{BufReader, Read},
  path::Path,
};

use anyhow::{anyhow, bail};
use gltf::{
  Accessor,
  accessor::{DataType, Dimensions},
  buffer::{self, Target},
};
use ktx2_rw::Ktx2Texture;
use zip::ZipArchive;

use crate::{MATERIAL_METADATA, MaterialComponentSource, TextureMetadata, per_texture::PerTexture};

pub struct Material {
  textures: PerTexture<MaterialComponentSource>,
}

impl Material {
  pub fn read(zip_path: impl AsRef<Path>) -> anyhow::Result<Self> {
    let mut zip = ZipArchive::new(File::open(zip_path)?)?;
    let f = zip.by_path(MATERIAL_METADATA)?;
    let metadata: PerTexture<TextureMetadata> = toml::from_str(&std::io::read_to_string(f)?)?;
    // FIXME: move path construction logic out!
    let textures = metadata
      .map_named(|n, meta| match meta.format {
        crate::TextureFormat::Png => {
          // Get index by path
          let rdr = BufReader::new(zip.by_name_seek(&format!("surface/{n}.png"))?);
          Ok(MaterialComponentSource::Image(
            image::ImageReader::new(rdr)
              .with_guessed_format()?
              .decode()?,
          ))
        }
        crate::TextureFormat::Ktx2 => {
          let mut rdr = zip.by_name(&format!("surface/{n}.ktx2"))?;
          let mut buf = Vec::with_capacity(rdr.size() as usize);
          rdr.read_to_end(&mut buf)?;
          Ok(MaterialComponentSource::Ktx(Ktx2Texture::from_memory(
            &buf,
          )?))
        }
        crate::TextureFormat::Rgba { r, g, b, a } => {
          Ok::<_, anyhow::Error>(MaterialComponentSource::Rgba { r, g, b, a })
        }
      })
      .map_named(|n, t| {
        log::trace!("mapping {n}");
        t.inspect_err(|e| log::error!("failed to load {n} texture: {e}"))
          .unwrap_or(MaterialComponentSource::Rgba {
            r: 4.0 / 255.0,
            g: 65.0 / 255.0,
            b: 229.0 / 255.0,
            a: 1.0,
          })
      });
    Ok(Self { textures })
  }
}

#[derive(Debug)]
pub struct GltfBufferView {
  /// Offset of the view within the buffer
  pub offset: usize,
  /// Count of entries in the accessor
  pub count:  usize,
  /// Length of view in bytes
  pub length: usize,
  /// Stride of the accessor. If glTF contains no value,
  /// it is inferred using the size of buffer component.
  pub stride: usize,
}

impl GltfBufferView {
  fn new(
    mesh_name: &str,
    acc: Accessor<'_>,
    dimensions: Dimensions,
    ty: DataType,
  ) -> anyhow::Result<Self> {
    let index = acc.index();
    Self::validate_not_sparse(mesh_name, &acc, index)?;
    Self::validate_dim(mesh_name, &acc, dimensions, index)?;
    Self::validate_ty(mesh_name, &acc, ty, index)?;
    let view = acc.view().unwrap();
    if view.target().is_none() {
      log::warn!("Accessor's view doesn't have a binding target");
    }

    Ok(Self {
      offset: view.offset() + acc.offset(),
      count:  acc.count(),
      length: view.length(),
      stride: view.stride().unwrap_or(acc.size()),
    })
  }
  fn new_validate_dim(
    mesh_name: &str,
    acc: Accessor<'_>,
    dimensions: Dimensions,
  ) -> anyhow::Result<Self> {
    let index = acc.index();
    Self::validate_not_sparse(mesh_name, &acc, index)?;
    Self::validate_dim(mesh_name, &acc, dimensions, index)?;
    let view = acc.view().unwrap();
    if view.target().is_none() {
      log::warn!("Accessor's view doesn't have a binding target");
    }

    Ok(Self {
      offset: view.offset(),
      count:  acc.count(),
      length: view.length(),
      stride: view.stride().unwrap_or(acc.size()),
    })
  }
  fn validate_not_sparse(
    mesh_name: &str,
    acc: &Accessor<'_>,
    index: usize,
  ) -> Result<(), anyhow::Error> {
    if acc.sparse().is_some() {
      log::error!("mesh {mesh_name}: accessor {index} is sparse");
      anyhow::bail!("sparse accessors are disallowed");
    }
    Ok(())
  }
  fn validate_dim(
    mesh_name: &str,
    acc: &Accessor<'_>,
    dimensions: Dimensions,
    index: usize,
  ) -> Result<(), anyhow::Error> {
    if acc.dimensions() != dimensions {
      log::error!("mesh {mesh_name}, accessor {index}: invalid dimension");
      anyhow::bail!("invalid accessor dimension");
    }
    Ok(())
  }
  fn validate_ty(
    mesh_name: &str,
    acc: &Accessor<'_>,
    ty: DataType,
    index: usize,
  ) -> Result<(), anyhow::Error> {
    if acc.data_type() != ty {
      log::error!("mesh {mesh_name}, accessor {index}: invalid data type");
      anyhow::bail!("invalid accessor data type");
    }
    Ok(())
  }
}

#[derive(Debug)]
pub struct GltfGeometry {
  pub position:   GltfBufferView,
  pub normals:    GltfBufferView,
  pub uv:         GltfBufferView,
  pub indices:    GltfBufferView,
  pub index_size: u8,
  pub buffer:     Vec<u8>,
}

impl GltfGeometry {
  pub fn import_validate(path: impl AsRef<Path>) -> anyhow::Result<Self> {
    let (doc, buffers, _images) = gltf::import(path.as_ref())?;
    if doc.meshes().count() != 1 {
      log::error!(
        "{} contains {} meshes, expected 1",
        path.as_ref().to_string_lossy(),
        doc.meshes().count()
      );
      bail!("wrong count of meshes in gltf");
    }
    let mesh = doc.meshes().next().unwrap();
    let mesh_name = mesh.name().unwrap_or("<unnamed mesh>");

    if buffers.len() != 0 {
      log::error!(
        "Mesh {mesh_name} contains {} meshes, expected 1",
        buffers.len()
      );
      bail!("wrong count of buffers in gltf");
    }
    let buffer = buffers.into_iter().next().unwrap().0;

    log::info!(
      "Reading mesh {mesh_name} from {}",
      path.as_ref().to_string_lossy()
    );
    if mesh.primitives().count() != 1 {
      log::error!(
        "Mesh {} contains {} primitives, expected 1",
        mesh_name,
        mesh.primitives().count()
      );
      bail!("wrong count of primitives in mesh");
    }

    let primitive = mesh.primitives().next().unwrap();
    let position = primitive.get(&gltf::Semantic::Positions).ok_or_else(|| {
      log::error!("Mesh {mesh_name} doesn't have position attribute");
      anyhow!("mesh doesn't have position attribute")
    })?;
    let normals = primitive.get(&gltf::Semantic::Normals).ok_or_else(|| {
      log::error!("Mesh {mesh_name} doesn't have normal map");
      anyhow!("mesh doesn't have normal map")
    })?;
    let uv = primitive
      .get(&gltf::Semantic::TexCoords(0))
      .ok_or_else(|| {
        log::error!("Mesh {mesh_name} doesn't have UV map");
        anyhow!("mesh doesn't have UV map")
      })?;
    if primitive.attributes().count() != 3 {
      log::warn!(
        "Mesh {mesh_name} has {} attributes, expected 3. Unknown attributes ignored.",
        primitive.attributes().count()
      );
    }
    let indices = primitive.indices().ok_or_else(|| {
      log::error!("Mesh {mesh_name} doesn't have index buffer");
      anyhow::anyhow!("mesh doesn't have index buffer")
    })?;

    Ok(Self {
      index_size: indices.size() as u8,
      position:   GltfBufferView::new(mesh_name, position, Dimensions::Vec3, DataType::F32)?,
      normals:    GltfBufferView::new(mesh_name, normals, Dimensions::Vec3, DataType::F32)?,
      uv:         GltfBufferView::new(mesh_name, uv, Dimensions::Vec2, DataType::F32)?,
      indices:    GltfBufferView::new_validate_dim(mesh_name, indices, Dimensions::Scalar)?,
      buffer
    })
  }
}
