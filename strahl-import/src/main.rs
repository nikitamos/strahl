// #![cfg(feature = "converter")]

use std::{
  fs::{self, File},
  path::{self, PathBuf},
};

use clap::Parser;
use serde::Deserialize;
use strahl_import::{
  MaterialComponentSource, builder::MaterialFileBuilder, per_texture::PerTexture,
  reader::GltfGeometry,
};

#[derive(Deserialize)]
#[serde(untagged)]
enum BSDFSpec {
  Path(String),
  Color { r: u8, g: u8, b: u8, a: u8 },
}

#[derive(serde::Deserialize)]
struct MaterialPacker {
  /// Paths to corresponding textures
  textures: PerTexture<BSDFSpec>,
}

#[derive(clap::Parser)]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
  /// Pack material into a ZIP archive
  CreateMaterial {
    /// Path to the material.toml
    descriptor: PathBuf,
    /// Path to the output material (zip archive)
    output:     PathBuf,
  },
  /// Validate glTF
  ValidateGltf {
    /// Path to the gltf (or glb) file
    path: PathBuf,
  },
}

fn main() -> anyhow::Result<()> {
  env_logger::builder()
    .filter_module("strahl_import", log::LevelFilter::Trace)
    .format_timestamp(None)
    .parse_default_env()
    .init();
  let cli = Cli::parse();
  match cli.command {
    Commands::CreateMaterial { descriptor, output } => create_material(descriptor, output)?,
    Commands::ValidateGltf { path } => {
      let geom = GltfGeometry::import_validate(path)?;
      log::info!("glTF is valid, {geom:#?}");
    }
  }
  Ok(())
}

fn create_material(descriptor: PathBuf, output: PathBuf) -> Result<(), anyhow::Error> {
  let packer: MaterialPacker = toml::from_str(&fs::read_to_string(&descriptor)?)?;
  let textures = packer
    .textures
    .map_all(|bsdf| match bsdf {
      BSDFSpec::Path(tex_path) => {
        let abs_tex_path = path::absolute(&descriptor)?
          .parent()
          .unwrap()
          .join(tex_path);
        Ok(MaterialComponentSource::File(File::open(abs_tex_path)?))
      }
      BSDFSpec::Color { r, g, b, a } => Ok(MaterialComponentSource::Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: a as f32 / 255.0,
      }),
    })
    .and_then(|n, f: std::io::Result<MaterialComponentSource>| {
      f.inspect_err(|e| eprintln!("Failed to read '{n}' texture: {e}"))
        .ok()
    });
  MaterialFileBuilder::new()
    .textures(textures)
    .write(output)?;
  Ok(())
}
