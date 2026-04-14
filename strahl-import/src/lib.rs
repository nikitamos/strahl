use std::io::Write;

use ktx2_rw::Ktx2Texture;

pub mod builder;

pub mod material_textures;

pub enum StoredTexture {
  Png(image::DynamicImage),
  Ktx(Ktx2Texture),
}

pub struct TextureMetadata {}

impl StoredTexture {
  fn write<W: Write>(self, mut w: W) -> anyhow::Result<W> {
    match self {
      StoredTexture::Png(img) => {
        let mut cursor = std::io::Cursor::new(Vec::new());
        img.write_to(&mut cursor, image::ImageFormat::Png)?;
        w.write(&cursor.into_inner())?;
      }
      StoredTexture::Ktx(ktx) => w.write_all(&ktx.write_to_memory()?)?,
    };
    Ok(w)
  }
  fn append_ext(&self, mut path: String) -> String {
    path.push_str(match self {
      StoredTexture::Png(_) => "png",
      StoredTexture::Ktx(_) => "ktx2",
    });
    path
  }
}
