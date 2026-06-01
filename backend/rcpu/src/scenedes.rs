use crate::{
  Body, Geometry, Material, Quad, Sphere, TransformParts,
  material::{ConcreteMaterial, bsdf::lambertian::Lambertian},
};
use glam::{Quat, Vec3};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

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

pub struct SceneLoader {
  geom_registry: HashMap<String, Arc<dyn Geometry>>,
  mat_registry:  HashMap<String, Arc<dyn Material>>,
}

impl SceneLoader {
  pub fn new() -> Self {
    Self {
      geom_registry: HashMap::new(),
      mat_registry:  HashMap::new(),
    }
  }

  /// First pass: populate registries from named definitions
  pub fn register_definitions(&mut self, defs: &SceneFile) -> Result<(), String> {
    // Register geometries
    for (name, geom_def) in &defs.geometries {
      let geom = Self::build_geometry(geom_def)?;
      self.geom_registry.insert(name.clone(), geom);
    }

    // Register materials
    for (name, mat_def) in &defs.materials {
      let mat = Self::build_material(mat_def)?;
      self.mat_registry.insert(name.clone(), mat);
    }

    Ok(())
  }

  /// Resolve GeometryRef -> Arc<dyn Geometry>
  pub fn resolve_geometry(&self, geom_ref: &GeometryRef) -> Result<Arc<dyn Geometry>, String> {
    match geom_ref {
      GeometryRef::Named(name) => self
        .geom_registry
        .get(name)
        .cloned()
        .ok_or_else(|| format!("Unknown geometry: '{}'", name)),
      GeometryRef::Inline(def) => Self::build_geometry(def),
    }
  }

  /// Resolve MaterialRef -> Arc<dyn Material>
  pub fn resolve_material(&self, mat_ref: &MaterialRef) -> Result<Arc<dyn Material>, String> {
    match mat_ref {
      MaterialRef::Named(name) => self
        .mat_registry
        .get(name)
        .cloned()
        .ok_or_else(|| format!("Unknown material: '{}'", name)),
      MaterialRef::Inline(def) => Self::build_material(def),
    }
  }

  /// Build runtime Geometry from GeometryDef
  fn build_geometry(def: &GeometryDef) -> Result<Arc<dyn Geometry>, String> {
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
          Err("Quad definition requires either (origin,u,v) or (variant,center,side)".into())
        }
      }
    }
  }

  /// Build runtime Material from MaterialDef
  fn build_material(def: &MaterialDef) -> Result<Arc<dyn Material>, String> {
    match def {
      MaterialDef::Lambertian { spectrum } => {
        let spec: crate::Spectrum = spectrum.clone().into();
        Ok(Arc::new(ConcreteMaterial {
          bsdf:   Lambertian { s: spec },
          medium: crate::material::medium::UniformMedium { ior: 1.0 },
        }))
      }
      MaterialDef::Specular { reflectance } => {
        // TODO: Implement Specular material construction
        let _spec: crate::Spectrum = reflectance.clone().into();
        Err("Specular material inline definition not yet implemented".into())
      }
      MaterialDef::Concrete { bsdf, medium } => {
        // Delegate to BSDF builder, then wrap with medium
        let _bsdf = Self::build_bsdf(bsdf)?;
        let _med = Self::build_medium(medium)?;
        // TODO: Combine into ConcreteMaterial
        Err("Concrete material with custom BSDF+Medium not yet implemented".into())
      }
    }
  }

  fn build_bsdf(_def: &BsdfDef) -> Result<Arc<dyn crate::material::bsdf::BSDF>, String> {
    // TODO: Implement BSDF construction
    unimplemented!()
  }

  fn build_medium(_def: &MediumDef) -> Result<Arc<dyn crate::material::medium::Medium>, String> {
    // TODO: Implement Medium construction
    unimplemented!()
  }

  /// Convert BodyDef -> runtime Body
  pub fn build_body(&self, def: &BodyDef) -> Result<Body, String> {
    let geometry = self.resolve_geometry(&def.geometry)?;
    let material = self.resolve_material(&def.material)?;

    let coordinates = TransformParts {
      pos:      crate::PointGlobal::new(def.position.into()),
      rotation: Quat::from_array(def.rotation),
    };

    Ok(Body {
      geometry,
      material,
      coordinates: std::sync::nonpoison::RwLock::new(coordinates),
    })
  }
}
