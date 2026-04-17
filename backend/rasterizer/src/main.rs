use anyhow::anyhow;
use image::Rgba;
pub use rasterizer::*;

pub fn main() -> anyhow::Result<()> {
  env_logger::Builder::new()
    .filter_level(log::LevelFilter::Info)
    .filter_module("rasterizer", log::LevelFilter::Trace)
    .format_timestamp(None)
    .parse_default_env()
    .init();

  tokio::runtime::Builder::new_current_thread()
    .build()?
    .block_on(true_main())?;
  Ok(())
}

pub async fn true_main() -> anyhow::Result<()> {
  let size = glam::uvec2(1920, 1080);
  let mut strahl = Rasterizer::new(RasterizerCreateInfo { viewport: size }).await?;
  let material = strahl.load_material("../../../strahl-import/assets/gas.zip")?;
  let geometry = strahl.load_mesh("../../../strahl-import/assets/lava/Lava.gltf")?;
  let mut scene = strahl.create_scene();
  let body = scene.add_body(geometry, material);
  let test = strahl.render(&scene);
  let buf = image::ImageBuffer::<Rgba<u8>, _>::from_raw(size.x, size.y, test).ok_or_else(|| {
    log::error!("failed to import image");
    anyhow!("failed to import image")
  })?;
  buf.save("out.png")?;

  Ok(())
}
