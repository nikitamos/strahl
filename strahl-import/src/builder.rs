use std::{fs::File, io::BufReader, path::Path};

use anyhow::bail;
use ktx2_rw::Ktx2Texture;
use zip::write::FileOptions;

use crate::{StoredTexture, TextureMetadata, material_textures::MaterialTextures};

#[derive(Default)]
pub struct MaterialFileBuilder {
  textures: MaterialTextures<File>,
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

impl MaterialFileBuilder {
  pub fn new() -> Self { Default::default() }
  pub fn textures(mut self, textures: MaterialTextures<File>) -> Self {
    self.textures = textures;
    self
  }
  material_type_import! {roughness,specular,glossy,diffuse,emission,normal,}
  pub fn write(mut self, output: impl AsRef<Path>) -> anyhow::Result<()> {
    let mut zip = zip::ZipWriter::new(File::create(output)?);
    zip.add_directory("surface", FileOptions::DEFAULT)?;
    // TODO: parallelize image parsing!
    let taken = self.textures.take();
    let _texs: MaterialTextures<TextureMetadata> = taken
      .map_named(|n, f| match self.parse_image(f) {
        Ok(tex) => {
          zip.start_file(tex.append_ext(format!("surface/{n}")), FileOptions::DEFAULT)?;

          tex.write(&mut zip)?;
          Ok(TextureMetadata {})
        }
        Err(e) => bail!(e),
      })
      .and_then(|n, s| {
        s.inspect_err(|e| eprintln!("failed to pack {n} texture: {e}; skipping"))
          .ok()
      });
    Ok(())
  }
  pub fn parse_image(&self, f: File) -> anyhow::Result<StoredTexture> {
    match image::ImageReader::new(BufReader::new(f))
      .with_guessed_format()?
      .decode()
    {
      Ok(img) => {
        Ok(StoredTexture::Png(img))
        // encode_to_ktx(img)
      }
      Err(e @ image::ImageError::Unsupported(_)) => {
        anyhow::bail!(e)
      }
      Err(e) => anyhow::bail!(e),
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
