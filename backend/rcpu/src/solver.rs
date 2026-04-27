use std::{fs::File, io::Write, ops::Neg};

use super::{Interaction, IntersectionContext, Scene, Spectrum};
use crate::{
  Body, Castable, PointGlobal, PointLocal, Sample, Sampler, VecGlobal,
  camera::{self, CameraRay},
  light::{LightSampleContext, LightSampleMetadata, LightSource},
  material::bsdf::BSDFSampleContext,
};
use glam::{Vec3, Vec3Swizzles, Vec4Swizzles};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

pub struct Solver {
  pub(crate) sampler: Sampler,
}

impl Solver {
  pub(crate) fn new() -> Self {
    Self {
      sampler: Sampler::default(),
    }
  }
  pub fn render(&self, scene: &Scene, cam: &mut camera::Camera) {
    let rays = cam.init_rays();
    const SAMPLES: i32 = 128;
    rays.into_par_iter().enumerate().for_each(|(_i, ray)| {
      for _ in 0..SAMPLES {
        self.trace_camera_ray(scene, ray);
        ray.reset_direction();
      }
      ray.color /= SAMPLES as f32;
    });
    #[cfg(false)]
    Self::dump_rays(rays);
  }

  fn dump_rays(rays: &[CameraRay]) {
    let mut file = File::create("scatter.csv").unwrap();
    for r in rays {
      let next = r.origin.xyz() + r.direction;
      let _ = writeln!(
        file,
        "{},{},{},{},{},{}",
        r.origin.x, r.origin.y, r.origin.z, next.x, next.y, next.z
      );
    }
  }

  fn trace_camera_ray(&self, scene: &Scene, ray: &mut CameraRay) {
    let mut throughput = Vec3::ONE;
    const BOUNCES: u32 = 1;
    for _b in 0..BOUNCES {
      let Some(isect) = Self::closest_hit(scene, ray) else {
        // Infinite lights
        // println!("no hit, exiting {b}");
        break;
      };
      // if emissive then L += beta * interaction.emission(-ray.dir) w.r.t. spectrum
      // if b == BOUNCES break;
      let bsdf = isect.body.material.bsdf();
      let cur_ray = isect.hit.to_hit((-ray.direction).into());
      if let Some(light) = self.sample_light(scene, isect.hit.point_global())
        && light.prob != 0.0
      {
        let wi = isect.hit.global_to_hit(
          (light.metadata.point.xyz() - isect.hit.point_global().xyz())
            .normalize()
            .into(),
        );
        if let Some(bsdf2) = bsdf.bsdf2(cur_ray, wi, BSDFSampleContext::Camera) {
          // In hit space wi.z corresponds to dot(wi, normal)
          // Note that if using normal mapping that's generally not the case.
          let f = bsdf2.sample * cur_ray.z.abs(); // is this really *that* angle?
          ray.color += throughput * f * light.sample / (light.prob) / 4.0;
        }
      }
      // Sample a new direction
      let Some(bs) = bsdf.sample_bsdf(cur_ray, self.sampler.sample(), BSDFSampleContext::Camera)
      else {
        break;
      };
      throughput *= bs.sample * bs.metadata.inc.z.abs() / bs.prob;
      ray.direction = isect.hit.to_global(bs.metadata.inc).normalize();
      ray.origin = (isect.hit.point_global().xyz() + ray.direction * 0.0001).into();
    }
  }
  fn sample_light(
    &self,
    scene: &Scene,
    dest: PointGlobal,
  ) -> Option<Sample<Spectrum, LightSampleMetadata>> {
    scene
      .sample_light_source(&self.sampler, dest)
      .and_then(|src, ()| {
        src.sample(self.sampler.sample(), LightSampleContext {
          dst: dest,
          scene,
        })
      })
  }
  pub(crate) fn hit_light(&self, scene: &Scene, ray: &dyn Castable) -> Option<Spectrum> {
    let light = &scene.lights[0];
    light.try_intersect(ray).map(|_hit| light.spectrum.get())
  }
  pub(crate) fn closest_hit<'a>(scene: &'a Scene, r: &impl Castable) -> Option<Interaction<'a>> {
    let mut x = 0;
    scene
      .bodies
      .iter()
      .filter_map(|b| {
        let ctx = IntersectionContext {
          transform: b.transform(),
        };
        b.geometry
          .try_intersect(ctx, r)
          .map(|hit| Interaction {
            body: b,
            ray_dir: hit.transform.v2local(r.direction()),
            hit,
          })
          .inspect(|_| x += 1)
      })
      .min_by(|a, b| a.hit.ray_distance.partial_cmp(&b.hit.ray_distance).unwrap())
      .inspect(|_| {
        if x > 1 {
          println!("{x} bodies intersected")
        } else {
        }
      })
  }
}

pub struct BDPTParams<LT: PathTerminator, ET: PathTerminator> {
  pub light_terminator: LT,
  pub eye_terminator:   ET,
  pub sample_count:     usize,
}

pub struct BidirectionalPathTracer<LT: PathTerminator, ET: PathTerminator> {
  params:  BDPTParams<LT, ET>, // TODO: cache of subpaths?
  sampler: Sampler,
}

impl<LT, ET> BidirectionalPathTracer<LT, ET>
where
  LT: PathTerminator,
  ET: PathTerminator,
{
  pub fn render(&self, scene: &Scene, camera: &mut camera::Camera) {
    let resolution = camera.resolution().as_uvec2();
    let rays = camera.init_rays();
    for s in 0..self.params.sample_count {
      rays.into_par_iter().enumerate().for_each(|(px, ray)| {
        let pixel = glam::uvec2(px as u32 % resolution.x, px as u32 / resolution.x);
        let path = self.generate_pixel_path(scene, ray, pixel);
        ray.color += path.eye.alpha;
        // based on path add to vector
        // so should we do something like
        // sounds to easy here
        // ray.color += path.mis_weight() * path.throughput()?
      });
    }
  }
  #[inline]
  fn generate_pixel_path<'s>(
    &self,
    scene: &'s Scene,
    ray: &mut CameraRay,
    pixel: glam::UVec2,
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

pub struct LightRay {
  origin:    PointGlobal,
  direction: VecGlobal,
}

impl Castable for LightRay {
  fn pos(&self) -> PointGlobal { self.origin }

  fn direction(&self) -> VecGlobal { self.direction }
}

pub enum PathSurface<'a> {
  Light(&'a LightSource),
  Body(&'a Body),
  Camera(glam::UVec2),
}

pub struct PathVertex<'a> {
  pub surface:  PathSurface<'a>,
  /// Point of the intersection of ray and scene
  pub point:    PointGlobal,
  /// Whether the BSDF at the vertex is specular
  pub specular: bool,
  #[deprecated(note = "use `radiance`")]
  pub alpha:    f32,
  /// Unconditional probability of generating subpath in this vertex
  pub prob:     f32,
  pub light:    VecGlobal,
  pub eye:      VecGlobal,
  pub radiance: Spectrum, // Hit-space vectors may be required
}

impl<'a> PathVertex<'a> {
  // TODO: add hit space vectors or normal to the PathVertex
  pub fn jacobian(&self) -> f32 { todo!() }
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
  light: Subpath<'a>,
  eye:   Subpath<'a>,
}

pub struct Subpath<'a> {
  pub vertices: Vec<PathVertex<'a>>,
  alpha:        Spectrum,
  p:            f32,
}

impl<'a> Subpath<'a> {
  pub fn len(&self) -> usize { self.vertices.len() }
}

impl<'a> BidirectionalPath<'a> {
  pub fn sample(
    scene: &'a Scene,
    sampler: &Sampler,
    camera_term: &(impl PathTerminator + ?Sized),
    light_term: &(impl PathTerminator + ?Sized),
    ray: &mut CameraRay,
  ) -> Self {
    let light = Self::sample_light_subpath(scene, sampler, light_term);
    let eye = Self::sample_eye_subpath(scene, sampler, ray, light_term, glam::UVec2::ZERO);
    todo!()
  }

  pub fn sample_light_subpath(
    scene: &'a Scene,
    sampler: &Sampler,
    term_cond: &(impl PathTerminator + ?Sized),
  ) -> Subpath<'a> {
    let source = scene.sample_any_light_source(sampler);
    let radiance = source
      .sample
      .sample_point_and_direction(sampler, LightSampleContext {
        dst: Vec3::ZERO.into(),
        scene,
      });

    let init = PathVertex {
      surface:  PathSurface::Light(source.sample),
      point:    radiance.metadata.point,
      specular: false, // TODO
      alpha:    1.0,
      prob:     1.0, //radiance.metadata.point_prob,
      eye:      radiance.metadata.direction,
      light:    Vec3::ZERO.into(),
      radiance: radiance.sample,
    };

    let mut radiance_next = radiance.sample / radiance.metadata.point_prob;
    let mut prob_next = radiance.metadata.point_prob;

    let mut path = vec![init];

    while let Some(last) = path.last()
      && !term_cond.should_terminate(path.len(), last, sampler)
    {
      // Cast ray and sample the BSDF
      let ray = LightRay {
        origin:    (last.point.xyz() + last.eye.xyz() * 1E-4).into(),
        direction: last.eye,
      };
      if let Some(intr) = Solver::closest_hit(scene, &ray) {
        let bsdf = intr.body.material.bsdf();
        let out = intr.hit.global_to_hit((-*ray.direction).into());
        if let Some(bs) = bsdf.sample_bsdf(out, sampler.sample(), BSDFSampleContext::Light) {
          let new_vert = PathVertex {
            surface:  PathSurface::Body(intr.body),
            point:    intr.hit.point_global(),
            specular: bs.metadata.dirac,
            alpha:    1.0, // todo!
            prob:     prob_next,
            light:    ray.direction.neg().into(),
            eye:      intr.hit.to_global(bs.metadata.inc.normalize().into()),
            radiance: radiance_next,
          };
          prob_next *= bs.prob * bs.metadata.jacobian_with(out);
          radiance_next *= /*last.radiance * */ bs.sample / bs.metadata.jacobian_with(out);
          path.push(new_vert);
        }
      } // else {TODO: infinite light?}
    }
    Subpath {
      vertices: path,
      alpha:    radiance_next,
      p:        prob_next,
    }
  }

  pub fn sample_eye_subpath(
    scene: &'a Scene,
    sampler: &Sampler,
    init_ray: &mut CameraRay,
    term_cond: &(impl PathTerminator + ?Sized),
    pixel: glam::UVec2,
  ) -> Subpath<'a> {
    const INITIAL_IMPORTANCE: Vec3 = Vec3::ONE;
    let init = PathVertex {
      surface:  PathSurface::Camera(pixel),
      point:    init_ray.origin,
      specular: false, // Depends on camera type, TODO
      alpha:    1.0,
      prob:     1.0,
      light:    init_ray.direction.into(),
      eye:      Vec3::ZERO.into(),
      radiance: INITIAL_IMPORTANCE, // TODO
    };

    let mut prob_next = 1.0; // Isn't camera specular in such case? (or should we get sth like 1/PIXELS)
    let mut importance_next = INITIAL_IMPORTANCE / prob_next;

    let mut path = vec![init];

    while let Some(last) = path.last()
      && !term_cond.should_terminate(path.len(), last, sampler)
    {
      init_ray.origin = (last.point.xyz() + last.eye.xyz() * 1E-4).into();
      init_ray.direction = last.light.into();
      // Cast ray and sample the BSDF
      if let Some(intr) = Solver::closest_hit(scene, init_ray) {
        let bsdf = intr.body.material.bsdf();
        let out = intr.hit.global_to_hit((-init_ray.direction).into());
        if let Some(bs) = bsdf.sample_bsdf(out, sampler.sample(), BSDFSampleContext::Camera) {
          let new_vert = PathVertex {
            surface:  PathSurface::Body(intr.body),
            point:    intr.hit.point_global(),
            specular: bs.metadata.dirac,
            alpha:    1.0, // todo!
            prob:     prob_next,
            eye:      init_ray.direction.neg().into(),
            light:    intr.hit.to_global(bs.metadata.inc.normalize().into()),
            radiance: importance_next,
          };
          prob_next = bs.prob * bs.metadata.jacobian_with(out);
          importance_next = last.radiance * bs.sample / bs.metadata.jacobian_with(out);
          path.push(new_vert);
        }
      } // else {TODO: infinite light?}
    }
    Subpath {
      vertices: path,
      alpha:    importance_next,
      p:        prob_next,
    }
  }

  pub fn mis_weight(&self) -> f32 { todo!() } // or should we return spectrum?
  pub fn throughput(&self) -> Spectrum {
    match (self.eye.len(), self.light.len()) {
      (1, _) => {
        todo!()
      }
      (_, 1) => {
        todo!()
      }
      (_, _) => {
        todo!()
      }
    }
  }
}
