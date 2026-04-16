use std::{
  fs::File,
  io::{BufReader, Read},
  path::Path,
  process::{ExitCode, Termination},
};

use image::codecs::png::PngDecoder;
use ktx2_rw::Ktx2Texture;
use zip::{HasZipMetadata, ZipArchive, result::ZipResult};

use crate::{MATERIAL_METADATA, StoredTexture, TextureMetadata, per_texture::PerTexture};

pub struct Material {
  textures: PerTexture<StoredTexture>,
}

impl Termination for Material {
  fn report(self) -> std::process::ExitCode {
    println!("ok?");
    ExitCode::from(12)
  }
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
          Ok(StoredTexture::Image(
            image::ImageReader::new(rdr)
              .with_guessed_format()?
              .decode()?,
          ))
        }
        crate::TextureFormat::Ktx2 => {
          let mut rdr = zip.by_name(&format!("surface/{n}.ktx2"))?;
          let mut buf = Vec::with_capacity(rdr.size() as usize);
          rdr.read_to_end(&mut buf)?;
          Ok(StoredTexture::Ktx(Ktx2Texture::from_memory(&buf)?))
        }
        crate::TextureFormat::Rgba { r, g, b, a } => {
          Ok::<_, anyhow::Error>(StoredTexture::Rgba { r, g, b, a })
        }
      })
      .map_named(|n, t| {
        log::trace!("mapping {n}");
        t.inspect_err(|e| log::error!("failed to load {n} texture: {e}"))
          .unwrap_or(StoredTexture::Rgba {
            r: 4.0 / 255.0,
            g: 65.0 / 255.0,
            b: 229.0 / 255.0,
            a: 1.0,
          })
      });
    Ok(Self { textures })
  }
  pub fn textures(&self) -> &PerTexture<StoredTexture> { &self.textures }
}
