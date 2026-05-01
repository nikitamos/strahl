use std::{
  fs::File,
  io::{BufReader, Read},
  path::Path,
};

use anyhow::{Context, anyhow, bail};
use clap::error;
use gltf::{
  Accessor,
  accessor::{DataType, Dimensions},
  buffer::{self, Target},
};
use image::{DynamicImage, RgbImage};
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

    if buffers.len() != 1 {
      log::error!(
        "Mesh {mesh_name} contains {} buffers, expected 1",
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
      position: GltfBufferView::new(mesh_name, position, Dimensions::Vec3, DataType::F32)?,
      normals: GltfBufferView::new(mesh_name, normals, Dimensions::Vec3, DataType::F32)?,
      uv: GltfBufferView::new(mesh_name, uv, Dimensions::Vec2, DataType::F32)?,
      indices: GltfBufferView::new_validate_dim(mesh_name, indices, Dimensions::Scalar)?,
      buffer,
    })
  }
}

#[derive(Debug)]
pub struct CubemapImages {
  pub x_plus:  RgbImage,
  pub x_minus: RgbImage,
  pub y_plus:  RgbImage,
  pub y_minus: RgbImage,
  pub z_plus:  RgbImage,
  pub z_minus: RgbImage,
}

impl From<[RgbImage; 6]> for CubemapImages {
  fn from(images: [RgbImage; 6]) -> Self {
    let [x_plus, x_minus, y_plus, y_minus, z_plus, z_minus] = images;
    CubemapImages {
      x_plus,
      x_minus,
      y_plus,
      y_minus,
      z_plus,
      z_minus,
    }
  }
}

impl From<CubemapImages> for [RgbImage; 6] {
  fn from(cubemap: CubemapImages) -> Self {
    [
      cubemap.x_plus,
      cubemap.x_minus,
      cubemap.y_plus,
      cubemap.y_minus,
      cubemap.z_plus,
      cubemap.z_minus,
    ]
  }
}

#[derive(Debug)]
pub enum Cubemap {
  Images(CubemapImages),
}

impl Cubemap {
  const FACE_FILES: [&str; 6] = [
    "x_plus.png",
    "x_minus.png",
    "y_plus.png",
    "y_minus.png",
    "z_plus.png",
    "z_minus.png",
  ];
  pub fn read_from_dir_png(cubemap_dir: impl AsRef<Path>, transcode: bool) -> anyhow::Result<Self> {
    let paths = Self::FACE_FILES;
    let mut images: [Option<RgbImage>; 6] = Default::default();
    // 0 is invalid dimension for PNG file
    let mut width = 0;
    for (i, fname) in paths.into_iter().enumerate() {
      let path = cubemap_dir.as_ref().join(fname);
      let f = File::open_buffered(&path)
        .inspect_err(|e| log::error!("Failed to open cubemap face {fname}: {e}"))
        .context(format!("failed to open cubemap face {}", path.display()))?;
      let img = image::ImageReader::with_format(f, image::ImageFormat::Png).decode()?;

      Self::validate_square(&img, &path, cubemap_dir.as_ref())?;
      Self::validate_dim(&img, &mut width, cubemap_dir.as_ref())?;

      if let DynamicImage::ImageRgb8(buf) = img {
        images[i] = Some(buf);
      } else if transcode {
        log::warn!(
          "Image {} is in invalid format; transcoding to RGB8",
          path.display()
        );
        let buf = img.into_rgb8();
        let mut writer = File::create_buffered(path)
          .inspect_err(|e| log::error!("Transcoding error: failed to open file ({e})"))
          .context("transcoding error")?;
        buf
          .write_to(&mut writer, image::ImageFormat::Png)
          .inspect_err(|e| log::error!("Failed to transcode image: {e}"))
          .context("transcoding error")?;
        images[i] = Some(buf);
      } else {
        log::error!(
          "Cubemap {} is invalid: {} has non-RGB8 pixel format",
          cubemap_dir.as_ref().display(),
          fname
        );
        bail!(
          "Cubemap {} is invalid: there is non-RGB8 image",
          cubemap_dir.as_ref().display()
        );
      }
    }
    Ok(Cubemap::Images(images.map(Option::unwrap).into()))
  }

  fn validate_dim(
    img: &DynamicImage,
    width: &mut u32,
    cubemap_dir: &Path,
  ) -> Result<(), anyhow::Error> {
    if *width == 0 {
      *width = img.width();
    } else if img.width() != *width {
      log::error!(
        "Cubemap {0} is invalid: there are images with different dimensions {width}x{width} != {1}x{1}",
        cubemap_dir.display(),
        img.width()
      );
      bail!(
        "Cubemap {} is invalid: there are images with different dimensions",
        cubemap_dir.display()
      );
    }
    Ok(())
  }

  fn validate_square(
    img: &DynamicImage,
    path: &Path,
    cubemap_dir: &Path,
  ) -> Result<(), anyhow::Error> {
    if img.width() != img.height() {
      log::error!(
        "Cubemap {} is invalid: image {} has distinct height and width: {} != {}",
        cubemap_dir.display(),
        path.display(),
        img.height(),
        img.width()
      );
      bail!(
        "Image {} is invalid for the cubemap: width != height",
        path.display()
      );
    }
    Ok(())
  }
}
