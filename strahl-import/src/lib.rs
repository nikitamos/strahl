#![doc = include_str!("../README.md")]
#![feature(file_buffered)]
use std::{fs::File, io::Write};

use image::codecs::png::PngEncoder;
use serde::{Deserialize, Serialize};

pub mod builder;
pub mod reader;

pub mod per_texture;

pub type ImportedMaterial = reader::Material;
pub enum Never {}
impl Never {
  pub fn write_to_memory(self) -> Result<Vec<u8>, anyhow::Error> { unimplemented!() }
  pub fn from_memory(mem: &[u8]) -> Result<Self, anyhow::Error> { unimplemented!() }
}
pub(crate) type Ktx2Texture = Never;

#[derive(Serialize, Deserialize)]
pub enum TextureFormat {
  Png,
  Ktx2,
  Rgba { r: f32, g: f32, b: f32, a: f32 },
}

#[derive(Serialize, Deserialize)]
pub struct TextureMetadata {
  pub format: TextureFormat,
}

impl Default for TextureMetadata {
  fn default() -> Self {
    Self {
      format: TextureFormat::Rgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
      },
    }
  }
}

#[allow(unused)]
pub enum MaterialComponentSource {
  Image(image::DynamicImage),
  /// Valid PNG file
  File(File),
  Ktx(Ktx2Texture),
  Rgba {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
  },
}

impl MaterialComponentSource {
  fn write<W: Write>(self, w: &mut W) -> anyhow::Result<TextureMetadata> {
    match self {
      MaterialComponentSource::Image(img) => {
        let enc = PngEncoder::new(w);
        img.write_with_encoder(enc)?;
        Ok(TextureMetadata {
          format: TextureFormat::Png,
        })
      }
      MaterialComponentSource::Ktx(ktx) => {
        w.write_all(&ktx.write_to_memory()?)?;
        Ok(TextureMetadata {
          format: TextureFormat::Ktx2,
        })
      }
      MaterialComponentSource::File(mut f) => {
        std::io::copy(&mut f, w)?;
        Ok(TextureMetadata {
          format: TextureFormat::Png,
        })
      }
      Self::Rgba { r, g, b, a } => Ok(TextureMetadata {
        format: TextureFormat::Rgba { r, g, b, a },
      }),
    }
  }
  fn append_ext(&self, mut path: String) -> String {
    path.push_str(match self {
      MaterialComponentSource::Image(_) | MaterialComponentSource::File(_) => ".png",
      MaterialComponentSource::Ktx(_) => ".ktx2",
      MaterialComponentSource::Rgba { .. } => "",
    });
    path
  }
}

const MATERIAL_METADATA: &str = "metadata.toml";
