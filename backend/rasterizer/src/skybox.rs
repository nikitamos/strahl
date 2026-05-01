use image::RgbaImage;
use strahl_import::reader::{Cubemap, CubemapImages};
use strahl_types::with;

use crate::shader_manager::ShaderEntryPoint;

pub struct Skybox {
  texture: wgpu::Texture,
  layout:  wgpu::BindGroupLayout,
  group:   wgpu::BindGroup,
  sampler: wgpu::Sampler,
}

impl Skybox {
  pub fn from_cubemap(cubemap: Cubemap, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    if let Cubemap::Images(imgs) = cubemap {
      Self::from_images(imgs, device, queue)
    } else {
      unreachable!()
    }
  }

  pub fn from_images(imgs: CubemapImages, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
      label:           Some("skybox"),
      size:            wgpu::Extent3d {
        width:                 512,
        height:                512,
        depth_or_array_layers: 6,
      },
      mip_level_count: 1,
      sample_count:    1,
      dimension:       wgpu::TextureDimension::D2,
      format:          wgpu::TextureFormat::Rgba8Unorm,
      usage:           wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
      view_formats:    &[],
    });

    let imgs: [RgbaImage; 6] = imgs.into();
    let origin = wgpu::Origin3d::ZERO;
    let dim = imgs[0].width();
    for i in 0..6 {
      queue.write_texture(
        wgpu::TexelCopyTextureInfoBase {
          texture:   &texture,
          mip_level: 0,
          origin:    with!(origin: z = i as u32),
          aspect:    wgpu::TextureAspect::All,
        },
        &imgs[i],
        wgpu::TexelCopyBufferLayout {
          offset:         0,
          bytes_per_row:  Some(4 * dim),
          rows_per_image: Some(dim),
        },
        wgpu::Extent3d {
          width:                 dim,
          height:                dim,
          depth_or_array_layers: 1,
        },
      );
    }
    queue.submit([]);

    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label:   Some("skybox layout"),
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding:    0,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty:         wgpu::BindingType::Texture {
            sample_type:    wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::Cube,
            multisampled:   false,
          },
          count:      None,
        },
        wgpu::BindGroupLayoutEntry {
          binding:    1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty:         wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count:      None,
        },
      ],
    });
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("cubemap sampler"),
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Nearest,
      ..Default::default()
    });
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label:   Some("skybox bg"),
      layout:  &layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding:  0,
          resource: wgpu::BindingResource::TextureView(&texture.create_view(
            &wgpu::TextureViewDescriptor {
              dimension: Some(wgpu::TextureViewDimension::Cube),
              ..Default::default()
            },
          )),
        },
        wgpu::BindGroupEntry {
          binding:  1,
          resource: wgpu::BindingResource::Sampler(&sampler),
        },
      ],
    });

    Self {
      texture,
      layout,
      group,
      sampler,
    }
  }

  pub fn texture(&self) -> &wgpu::Texture { &self.texture }

  pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout { &self.layout }

  pub fn bind_group(&self) -> &wgpu::BindGroup { &self.group }

  pub(crate) fn vertex_state<'a>(&self, shader: &'a ShaderEntryPoint) -> wgpu::VertexState<'a> {
    wgpu::VertexState {
      module:              &shader.module,
      entry_point:         shader.entry_point.as_deref(),
      compilation_options: Default::default(),
      buffers:             &[], // No vertex attributes
    }
  }
  pub(crate) fn fragment_state<'a>(&self, shader: &'a ShaderEntryPoint) -> wgpu::FragmentState<'a> {
    wgpu::FragmentState {
      module:              &shader.module,
      entry_point:         shader.entry_point.as_deref(),
      compilation_options: Default::default(),
      targets:             &[Some(wgpu::ColorTargetState {
        format:     wgpu::TextureFormat::Rgba8Unorm,
        blend:      None,
        write_mask: wgpu::ColorWrites::ALL,
      })],
    }
  }
}
