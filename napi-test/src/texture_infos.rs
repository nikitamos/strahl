use napi_derive::napi;

  use std::collections::HashMap;

  #[napi]
  pub struct Plane {
/// The strides and offsets in bytes to be used when accessing the buffers via a
/// memory mapping. One per plane per entry.
stride: usize,
// The strides and offsets in bytes to be used when accessing the buffers via a
// memory mapping. One per plane per entry.
offset: usize,
/// Size in bytes of the plane. This is necessary to map the buffers.
size: usize,
/// File descriptor for the underlying memory object (usually dmabuf).
fd: usize,
  }

  #[napi]
  pub struct Size {
pub width: i64,
pub height: i64,
  }

  #[napi]
  pub struct Rectangle {
/**
     * The height of the rectangle (must be an integer).
     */
pub height: i64,
/**
     * The width of the rectangle (must be an integer).
     */
pub width: i64,
/**
     * The x coordinate of the origin of the rectangle (must be an integer).
     */
pub x: i64,
/**
     * The y coordinate of the origin of the rectangle (must be an integer).
     */
pub y: i64,
  }

  #[napi]
  pub struct NativePixmap {}

  #[napi(string_enum)]
  pub enum PixelFormat {
#[allow(non_camel_case_types)]
bgra,
#[allow(non_camel_case_types)]
rgba,
#[allow(non_camel_case_types)]
rgbaf16,
#[allow(non_camel_case_types)]
nv12,
  }

  #[napi]
  pub struct SharedTextureHandle {
pub nt_handle: i64,
  }

  #[napi]
  pub struct SharedTextureImportTextureInfo {
/// The full dimensions of the shared texture.
pub coded_size: Size,
///  The color space of the texture.
pub color_space: Option<HashMap<String, String>>,
/// The shared texture handle.
pub handle: SharedTextureHandle,
/// The pixel format of the texture.
pub pixel_format: PixelFormat,
/// A timestamp in microseconds that will be reflected to `VideoFrame`.
pub timestamp: f64,
/// A subsection of [0, 0, codedSize.width, codedSize.height]. In common cases, it
/// is the full section area.
pub visible_rect: Option<Rectangle>,
  }

  #[napi]
  pub struct ImportSharedTextureOptions {
pub texture_info: SharedTextureImportTextureInfo,
  }
