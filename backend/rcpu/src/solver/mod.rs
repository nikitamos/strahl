use std::{fs::File, io::Write};

use super::{Interaction, IntersectionContext, Scene, Spectrum};
use crate::{
  Castable, PointGlobal, Sample, Sampler,
  camera::CameraRay,
  light::{LightSampleContext, LightSampleMetadata},
};
use glam::Vec3Swizzles;

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

fn sample_light(
  sampler: &Sampler,
  scene: &Scene,
  dest: PointGlobal,
) -> Option<Sample<Spectrum, LightSampleMetadata>> {
  scene
    .sample_light_source(sampler, dest)
    .and_then(|src, ()| src.sample(sampler.sample(), LightSampleContext { dst: dest, scene }))
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
}

#[cfg(false)]
pub mod bdpt;

mod solver2;
pub use solver2::ForwardPathTracer;
