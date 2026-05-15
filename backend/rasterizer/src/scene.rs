use std::sync::{Arc, RwLock};

use glam::{Mat4, Vec3, Vec3Swizzles};
use wgpu::RenderPipeline;
use zerocopy::IntoBytes;

use crate::{
  Camera,
  geometry::Geometry,
  material::Material,
  shader_manager::ShaderManager,
  skybox::{self, Skybox},
};

#[repr(C)]
#[derive(Clone)]
pub struct BodyData {
  pub rotation:    glam::Quat,
  pub translation: glam::Vec3,
  pub scale:       f32,
}

pub struct Body {
  geometry: Arc<Geometry>,
  material: Arc<Material>,
  pipeline: wgpu::RenderPipeline,
  // TODO: determine which RwLock should be used hare
  data:     RwLock<BodyData>,
}

impl Body {
  pub(crate) fn new(
    geometry: Arc<Geometry>,
    material: Arc<Material>,
    manager: &ShaderManager,
  ) -> Self {
    Self {
      pipeline: manager.create_pipeline_for_mesh_geometry(&material, &geometry),
      geometry,
      material,
      data: RwLock::new(BodyData {
        scale:       1.0,
        rotation:    glam::Quat::IDENTITY,
        translation: glam::Vec3::ZERO,
      }),
    }
  }

  /// Renders the body into provided render pass.
  /// This function assumes that the global uniform buffer
  /// (group 0) is already bound by caller.
  pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(1, self.material.bind_group(), &[]);
    pass.set_bind_group(2, self.geometry.bind_group(), &[]);
    let transform = self.transform();
    pass.set_immediates(0, transform.as_bytes());
    pass.set_immediates(
      std::mem::size_of_val(&transform) as u32,
      self.translation().as_bytes(),
    );
    self.geometry.setup_attributes(pass);
    self.geometry.dispatch_draw(pass);
  }

  fn transform(&self) -> Mat4 {
    Mat4::from_translation(self.translation())
      * Mat4::from_quat(self.rotation())
      * Mat4::from_scale(glam::Vec3::splat(self.scale()))
  }

  pub fn rotation(&self) -> glam::Quat { self.data.read().unwrap().rotation }
  pub fn set_rotation(&self, rotation: glam::Quat) {
    self.data.write().unwrap().rotation = rotation;
  }
  pub fn translation(&self) -> glam::Vec3 { self.data.read().unwrap().translation }
  pub fn set_translation(&self, translation: glam::Vec3) {
    self.data.write().unwrap().translation = translation;
  }
  pub fn scale(&self) -> f32 { self.data.read().unwrap().scale }
  pub fn set_scale(&self, scale: f32) { self.data.write().unwrap().scale = scale; }

  pub fn material(&self) -> &Material { &self.material }
  pub fn geometry(&self) -> &Geometry { &self.geometry }

  pub fn display_location(&self, camera: &Camera, viewport: glam::UVec2) -> Option<glam::UVec2> {
    let transform = self.transform();
    let origin = transform.transform_point3a(glam::Vec3A::ZERO);
    let ndc = camera
      .projection
      .project_point3a(camera.camera.project_point3a(origin));
    map_ndc(viewport, ndc)
  }

  pub fn display_radius(&self, camera: &Camera, viewport: glam::UVec2) -> f32 {
    let transform = self.transform();
    let origin = transform.transform_point3a(glam::Vec3A::ZERO);
    let scale = self.scale();

    let view_pos = camera.camera.project_point3a(origin);
    let depth = view_pos.z.abs();

    let proj_scale = camera.projection.col(1).y;

    (scale * proj_scale / depth) * (viewport.y as f32 * 0.5)
  }

  pub fn location_radius(
    &self,
    camera: &Camera,
    viewport: glam::UVec2,
  ) -> (Option<glam::UVec2>, f32) {
    let transform = self.transform();
    let origin = transform.transform_point3a(glam::Vec3A::ZERO);
    let ndc = camera
      .projection
      .project_point3a(camera.camera.project_point3a(origin));

    let scale = self.scale();
    let view_pos = camera.camera.project_point3a(origin);
    let depth = view_pos.z.abs();
    let proj_scale = camera.projection.col(1).y;

    (
      map_ndc(viewport, ndc),
      (scale * proj_scale / depth) * (viewport.y as f32 * 0.5),
    )
  }
}

/// Maps normalized device coordinates to viewport cooridinates
fn map_ndc(viewport: glam::UVec2, ndc: glam::Vec3A) -> Option<glam::UVec2> {
  if 0.0 <= ndc.z && ndc.z <= 1.0 && ndc.x.abs() <= 1.0 && ndc.y.abs() <= 1.0 {
    let viewport_mapped = 0.5 * (ndc.xy() * glam::vec2(1.0, -1.0) + glam::Vec2::ONE);
    let viewport = glam::vec2(viewport.x as f32, viewport.y as f32);
    let res = viewport * viewport_mapped;
    Some(glam::uvec2(res.x as u32, res.y as u32))
  } else {
    None
  }
}

pub struct Scene {
  bodies:  Vec<Arc<Body>>,
  manager: Arc<ShaderManager>,
  skybox:  Option<(Arc<Skybox>, wgpu::RenderPipeline)>,
}

impl Scene {
  pub(crate) fn new(manager: Arc<ShaderManager>) -> Self {
    Self {
      bodies: vec![],
      manager,
      skybox: None,
    }
  }
  pub fn set_skybox(&mut self, skybox: Arc<Skybox>) {
    let pipeline = self.manager.create_pipeline_for_skybox(&skybox);
    self.skybox = Some((skybox, pipeline));
  }
  pub fn take_skybox(&mut self, skybox: Arc<Skybox>) -> Option<Arc<Skybox>> {
    self.skybox.take().map(|x| x.0)
  }
  pub fn draw_skybox(&self, pass: &mut wgpu::RenderPass) {
    if let Some((skybox, pipeline)) = &self.skybox {
      pass.set_pipeline(pipeline);
      pass.set_bind_group(1, skybox.bind_group(), &[]);
      pass.draw(0..36, 0..1);
    }
  }
  pub fn add_body(&mut self, geometry: Arc<Geometry>, material: Arc<Material>) -> Arc<Body> {
    let res = Arc::new(Body::new(geometry, material, &self.manager));
    self.bodies.push(res.clone());
    res
  }
  pub fn remove_body(&mut self, body: &Arc<Body>) {
    let item = self
      .bodies
      .iter()
      .find(|x| Arc::as_ptr(x) == Arc::as_ptr(&body));
    if let Some(element) = item {
      let index = self.bodies.element_offset(element).unwrap();
      self.bodies.remove(index);
    }
  }
  pub fn bodies(&self) -> &[Arc<Body>] { &self.bodies }
}
