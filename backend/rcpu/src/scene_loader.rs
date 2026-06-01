use crate::{
  Body, Geometry, Material, PointGlobal, Quad, Scene, Spectrum, Sphere, SurfaceProperty, TransformParts, camera::Camera, light::{LightEmissionDirection, LightSource}, material::{
    ConcreteMaterial, TypeErasedMaterial,
    bsdf::{BSDF, lambertian::Lambertian, specular::Specular},
    medium::{Medium, UniformMedium},
  }
};
use glam::{Quat, Vec3};
use serde::Deserialize;
use std::{collections::HashMap, path::{Path, PathBuf}, sync::Arc};

// =============================================================================
// UNTAGGED ENUMS: Reference OR Inline
// =============================================================================

/// Resolves to either a named geometry or an inline definition
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum GeometryRef {
  /// Reference to a named geometry in [geometries] section
  Named(String),
  /// Inline geometry definition
  Inline(GeometryDef),
}

/// Resolves to either a named material or an inline definition  
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum MaterialRef {
  /// Reference to a named material in [materials] section
  Named(String),
  /// Inline material definition
  Inline(MaterialDef),
}

/// Resolves to either a named texture or an inline definition
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum TextureRef {
  Named(String),
  Inline(TextureDef),
}

// =============================================================================
// SPECTRUM: Uniform or Textured
// =============================================================================

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SpectrumDef {
  /// Uniform color: [r, g, b]
  Uniform([f32; 3]),
  /// Textured: { texture = "name" } or inline texture def
  Textured { texture: TextureRef },
}

impl From<SpectrumDef> for crate::Spectrum {
  fn from(def: SpectrumDef) -> Self {
    match def {
      SpectrumDef::Uniform([r, g, b]) => Vec3::new(r, g, b),
      SpectrumDef::Textured { .. } => {
        // TODO: Implement texture sampling
        Vec3::ONE
      }
    }
  }
}

// =============================================================================
// GEOMETRY DEFINITIONS
// =============================================================================

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum GeometryDef {
  #[serde(rename = "sphere")]
  Sphere { radius: f32 },

  #[serde(rename = "quad")]
  Quad {
    // Arbitrary parallelogram
    #[serde(flatten)]
    params: QuadParams,
  },
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct QuadParams {
  // Parallelogram definition
  pub origin: Option<[f32; 3]>,
  pub u:      Option<[f32; 3]>,
  pub v:      Option<[f32; 3]>,

  // Convenience square variants
  pub variant: Option<QuadVariant>,
  pub center:  Option<[f32; 3]>,
  pub side:    Option<f32>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum QuadVariant {
  XySquare,
  YzSquare,
  ZxSquare,
}

// =============================================================================
// MATERIAL DEFINITIONS
// =============================================================================

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MaterialDef {
  /// Simple material: just a BSDF
  Lambertian {
    spectrum: SpectrumDef,
  },
  Specular {
    reflectance: SpectrumDef,
  },

  /// Full material with BSDF + Medium
  Concrete {
    bsdf:   BsdfDef,
    medium: MediumDef,
  },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BsdfDef {
  #[serde(rename = "lambertian")]
  Lambertian { spectrum: SpectrumDef },

  #[serde(rename = "specular")]
  Specular { reflectance: SpectrumDef },
  // Future extensions:
  // Glossy { spectrum: SpectrumDef, roughness: f32 },
  // Transmissive { spectrum: SpectrumDef, ior: f32 },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MediumDef {
  #[serde(rename = "uniform")]
  Uniform { ior: f32 },
}

// =============================================================================
// SCENE STRUCTURES
// =============================================================================

#[derive(Deserialize, Debug)]
pub struct SceneFile {
  #[serde(default)]
  pub scene: SceneMeta,

  #[serde(default)]
  pub materials: std::collections::HashMap<String, MaterialDef>,

  #[serde(default)]
  pub geometries: std::collections::HashMap<String, GeometryDef>,

  #[serde(default)]
  pub textures: std::collections::HashMap<String, TextureDef>,

  pub bodies: Vec<BodyDef>,

  #[serde(default)]
  pub lights: Vec<LightDef>,

  pub camera: Option<CameraDef>,
}

#[derive(Deserialize, Debug, Default)]
pub struct SceneMeta {
  pub name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct BodyDef {
  pub geometry: GeometryRef,
  pub material: MaterialRef,

  #[serde(default)]
  pub position: [f32; 3],

  #[serde(default = "default_quat")]
  pub rotation: [f32; 4], // [x, y, z, w] for glam::Quat
}

fn default_quat() -> [f32; 4] { [0.0, 0.0, 0.0, 1.0] }

#[derive(Deserialize, Debug)]
pub struct LightDef {
  pub geometry: GeometryRef,
  pub spectrum: SpectrumDef,

  #[serde(default)]
  pub direction: LightDirectionDef,

  #[serde(default)]
  pub position: [f32; 3],
}

#[derive(Deserialize, Debug, Default)]
#[serde(untagged)]
pub enum LightDirectionDef {
  #[default]
  #[serde(rename = "omni")]
  Omni, // Deserializes from string "omni"

  #[serde(rename_all = "lowercase")]
  Structured {
    #[serde(flatten)]
    inner: LightEmissionDef,
  },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LightEmissionDef {
  Omni,
  Directed { direction: [f32; 3] },
  Spot { direction: [f32; 3], cutoff: f32 },
}

#[derive(Deserialize, Debug)]
pub struct CameraDef {
  pub resolution: [usize; 2],
  pub position:   [f32; 3],
  pub direction:  [f32; 3],
  pub right:      [f32; 3],

  #[serde(default)]
  #[serde(rename = "type")]
  pub cam_type: CameraType,

  #[serde(default = "default_fov")]
  pub fov: f32,
}

#[derive(Deserialize, Debug, Default, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum CameraType {
  #[default]
  Perspective,
  Orthographic,
}

fn default_fov() -> f32 { 45.0 }

// =============================================================================
// TEXTURE DEFINITIONS (stub for future)
// =============================================================================

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TextureDef {
  Checkerboard {
    color1: [f32; 3],
    color2: [f32; 3],
    #[serde(default = "default_scale")]
    scale:  [f32; 2],
  },
  // Image { path: String, ... },
}

fn default_scale() -> [f32; 2] { [1.0, 1.0] }

#[derive(Debug)]
pub enum SceneLoadError {
  Io(std::io::Error),
  Toml(toml::de::Error),
  Parse(String),
  NotFound(String),
  Unsupported(String),
}

impl std::fmt::Display for SceneLoadError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      SceneLoadError::Io(e) => write!(f, "IO error: {}", e),
      SceneLoadError::Toml(e) => write!(f, "TOML parse error: {}", e),
      SceneLoadError::Parse(e) => write!(f, "Parse error: {}", e),
      SceneLoadError::NotFound(e) => write!(f, "Not found: {}", e),
      SceneLoadError::Unsupported(e) => write!(f, "Unsupported: {}", e),
    }
  }
}

impl std::error::Error for SceneLoadError {}

impl From<std::io::Error> for SceneLoadError {
  fn from(e: std::io::Error) -> Self { SceneLoadError::Io(e) }
}

impl From<toml::de::Error> for SceneLoadError {
  fn from(e: toml::de::Error) -> Self { SceneLoadError::Toml(e) }
}

type Result<T> = std::result::Result<T, SceneLoadError>;

// =============================================================================
// SCENE LOADER
// =============================================================================

pub struct SceneLoader {
  geom_registry: HashMap<String, Arc<dyn Geometry>>,
  mat_registry:  HashMap<String, Arc<dyn Material>>,
  source_path:   Option<PathBuf>,
}

impl Default for SceneLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneLoader {
  pub fn new() -> Self {
    Self {
      geom_registry: HashMap::new(),
      mat_registry:  HashMap::new(),
      source_path:   None,
    }
  }

  /// Load a scene from a TOML file at `path`
  pub fn load(&mut self, path: impl AsRef<Path>) -> Result<Scene> {
    let path = path.as_ref();
    self.source_path = path.parent().map(|p| p.to_path_buf());

    // Read and parse TOML
    let contents = std::fs::read_to_string(path)?;
    let scene_file: SceneFile = toml::from_str(&contents)?;

    // First pass: register named definitions
    self.register_definitions(&scene_file)?;

    // Build the scene
    let mut scene = Scene::new();

    // Add bodies
    for body_def in &scene_file.bodies {
      let body = self.build_body(body_def)?;
      scene.push_body(body);
    }

    // Add lights
    for light_def in &scene_file.lights {
      let light = self.build_light(light_def)?;
      scene.push_light(light);
    }

    // Configure camera if present
    if let Some(cam_def) = &scene_file.camera {
      let cam = self.build_camera(cam_def)?;
      scene.cameras.push(Arc::new(cam));
    }

    Ok(scene)
  }

  /// First pass: populate registries from named definitions
  fn register_definitions(&mut self, defs: &SceneFile) -> Result<()> {
    for (name, geom_def) in &defs.geometries {
      let geom = Self::build_geometry(geom_def)?;
      self.geom_registry.insert(name.clone(), geom);
    }

    for (name, mat_def) in &defs.materials {
      let mat = Self::build_material(mat_def)?;
      self.mat_registry.insert(name.clone(), mat);
    }

    Ok(())
  }

  /// Resolve GeometryRef -> Arc<dyn Geometry>
  fn resolve_geometry(&self, geom_ref: &GeometryRef) -> Result<Arc<dyn Geometry>> {
    match geom_ref {
      GeometryRef::Named(name) => self
        .geom_registry
        .get(name)
        .cloned()
        .ok_or_else(|| SceneLoadError::NotFound(format!("Unknown geometry: '{}'", name))),
      GeometryRef::Inline(def) => Self::build_geometry(def),
    }
  }

  /// Resolve MaterialRef -> Arc<dyn Material>
  fn resolve_material(&self, mat_ref: &MaterialRef) -> Result<Arc<dyn Material>> {
    match mat_ref {
      MaterialRef::Named(name) => self
        .mat_registry
        .get(name)
        .cloned()
        .ok_or_else(|| SceneLoadError::NotFound(format!("Unknown material: '{}'", name))),
      MaterialRef::Inline(def) => Self::build_material(def),
    }
  }

  /// Build runtime Geometry from GeometryDef
  fn build_geometry(def: &GeometryDef) -> Result<Arc<dyn Geometry>> {
    match def {
      GeometryDef::Sphere { radius } => Ok(Arc::new(Sphere { radius: *radius })),
      GeometryDef::Quad { params } => {
        // Prefer explicit parallelogram params
        if let (Some(origin), Some(u), Some(v)) = (params.origin, params.u, params.v) {
          Ok(Arc::new(Quad::new(
            Vec3::from(origin).into(),
            Vec3::from(u).into(),
            Vec3::from(v).into(),
          )))
        }
        // Fall back to convenience square variants
        else if let (Some(variant), Some(center), Some(side)) =
          (params.variant, params.center, params.side)
        {
          let center = Vec3::from(center).into();
          let quad = match variant {
            QuadVariant::XySquare => Quad::xy_square(center, side),
            QuadVariant::YzSquare => Quad::yz_square(center, side),
            QuadVariant::ZxSquare => Quad::zx_square(center, side),
          };
          Ok(Arc::new(quad))
        } else {
          Err(SceneLoadError::Parse(
            "Quad definition requires either (origin,u,v) or (variant,center,side)".into(),
          ))
        }
      }
    }
  }

  /// Build runtime BSDF from BsdfDef
  fn build_bsdf(def: &BsdfDef) -> Result<Arc<dyn BSDF>> {
    match def {
      BsdfDef::Lambertian { spectrum } => {
        let spec = Self::parse_spectrum(spectrum)?;
        Ok(Arc::new(Lambertian { s: spec }))
      }
      BsdfDef::Specular { reflectance } => {
        let spec = Self::parse_spectrum(reflectance)?;
        Ok(Arc::new(Specular { r: spec }))
      }
    }
  }

  /// Build runtime Medium from MediumDef
  fn build_medium(def: &MediumDef) -> Result<Arc<dyn Medium>> {
    match def {
      MediumDef::Uniform { ior } => Ok(Arc::new(UniformMedium { ior: *ior })),
    }
  }

  /// Build runtime Material from MaterialDef
  fn build_material(def: &MaterialDef) -> Result<Arc<dyn Material>> {
    match def {
      MaterialDef::Lambertian { spectrum } => {
        let spec = Self::parse_spectrum(spectrum)?;
        Ok(Arc::new(ConcreteMaterial {
          bsdf:   Lambertian { s: spec },
          medium: UniformMedium { ior: 1.0 },
        }))
      }
      MaterialDef::Specular { reflectance } => {
        let spec = Self::parse_spectrum(reflectance)?;
        Ok(Arc::new(ConcreteMaterial {
          bsdf:   Specular { r: spec },
          medium: UniformMedium { ior: 1.0 },
        }))
      }
      MaterialDef::Concrete { bsdf, medium } => {
        let bsdf_obj = Self::build_bsdf(bsdf)?;
        let medium_obj = Self::build_medium(medium)?;

        Ok(Arc::new(TypeErasedMaterial::new(bsdf_obj, medium_obj)))
      }
    }
  }

  /// Helper: parse SpectrumDef into runtime Spectrum (Vec3)
  fn parse_spectrum(def: &SpectrumDef) -> Result<Spectrum> {
    match def {
      SpectrumDef::Uniform([r, g, b]) => Ok(Vec3::new(*r, *g, *b)),
      SpectrumDef::Textured { .. } => {
        // Textures not yet implemented; return white as fallback
        Ok(Vec3::ONE)
      }
    }
  }

  /// Convert BodyDef -> runtime Body
  fn build_body(&self, def: &BodyDef) -> Result<Body> {
    let geometry = self.resolve_geometry(&def.geometry)?;
    let material = self.resolve_material(&def.material)?;

    let coordinates = TransformParts {
      pos:      PointGlobal::new(def.position.into()),
      rotation: Quat::from_array(def.rotation),
    };

    Ok(Body {
      geometry,
      material,
      coordinates: std::sync::nonpoison::RwLock::new(coordinates),
    })
  }

  /// Build LightSource from LightDef
  fn build_light(&self, def: &LightDef) -> Result<LightSource> {
    let geometry = self.resolve_geometry(&def.geometry)?;
    let spectrum = Self::parse_spectrum(&def.spectrum)?;
    let surface_prop = SurfaceProperty::Uniform(spectrum);

    let dir = match &def.direction {
      LightDirectionDef::Omni => LightEmissionDirection::Omni,
      LightDirectionDef::Structured { inner } => match inner {
        LightEmissionDef::Omni => LightEmissionDirection::Omni,
        LightEmissionDef::Directed { direction } => {
          LightEmissionDirection::Directed(Vec3::from(*direction).into())
        }
        LightEmissionDef::Spot { direction, cutoff } => {
          LightEmissionDirection::Spot(Vec3::from(*direction).into(), *cutoff)
        }
      },
    };

    Ok(LightSource::new(
      geometry,
      surface_prop,
      def.position.into(),
      dir,
    ))
  }

  /// Build Camera from CameraDef
  fn build_camera(&self, def: &CameraDef) -> Result<Camera> {
    Ok(Camera::new(
      def.resolution.into(),
      Vec3::from(def.direction),
      Vec3::from(def.right),
      PointGlobal::new(def.position.into()),
      match def.cam_type {
        CameraType::Perspective => crate::camera::CameraType::Perspective,
        CameraType::Orthographic => crate::camera::CameraType::Orthographic,
      },
    ))
  }
}
