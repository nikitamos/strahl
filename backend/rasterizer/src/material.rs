use std::num::NonZero;

use strahl_import::{ImportedMaterial, StoredTexture};
use wgpu::{BindGroupDescriptor, ShaderStages, util::DeviceExt};
use zerocopy::IntoBytes;

#[derive(Clone)]
#[non_exhaustive]
pub struct Material {
  colors_buffer: wgpu::Buffer,
  group:         wgpu::BindGroup,
  layout:        wgpu::BindGroupLayout,
  color:         u32,
}

const MATERIAL_COMPONENTS: usize = 6;
#[repr(transparent)]
#[derive(Default, zerocopy::IntoBytes)]
struct Colors([glam::Vec4; MATERIAL_COMPONENTS]);


impl Material {
  pub fn from_imported(dev: &wgpu::Device, queue: &wgpu::Queue, s: ImportedMaterial) -> Self {
    // TODO: reuse the sampler
    const BUFFER_BINDING: u32 = (MATERIAL_COMPONENTS as u32) + 1;
    let mut color = 0x00;
    let mut colors = Colors::default();
    let mut layout = Vec::with_capacity(MATERIAL_COMPONENTS + 2);
    let mut descriptors = Vec::with_capacity(MATERIAL_COMPONENTS + 2);
    let mut textures: [Option<wgpu::Texture>; MATERIAL_COMPONENTS] = Default::default();
    let mut texture_views: [Option<wgpu::TextureView>; MATERIAL_COMPONENTS] = Default::default();

    // TODO: sampler settings
    layout.push(wgpu::BindGroupLayoutEntry {
      binding:    0,
      visibility: wgpu::ShaderStages::FRAGMENT,
      ty:         wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
      count:      None,
    });
    let sampler = dev.create_sampler(&wgpu::SamplerDescriptor {
      label: None,
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToBorder,
      ..Default::default()
    });
    descriptors.push(wgpu::BindGroupEntry {
      binding:  0,
      resource: wgpu::BindingResource::Sampler(&sampler),
    });

    for ((i, tex), view) in s
      .textures()
      .into_iter()
      .enumerate()
      .zip(texture_views.iter_mut())
    {
      match tex {
        Some(StoredTexture::Image(img)) => {
          layout.push(wgpu::BindGroupLayoutEntry {
            binding:    (i + 1) as u32,
            visibility: ShaderStages::FRAGMENT,
            ty:         wgpu::BindingType::Texture {
              sample_type:    wgpu::TextureSampleType::Float { filterable: true },
              view_dimension: wgpu::TextureViewDimension::D2,
              multisampled:   false,
            },
            count:      None,
          });

          let rgba = img.to_rgba8();
          let dimensions = rgba.dimensions();

          let tex = dev.create_texture(&wgpu::TextureDescriptor {
            label:           Some("Material Texture"),
            size:            wgpu::Extent3d {
              width:                 dimensions.0,
              height:                dimensions.1,
              depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count:    1,
            dimension:       wgpu::TextureDimension::D2,
            format:          wgpu::TextureFormat::Rgba8Unorm,
            usage:           wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats:    &[wgpu::TextureFormat::Rgba8Unorm],
          });

          queue.write_texture(
            wgpu::TexelCopyTextureInfo {
              texture:   &tex,
              mip_level: 0,
              origin:    wgpu::Origin3d::ZERO,
              aspect:    wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
              offset:         0,
              bytes_per_row:  Some(4 * dimensions.0),
              rows_per_image: Some(dimensions.1),
            },
            wgpu::Extent3d {
              width:                 dimensions.0,
              height:                dimensions.1,
              depth_or_array_layers: 1,
            },
          );

          *view = Some(tex.create_view(&wgpu::TextureViewDescriptor::default()));
          textures[i] = Some(tex);

          descriptors.push(wgpu::BindGroupEntry {
            binding:  (i + 1) as u32,
            resource: wgpu::BindingResource::TextureView(view.as_ref().unwrap()),
          });
        }
        Some(StoredTexture::Ktx(ktx)) => todo!(),
        Some(StoredTexture::Rgba { r, g, b, a }) => {
          color |= 0x01 << i;
          colors.0[i] = glam::vec4(*r, *g, *b, *a);
        }
        _ => panic!("Bad material (all components are expected to be present)"),
      }
    }

    let colors_buffer = dev.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label:    None,
      contents: colors.0.as_bytes(),
      usage:    wgpu::BufferUsages::UNIFORM,
    });
    layout.push(wgpu::BindGroupLayoutEntry {
      binding:    BUFFER_BINDING,
      visibility: wgpu::ShaderStages::FRAGMENT,
      ty:         wgpu::BindingType::Buffer {
        ty:                 wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size:   NonZero::new(std::mem::size_of::<Colors>() as u64),
      },
      count:      None,
    });
    descriptors.push(wgpu::BindGroupEntry {
      binding:  BUFFER_BINDING,
      resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
        buffer: &colors_buffer,
        offset: 0,
        size:   NonZero::new(std::mem::size_of::<Colors>() as u64),
      }),
    });

    let layout = dev.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label:   None,
      entries: &layout,
    });
    let group = dev.create_bind_group(&BindGroupDescriptor {
      label:   None,
      layout:  &layout,
      entries: &descriptors,
    });
    Self {
      colors_buffer,
      group,
      layout,
      color,
    }
  }

  pub fn bind_group(&self) -> &wgpu::BindGroup { &self.group }
  pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout { &self.layout }
  pub fn color(&self) -> u32 { self.color }
}
