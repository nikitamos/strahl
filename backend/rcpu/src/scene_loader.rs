use crate::{
  Body, Geometry, GeometryTrait, Material, PointGlobal, Quad, Scene, Spectrum, Sphere,
  SurfaceProperty, TransformParts, TriangleMesh,
  camera::Camera,
  light::{LightEmissionDirection, LightSource},
  material::{
    ConcreteMaterial,
    TypeErasedMaterial,
    bsdf::{BSDF, dielectric::Dielectric, lambertian::Lambertian, specular::Specular}, // <-- Added Dielectric
    medium::Medium,
  },
};
use glam::{Quat, Vec3};
use serde::Deserialize;
use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  sync::Arc,
};

// =============================================================================
// UNTAGGED ENUMS: Reference OR Inline
// =============================================================================

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum GeometryRef {
  Named(String),
  Inline(GeometryDef),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum MaterialRef {
  Named(String),
  Inline(MaterialDef),
}

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
  Uniform([f32; 3]),
  Textured { texture: TextureRef },
}

impl From<SpectrumDef> for crate::Spectrum {
  fn from(def: SpectrumDef) -> Self {
    match def {
      SpectrumDef::Uniform([r, g, b]) => Vec3::new(r, g, b),
      SpectrumDef::Textured { .. } => Vec3::ONE,
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
    #[serde(flatten)]
    params: QuadParams,
  },

  // NEW: glTF Mesh support
  #[serde(rename = "mesh")]
  Mesh { path: String },
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct QuadParams {
  pub origin:  Option<[f32; 3]>,
  pub u:       Option<[f32; 3]>,
  pub v:       Option<[f32; 3]>,
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
  Lambertian {
    spectrum: SpectrumDef,
  },
  Specular {
    reflectance: SpectrumDef,
  },

  // NEW: Dielectric material (Glass, Water, etc.)
  Dielectric {
    transmission: SpectrumDef,
    #[serde(default = "default_reflection")]
    reflection:   SpectrumDef,
    #[serde(default = "default_ior")]
    ior:          f32,
  },

  Concrete {
    bsdf:   BsdfDef,
    medium: MediumDef,
  },
}

fn default_reflection() -> SpectrumDef { SpectrumDef::Uniform([1.0, 1.0, 1.0]) }
fn default_ior() -> f32 { 1.5 }

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BsdfDef {
  #[serde(rename = "lambertian")]
  Lambertian { spectrum: SpectrumDef },
  #[serde(rename = "specular")]
  Specular { reflectance: SpectrumDef },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MediumDef {
  #[serde(rename = "uniform")]
  Uniform { ior: f32 },
}

// =============================================================================
// SCENE STRUCTURES (Unchanged)
// =============================================================================

#[derive(Deserialize, Debug)]
pub struct SceneFile {
  #[serde(default)]
  pub scene:      SceneMeta,
  #[serde(default)]
  pub materials:  std::collections::HashMap<String, MaterialDef>,
  #[serde(default)]
  pub geometries: std::collections::HashMap<String, GeometryDef>,
  #[serde(default)]
  pub textures:   std::collections::HashMap<String, TextureDef>,
  pub bodies:     Vec<BodyDef>,
  #[serde(default)]
  pub lights:     Vec<LightDef>,
  pub camera:     Option<CameraDef>,
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
  pub rotation: [f32; 4],
}
fn default_quat() -> [f32; 4] { [0.0, 0.0, 0.0, 1.0] }

#[derive(Deserialize, Debug)]
pub struct LightDef {
  pub geometry:  GeometryRef,
  pub spectrum:  SpectrumDef,
  #[serde(default)]
  pub direction: LightDirectionDef,
  #[serde(default)]
  pub position:  [f32; 3],
}

#[derive(Deserialize, Debug, Default)]
#[serde(untagged)]
pub enum LightDirectionDef {
  #[default]
  #[serde(rename = "omni")]
  Omni,
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
  pub cam_type:   CameraType,
  #[serde(default = "default_fov")]
  pub fov:        f32,
}

#[derive(Deserialize, Debug, Default, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum CameraType {
  #[default]
  Perspective,
  Orthographic,
}
fn default_fov() -> f32 { 45.0 }

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TextureDef {
  Checkerboard {
    color1: [f32; 3],
    color2: [f32; 3],
    #[serde(default = "default_scale")]
    scale:  [f32; 2],
  },
}
fn default_scale() -> [f32; 2] { [1.0, 1.0] }

// =============================================================================
// ERRORS
// =============================================================================

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
// GLTF DATA EXTRACTION HELPERS
// =============================================================================

fn extract_vec3(view: &strahl_import::reader::GltfBufferView, buffer: &[u8]) -> Vec<Vec3> {
  let mut res = Vec::with_capacity(view.count);
  for i in 0..view.count {
    let offset = view.offset + i * view.stride;
    let bytes = &buffer[offset..offset + 12];
    let x = f32::from_le_bytes(bytes[0..4].try_into().unwrap());
    let y = f32::from_le_bytes(bytes[4..8].try_into().unwrap());
    let z = f32::from_le_bytes(bytes[8..12].try_into().unwrap());
    res.push(Vec3::new(x, y, z));
  }
  res
}

fn extract_vec2(view: &strahl_import::reader::GltfBufferView, buffer: &[u8]) -> Vec<glam::Vec2> {
  let mut res = Vec::with_capacity(view.count);
  for i in 0..view.count {
    let offset = view.offset + i * view.stride;
    let bytes = &buffer[offset..offset + 8];
    let x = f32::from_le_bytes(bytes[0..4].try_into().unwrap());
    let y = f32::from_le_bytes(bytes[4..8].try_into().unwrap());
    res.push(glam::Vec2::new(x, y));
  }
  res
}

fn extract_indices(geom: &strahl_import::reader::GltfGeometry) -> Vec<[u32; 3]> {
  let mut res = Vec::with_capacity(geom.indices.count / 3);
  let read_idx = |i: usize| -> u32 {
    let offset = geom.indices.offset + i * geom.indices.stride;
    match geom.index_size {
      1 => geom.buffer[offset] as u32,
      2 => u16::from_le_bytes(geom.buffer[offset..offset + 2].try_into().unwrap()) as u32,
      4 => u32::from_le_bytes(geom.buffer[offset..offset + 4].try_into().unwrap()),
      _ => panic!("Invalid index size"),
    }
  };
  for i in 0..(geom.indices.count / 3) {
    let i0 = read_idx(i * 3);
    let i1 = read_idx(i * 3 + 1);
    let i2 = read_idx(i * 3 + 2);
    res.push([i0, i1, i2]);
  }
  res
}

// =============================================================================
// SCENE LOADER
// =============================================================================

pub struct SceneLoader {
  geom_registry: HashMap<String, Arc<Geometry>>,
  mat_registry:  HashMap<String, Arc<dyn Material>>,
  source_path:   Option<PathBuf>,
}

impl Default for SceneLoader {
  fn default() -> Self { Self::new() }
}

impl SceneLoader {
  pub fn new() -> Self {
    Self {
      geom_registry: HashMap::new(),
      mat_registry:  HashMap::new(),
      source_path:   None,
    }
  }

  pub fn load(&mut self, path: impl AsRef<Path>) -> Result<Scene> {
    let path = path.as_ref();
    self.source_path = path.parent().map(|p| p.to_path_buf());

    let contents = std::fs::read_to_string(path)?;
    let scene_file: SceneFile = toml::from_str(&contents)?;

    self.register_definitions(&scene_file)?;

    let mut scene = Scene::new();
    for body_def in &scene_file.bodies {
      scene.push_body(self.build_body(body_def)?);
    }
    for light_def in &scene_file.lights {
      scene.push_light(self.build_light(light_def)?);
    }
    if let Some(cam_def) = &scene_file.camera {
      scene.cameras.push(Arc::new(self.build_camera(cam_def)?));
    }

    Ok(scene)
  }

  fn register_definitions(&mut self, defs: &SceneFile) -> Result<()> {
    for (name, geom_def) in &defs.geometries {
      let geom = self.build_geometry(geom_def)?; // Changed to self.
      self.geom_registry.insert(name.clone(), geom);
    }
    for (name, mat_def) in &defs.materials {
      let mat = Self::build_material(mat_def)?;
      self.mat_registry.insert(name.clone(), mat);
    }
    Ok(())
  }

  fn resolve_geometry(&self, geom_ref: &GeometryRef) -> Result<Arc<Geometry>> {
    match geom_ref {
      GeometryRef::Named(name) => self
        .geom_registry
        .get(name)
        .cloned()
        .ok_or_else(|| SceneLoadError::NotFound(format!("Unknown geometry: '{}'", name))),
      GeometryRef::Inline(def) => self.build_geometry(def), // Changed to self.
    }
  }

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

  fn build_geometry(&self, def: &GeometryDef) -> Result<Arc<Geometry>> {
    // Added &self
    match def {
      GeometryDef::Sphere { radius } => Ok(Arc::new(Sphere { radius: *radius }.into())),
      GeometryDef::Quad { params } => {
        if let (Some(origin), Some(u), Some(v)) = (params.origin, params.u, params.v) {
          Ok(Arc::new(Quad::new(
            Vec3::from(origin).into(),
            Vec3::from(u).into(),
            Vec3::from(v).into(),
          ).into()))
        } else if let (Some(variant), Some(center), Some(side)) =
          (params.variant, params.center, params.side)
        {
          let center = Vec3::from(center).into();
          let quad = match variant {
            QuadVariant::XySquare => Quad::xy_square(center, side),
            QuadVariant::YzSquare => Quad::yz_square(center, side),
            QuadVariant::ZxSquare => Quad::zx_square(center, side),
          };
          Ok(Arc::new(quad.into()))
        } else {
          Err(SceneLoadError::Parse(
            "Quad definition requires either (origin,u,v) or (variant,center,side)".into(),
          ))
        }
      }
      // NEW: glTF Mesh Loading
      GeometryDef::Mesh { path } => {
        let full_path = if let Some(ref base) = self.source_path {
          base.join(path)
        } else {
          PathBuf::from(path)
        };

        let geom = strahl_import::reader::GltfGeometry::import_validate(&full_path)
          .map_err(|e| SceneLoadError::Parse(format!("Failed to load glTF mesh: {}", e)))?;

        let vertices = extract_vec3(&geom.position, &geom.buffer);
        let normals = extract_vec3(&geom.normals, &geom.buffer);
        let uvs = extract_vec2(&geom.uv, &geom.buffer);
        let indices = extract_indices(&geom);

        Ok(Arc::new(TriangleMesh::new(
          vertices,
          indices,
          Some(normals),
          Some(uvs),
        ).into()))
      }
    }
  }

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
  fn build_medium(def: &MediumDef) -> Result<Arc<Medium>> {
    match def {
      MediumDef::Uniform { ior } => Ok(Arc::new(Medium { ior: *ior })),
    }
  }

  fn build_material(def: &MaterialDef) -> Result<Arc<dyn Material>> {
    match def {
      MaterialDef::Lambertian { spectrum } => {
        let spec = Self::parse_spectrum(spectrum)?;
        Ok(Arc::new(ConcreteMaterial {
          bsdf:   Lambertian { s: spec },
          medium: Medium { ior: 1.0 },
        }))
      }
      MaterialDef::Specular { reflectance } => {
        let spec = Self::parse_spectrum(reflectance)?;
        Ok(Arc::new(ConcreteMaterial {
          bsdf:   Specular { r: spec },
          medium: Medium { ior: 1.0 },
        }))
      }
      // NEW: Dielectric Material Construction
      MaterialDef::Dielectric {
        transmission,
        reflection,
        ior,
      } => {
        let t = Self::parse_spectrum(transmission)?;
        let r = Self::parse_spectrum(reflection)?;
        Ok(Arc::new(TypeErasedMaterial::new(
          Arc::new(Dielectric {
            transmission: t,
            reflection:   r,
          }),
          Arc::new(Medium { ior: *ior }),
        )))
      }
      MaterialDef::Concrete { bsdf, medium } => {
        let bsdf_obj = Self::build_bsdf(bsdf)?;
        let medium_obj = Self::build_medium(medium)?;
        Ok(Arc::new(TypeErasedMaterial::new(bsdf_obj, medium_obj)))
      }
    }
  }

  fn parse_spectrum(def: &SpectrumDef) -> Result<Spectrum> {
    match def {
      SpectrumDef::Uniform([r, g, b]) => Ok(Vec3::new(*r, *g, *b)),
      SpectrumDef::Textured { .. } => Ok(Vec3::ONE),
    }
  }

  fn build_body(&self, def: &BodyDef) -> Result<Body> {
    let geometry = self.resolve_geometry(&def.geometry)?;
    let material = self.resolve_material(&def.material)?;
    let coordinates = TransformParts {
      pos:      PointGlobal::new(def.position.into()),
      rotation: Quat::from_array(def.rotation),
    };

    Ok(Body::new(geometry, material, coordinates))
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
