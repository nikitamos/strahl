use std::{
  fs::File,
  io::{BufReader, Seek, Write},
  path::Path,
};

use anyhow::bail;
use ktx2_rw::Ktx2Texture;
use zip::write::FileOptions;

use crate::{StoredTexture, TextureMetadata, per_texture::PerTexture};

#[derive(Default)]
pub struct MaterialFileBuilder {
  textures: PerTexture<File>,
}

macro_rules! material_type_import {
  {$($mat:ident,)*} => {
    $(
    #[doc = r"Imports "]
    #[doc = stringify!($mat)]
    #[doc = r" material component from given file"]
    pub fn $mat(mut self, $mat: File) -> Self {
      self.textures.$mat.replace($mat);
      self
    }
    )*
  };
}

const NO_COMPRESSION: FileOptions<'static, ()> = FileOptions::DEFAULT
  .compression_level(None)
  .compression_method(zip::CompressionMethod::Stored);

impl MaterialFileBuilder {
  pub fn new() -> Self { Default::default() }
  pub fn textures(mut self, textures: PerTexture<File>) -> Self {
    self.textures = textures;
    self
  }
  material_type_import! {roughness,specular,glossy,diffuse,emission,normal,}
  pub fn write(mut self, output: impl AsRef<Path>) -> anyhow::Result<()> {
    let mut zip = zip::ZipWriter::new(File::create(output)?);
    zip.add_directory("surface", NO_COMPRESSION)?;
    // TODO: parallelize image parsing!
    let taken = self.textures.take();
    let meta: PerTexture<TextureMetadata> = taken
      .map_named(|n, f| match self.parse_image(f) {
        Ok(tex) => {
          zip.start_file(tex.append_ext(format!("surface/{n}")), NO_COMPRESSION)?;

          Ok(tex.write(&mut zip)?)
        }
        Err(e) => bail!(e),
      })
      .map_named(|n, s| {
        s.inspect_err(|e| log::error!("skipping {n} texture due to an error: {e}"))
          .unwrap_or_default()
      }).or_else(|n| {
        log::warn!("No {n} texture supplied. Writing default metadata.");
        Some(Default::default())});
    zip.start_file("metadata.toml", NO_COMPRESSION)?;
    write!(zip, "{}", toml::to_string(&meta)?)?;
    Ok(())
  }
  fn parse_image(&self, f: File) -> anyhow::Result<StoredTexture> {
    // image::
    match image::ImageReader::new(BufReader::new(f))
      .with_guessed_format().map(|x| (x.format(),x,))
    {
      // Just copy the PNG.
      Ok((Some(image::ImageFormat::Png), rdr)) => {
        log::debug!("PNG image detected, copying");
        let mut f = rdr.into_inner().into_inner();
        f.rewind()?;
        Ok(StoredTexture::File(f))
      }
      Ok((Some(fmt), _rdr)) => {
        log::debug!("transcoding of {fmt:?} required, ignoring");
        bail!("transcoding is not implemented yet")
      }
      Ok((None, _rdr)) => {
        // might be KTX
        log::error!("unknown image format, ignoring");
        bail!("unknown format");
      }
      Err(e) => bail!(e),
    }
  }
}

fn encode_to_ktx(img: image::DynamicImage) -> anyhow::Result<()> {
  let mut tex = Ktx2Texture::create(
    img.width(),
    img.height(),
    1,
    1,
    1,
    1,
    ktx2_rw::VkFormat::Bc7UnormBlock,
  )?;
  tex.set_image_data(
    0,
    0,
    0,
    img
      .as_rgb8()
      .ok_or(anyhow::anyhow!("invalid image format"))?,
  )?;
  tex.compress_basis_simple(80)?;

  todo!()
}
