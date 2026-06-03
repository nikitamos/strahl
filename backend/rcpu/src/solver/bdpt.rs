use std::marker::PhantomData;

use glam::Vec3;
use rayon::prelude::*;

use crate::{
  Interaction, PointGlobal, RayGeneric, Sample, Sampler, Scene, Spectrum, VecGlobal,
  camera::{self, CameraRay},
  light::{LightSampleContext, LightSource},
  material::bsdf::{BSDFSampleContext, BsdfMetadata},
};

pub struct BDPTParams<LT: PathTerminator, ET: PathTerminator> {
  pub light_terminator: LT,
  pub eye_terminator:   ET,
  pub sample_count:     usize,
}

pub struct BidirectionalPathTracer<LT: PathTerminator, ET: PathTerminator> {
  pub(crate) params:  BDPTParams<LT, ET>, // TODO: cache of subpaths?
  pub(crate) sampler: Sampler,
}

pub(crate) type BSDFSample = Sample<Spectrum, BsdfMetadata>;

pub(crate) struct SubpathConfig<'a, P: PathTerminator + ?Sized> {
  pub(crate) scene:          &'a Scene,
  pub(crate) sampler:        &'a Sampler,
  pub(crate) term_cond:      &'a P,
  pub(crate) init_vertex:    PathVertex<'a>,
  pub(crate) init_direction: VecGlobal,
  pub(crate) init_radiance:  Spectrum,
  pub(crate) init_prob:      f32,
  pub(crate) bsdf_context:   BSDFSampleContext<'a>,
}

pub(crate) fn sample_subpath<'a, P>(cfg: SubpathConfig<'a, P>) -> Subpath<'a>
where P: PathTerminator + ?Sized {
  let mut radiance = cfg.init_radiance;
  let mut prob = cfg.init_prob;
  let mut path = vec![cfg.init_vertex];
  let mut last_ray_out = cfg.init_direction;

  while let Some(last) = path.last()
    && !cfg
      .term_cond
      .should_terminate(path.len(), last, cfg.sampler)
  {
    let ray = RayGeneric {
      position:  last.point,
      direction: last_ray_out,
    };
    if let Some(intr) = super::closest_hit(cfg.scene, &ray) {
      let bsdf = intr.body.material.bsdf();
      let out = intr.hit.global_to_hit(-ray.direction() );
      last_ray_out = intr.hit.to_global(out);
      let jac = cfg.scene.geom_factor_skip(
        last.point,
        last.normal,
        intr.hit.point_global(),
        intr.hit.normal_global(),
      );
      if let Some(bs) = bsdf.sample_bsdf(out, cfg.sampler.sample(), &cfg.bsdf_context) {
        let bs_jac = bs.metadata.jacobian_with(out);
        let total_jac = jac.x * bs_jac;
        let (new_prob, new_radiance) = (bs.prob * total_jac, radiance * bs.sample * total_jac);
        let new_vert = PathVertex {
          point: intr.hit.point_global(),
          specular: bs.metadata.dirac,
          prob,
          radiance,
          normal: intr.hit.normal_global(),
          light: cfg.bsdf_context.light_direction(&intr, &bs.metadata),
          eye: cfg.bsdf_context.light_direction(&intr, &bs.metadata),
          surface: PathSurface::from_interaction(intr),
        };

        prob = new_prob;
        radiance = new_radiance;
        path.push(new_vert);
      }
    }
  }

  Subpath {
    vertices: path,
    alpha:    radiance,
    p:        prob,
  }
}

impl<LT, ET> BidirectionalPathTracer<LT, ET>
where
  LT: PathTerminator,
  ET: PathTerminator,
{
  pub(crate) fn new(params: BDPTParams<LT, ET>, sampler: Sampler) -> Self {
    Self { params, sampler }
  }
  pub fn render(&self, scene: &Scene, camera: &mut camera::Camera) {
    // Probably here we should generate a lot of light paths and then re-use
    let resolution = camera.resolution().as_uvec2();
    let rays = camera.init_rays();
    rays.into_par_iter().enumerate().for_each(|(px, ray)| {
      ray.color = Spectrum::ZERO;
      for _s in 0..self.params.sample_count {
        let mut r = ray.clone();
        let pixel = glam::uvec2(px as u32 % resolution.x, px as u32 / resolution.x);
        let path = self.generate_pixel_path(scene, &mut r, pixel);
        ray.color += path.throughput(scene) / (self.params.sample_count as f32);
        // based on path add to vector
        // so should we do something like
        // sounds to easy here
        // ray.color += path.mis_weight() * path.throughput()?
      }
    });
  }
  #[inline]
  pub(crate) fn generate_pixel_path<'s>(
    &'s self,
    scene: &'s Scene,
    ray: &'s mut CameraRay,
    _pixel: glam::UVec2,
  ) -> BidirectionalPath<'s> {
    BidirectionalPath::sample(
      scene,
      &self.sampler,
      &self.params.eye_terminator,
      &self.params.light_terminator,
      ray,
    )
  }
}

pub type LightRay = RayGeneric;

pub enum PathSurface<'a> {
  Light(&'a LightSource),
  Body(Interaction<'a>),
  Camera(glam::UVec2), // TODO: replace pixel with world pos + direction
}

impl<'a> Default for PathSurface<'a> {
  fn default() -> Self { Self::Camera(Default::default()) }
}

impl<'a> PathSurface<'a> {
  pub fn from_interaction(intr: Interaction<'a>) -> Self { Self::Body(intr) }
}

#[derive(Default)]
pub struct PathVertex<'a> {
  pub surface:  PathSurface<'a>,
  /// Point of the intersection of ray and scene
  pub point:    PointGlobal,
  /// Whether the BSDF at the vertex is specular
  pub specular: bool,
  /// Unconditional probability of generating subpath in this vertex
  pub prob:     f32,
  pub light:    VecGlobal,
  pub eye:      VecGlobal,
  pub radiance: Spectrum, // Hit-space vectors may be required
  pub normal:   VecGlobal,
}

pub trait PathTerminator: Sync {
  fn should_terminate(&self, vertices: usize, gen_vertex: &PathVertex, sampler: &Sampler) -> bool;
}

impl<T> PathTerminator for T
where T: Fn(usize, &PathVertex, &Sampler) -> bool + Sync
{
  fn should_terminate(&self, length: usize, last_vertex: &PathVertex, sampler: &Sampler) -> bool {
    self.call((length, last_vertex, sampler))
  }
}

impl PathTerminator for usize {
  fn should_terminate(&self, length: usize, _last_vertex: &PathVertex, _sampler: &Sampler) -> bool {
    length >= *self
  }
}

pub struct BidirectionalPath<'a> {
  pub(crate) light: Subpath<'a>,
  pub(crate) eye:   Subpath<'a>,
}

pub struct Subpath<'a> {
  pub vertices:     Vec<PathVertex<'a>>,
  pub(crate) alpha: Spectrum,
  pub(crate) p:     f32,
}

impl<'a> Subpath<'a> {
  pub fn len(&self) -> usize { self.vertices.len() }
}

impl<'a> BidirectionalPath<'a> {
  pub fn sample(
    scene: &'a Scene,
    sampler: &'a Sampler,
    camera_termination: &'a (impl PathTerminator + ?Sized),
    light_termination: &'a (impl PathTerminator + ?Sized),
    ray: &'a mut CameraRay,
  ) -> Self {
    let light = Self::sample_light_subpath(scene, sampler, light_termination);
    let eye = Self::sample_eye_subpath(scene, sampler, ray, camera_termination, glam::UVec2::ZERO);
    Self { light, eye }
  }

  pub fn sample_light_subpath(
    scene: &'a Scene,
    sampler: &'a Sampler,
    term_cond: &'a (impl PathTerminator + ?Sized),
  ) -> Subpath<'a> {
    let source = scene.sample_any_light_source(sampler);
    let radiance = source
      .sample
      .sample_point_and_direction(sampler, LightSampleContext {
        dst: Vec3::ZERO.into(),
        scene,
      });

    let init_vertex = PathVertex {
      surface:  PathSurface::Light(source.sample),
      point:    radiance.metadata.point,
      specular: false,
      prob:     1.0,
      eye:      radiance.metadata.direction,
      light:    Vec3::ZERO.into(),
      radiance: radiance.sample,
      normal:   source.sample.transform().v2world(radiance.metadata.normal),
    };

    sample_subpath(SubpathConfig {
      scene,
      sampler,
      term_cond,
      init_vertex,
      init_radiance: radiance.sample,
      init_prob: radiance.metadata.point_prob,
      bsdf_context: BSDFSampleContext::Light, // TODO
      init_direction: radiance.metadata.direction,
    })
  }

  pub fn sample_eye_subpath(
    scene: &'a Scene,
    sampler: &'a Sampler,
    init_ray: &'a mut CameraRay,
    term_cond: &'a (impl PathTerminator + ?Sized),
    pixel: glam::UVec2,
  ) -> Subpath<'a> {
    pub(crate) const INITIAL_IMPORTANCE: Vec3 = Vec3::ONE;
    let init_vertex = PathVertex {
      surface:  PathSurface::Camera(pixel),
      point:    init_ray.origin,
      specular: false,
      prob:     1.0,
      light:    init_ray.direction.into(),
      eye:      Vec3::ZERO.into(),
      radiance: INITIAL_IMPORTANCE,
      normal:   init_ray.direction.into(),
    };

    sample_subpath(SubpathConfig {
      scene,
      sampler,
      term_cond,
      init_vertex,
      init_radiance: INITIAL_IMPORTANCE,
      init_prob: 1.0,
      bsdf_context: BSDFSampleContext::Camera, // TODO
      init_direction: init_ray.direction.into(),
    })
  }

  pub fn mis_weight(&self) -> f32 { todo!() } // or should we return spectrum?
  /// Evaluates the throughput along the given path
  /// # Panics
  /// This function will panic if ...
  pub fn throughput(&self, scene: &Scene) -> Spectrum {
    let cst = self.join_coeff(scene);
    cst * self.eye.alpha * self.light.alpha
  }

  pub(crate) fn join_coeff(&self, scene: &Scene) -> Spectrum {
    match (self.eye.len(), self.light.len()) {
      (1, 1) => Spectrum::ZERO,
      (1, _) => {
        // FIXME: it doesn't work properly.
        // One point at the eye and non-degenerate light path
        let _eye = self.eye.vertices.last().unwrap();
        let _light = self.light.vertices.last().unwrap();
        // Spectrum::splat(
        //   eye
        //     .light
        //     .normalize()
        //     .dot((eye.point - light.point).normalize())
        //     .abs()
        //     .clamp(0.5, 1.0),
        // )
        Spectrum::ZERO
      }
      (_, 1) => {
        // One point at the light and non-degenerate eye path
        let eye = self.eye.vertices.last().unwrap();
        let light = self.light.vertices.last().unwrap();
        let PathSurface::Light(src) = light.surface else {
          unreachable!()
        };
        let direction = src.transform().v2local(eye.point - light.point);
        let point = src.transform().p2local(light.point);
        
        src.emitted_radiance(point, direction, src.transform().v2local(light.normal))
      }
      (_, _) => {
        let y = self.eye.vertices.last().unwrap();
        let z = self.light.vertices.last().unwrap();
        let PathSurface::Body(ref intr) = y.surface else {
          unreachable!()
        };
        let y2z = z.point - y.point;
        let bsdf_light = intr.body.material.bsdf().bsdf(
          intr.hit.global_to_hit(pz(z.light)),
          intr.hit.global_to_hit(pz(y2z)),
          BSDFSampleContext::Light,
        );
        let PathSurface::Body(ref intr) = y.surface else {
          unreachable!()
        };
        let bsdf_eye = intr.body.material.bsdf().bsdf(
          intr.hit.global_to_hit(pz(y2z)),
          intr.hit.global_to_hit(pz(y.eye)),
          BSDFSampleContext::Camera,
        );
        let g = scene.geom_factor_vis(y.point, y.normal, z.point, z.normal);
        bsdf_eye * bsdf_light * g
      }
    }
  }
}

pub(crate) fn pz(mut v: VecGlobal) -> VecGlobal {
  if v.z < 0.0 {
    v.z = -v.z;
  }
  v
}

pub struct SampledVertex<'a> {
  /// Direction of outgoing castable
  pub(crate) outgoing_dir: VecGlobal,
  pub(crate) position:     PointGlobal,
  pub(crate) normal:       VecGlobal,
  pub(crate) phantom:      PhantomData<&'a ()>,
}

impl<'a> SampledVertex<'a> {
  pub fn ray(&self) -> RayGeneric {
    RayGeneric {
      position:  self.position,
      direction: self.outgoing_dir,
    }
  }
}
