use rasterizer::material::Material;
pub use rasterizer::*;

pub  fn main() -> anyhow::Result<()>{
  env_logger::Builder::new()
    .default_format()
    .filter_level(log::LevelFilter::Info)
    .format_timestamp(None)
    .build();

    tokio::runtime::Builder::new_current_thread().build()?.block_on(true_main())?;
    Ok(())
}

pub async fn true_main() -> anyhow::Result<()> {
  let strahl = Rasterizer::new().await?;
  Ok(())
}
