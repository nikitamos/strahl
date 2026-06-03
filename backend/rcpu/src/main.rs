use glam::Vec3;
use rcpu::{
  RayTracer, Scene,
  camera::Camera,
  scene_loader::{SceneLoadError, SceneLoader},
};

#[cfg(false)]
mod bdpt_legacy {
  struct UniformLEqPath {
    n:    usize,
    prob: f32,
  }

  impl UniformLEqPath {
    fn new(n: usize, prob: f32) -> Self { Self { n, prob } }
  }

  impl PathTerminator for UniformLEqPath {
    fn should_terminate(
      &self,
      vertices: usize,
      _gen_vertex: &rcpu::solver::bdpt::PathVertex,
      sampler: &Sampler,
    ) -> bool {
      sampler.sample().uniform_1d >= self.prob || self.n <= vertices
    }
  }
}

const IMAGE_SIDE: u32 = 512;

const DEPTH: u32 = 6;
const SAMPLES: u32 = 4;

fn main() {
  let back = RayTracer::new();
  let (scene, camera) = read_scene_from_file().unwrap();

  let camera = camera.unwrap_or_else(|| {
    Camera::new_with_fov(
      (IMAGE_SIDE as usize, IMAGE_SIDE as usize).into(),
      Vec3::NEG_Y,
      78f32.to_radians(),
      Vec3::Z,
      (7.0 * Vec3::Y + 4.2 * Vec3::Z).into(),
      rcpu::camera::CameraType::Perspective,
    )
  });
  solve2(back, scene, camera);
}

fn read_scene_from_file() -> Result<(Scene, Option<Camera>), SceneLoadError> {
  let mut loader = SceneLoader::new();
  loader.load("test-scene.toml")
}

fn solve2(back: RayTracer, scene: rcpu::Scene, mut cam: Camera) {
  let solver = back.create_solver2(DEPTH, SAMPLES);
  solver.render(&scene, &mut cam);
  let (width, height) = cam.resolution().into();
  let mut img = image::Rgb32FImage::new(width as u32, height as u32);
  cam.write_image(&mut img);
  img.save(format!("solver-{DEPTH}-x{SAMPLES}.tiff")).unwrap();
}
