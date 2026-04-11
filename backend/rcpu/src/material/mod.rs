pub mod bsdf;
pub mod medium;

pub trait Material: Send + Sync + std::fmt::Debug {}
