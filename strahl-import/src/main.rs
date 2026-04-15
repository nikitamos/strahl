// #![cfg(feature = "converter")]

use std::{
  fs::{self, File},
  path::{self, PathBuf},
};

use clap::Parser;
use strahl_import::{builder::MaterialFileBuilder, per_texture::PerTexture};

#[derive(serde::Deserialize)]
struct MaterialPacker {
  /// Paths to corresponding textures
  textures: PerTexture<String>,
}

#[derive(clap::Parser)]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
  CreateMaterial {
    /// Path to the material.toml
    descriptor: PathBuf,
    /// Path to the output material (zip archive)
    output:     PathBuf,
  },
}

fn main() -> anyhow::Result<()> {
  env_logger::init();
  let cli = Cli::parse();
  match cli.command {
    Commands::CreateMaterial { descriptor, output } => {
      let packer: MaterialPacker = toml::from_str(&fs::read_to_string(&descriptor)?)?;
      let textures = packer
        .textures
        .map_all(|tex_path| {
          let abs_tex_path = path::absolute(&descriptor)?
            .parent()
            .unwrap()
            .join(tex_path);
          File::open(abs_tex_path)
        })
        .and_then(|n, f| {
          f.inspect_err(|e| eprintln!("Failed to read '{n}' texture: {e}"))
            .ok()
        });
      MaterialFileBuilder::new()
        .textures(textures)
        .write(output)?;
    }
  }
  Ok(())
}
