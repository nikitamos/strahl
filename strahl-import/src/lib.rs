use std::{
  fs::File,
  io::{BufRead, Read, Seek, Write},
};

use image::codecs::png::{PngDecoder, PngEncoder};
use ktx2_rw::Ktx2Texture;

pub mod builder;

pub mod material_textures;

pub enum StoredTexture {
  Image(image::DynamicImage),
  /// Valid PNG file
  File(File),
  Ktx(Ktx2Texture),
}

pub struct TextureMetadata {}

impl StoredTexture {
  fn write<W: Write>(self, mut w: W) -> anyhow::Result<W> {
    log::trace!("writing started");
    match self {
      StoredTexture::Image(img) => {
        let enc = PngEncoder::new(&mut w);
        img.write_with_encoder(enc)?;
      }
      StoredTexture::Ktx(ktx) => w.write_all(&ktx.write_to_memory()?)?,
      StoredTexture::File(mut f) => {
        std::io::copy(&mut f, &mut w)?;
      }
    };
    log::trace!("writing ended successfully");
    Ok(w)
  }
  fn append_ext(&self, mut path: String) -> String {
    path.push_str(match self {
      StoredTexture::Image(_) | StoredTexture::File(_) => "png",
      StoredTexture::Ktx(_) => "ktx2",
    });
    path
  }
}
