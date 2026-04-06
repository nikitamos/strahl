#![deny(clippy::all)]

use napi_derive::napi;
use node::{node_export, node_only};

// #[cfg_attr(feature = "node", napi)]
#[napi]
pub struct Hello {
  pub x: u32,
  pub path: String,
}

#[napi]
impl Hello {
  // #[node_export(constructor)]
  #[napi(constructor)]
  pub fn new(x: u32) -> Self {
    Hello {
      x, path: "asdfasdf".to_string()
    }
  }
}

#[node_only]
impl Hello {
  #[napi]
  pub fn aaa(&self) {}
}

#[node_only]
pub fn plus_100(input: u32) -> napi::Result<Hello> {
  Ok(Hello {
    x: input + 100,
    path: std::env::current_dir()
      .map_err(|a| napi::Error::from_reason(a.to_string()))?
      .into_os_string()
      .into_string()
      .map_err(|_| napi::Error::from_reason("failed to convert path"))?,
  })
}
