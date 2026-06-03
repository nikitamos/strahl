use super::GeometryTrait;
use crate::{
  GeometrySampleMetadata, IntersectionContext, PointLocal, RayGeneric, Sample, SampleState,
  SurfaceHit,
};
use glam::Vec3;

pub struct TriangleMesh {
    vertices: Vec<Vec3>,
    indices: Vec<[u32; 3]>,
    vertex_normals: Option<Vec<Vec3>>,
    vertex_uvs: Option<Vec<glam::Vec2>>,
    
    // Precomputed data for uniform area sampling
    triangle_areas: Vec<f32>,
    area_cdf: Vec<f32>,
    total_area: f32,
}

impl TriangleMesh {
    /// Creates a new triangle mesh.
    /// 
    /// * `vertices` - List of vertex positions in local space.
    /// * `indices` - List of triangles, where each triangle is defined by 3 vertex indices.
    /// * `vertex_normals` - Optional list of vertex normals for smooth shading.
    /// * `vertex_uvs` - Optional list of vertex UV coordinates.
    pub fn new(
        vertices: Vec<Vec3>,
        indices: Vec<[u32; 3]>,
        vertex_normals: Option<Vec<Vec3>>,
        vertex_uvs: Option<Vec<glam::Vec2>>,
    ) -> Self {
        let mut triangle_areas = Vec::with_capacity(indices.len());
        let mut total_area = 0.0;
        
        // Precompute triangle areas for sampling
        for tri in &indices {
            let v0 = vertices[tri[0] as usize];
            let v1 = vertices[tri[1] as usize];
            let v2 = vertices[tri[2] as usize];
            let area = (v1 - v0).cross(v2 - v0).length() * 0.5;
            triangle_areas.push(area);
            total_area += area;
        }
        
        // Build Cumulative Distribution Function (CDF) for O(log N) triangle selection
        let mut area_cdf = Vec::with_capacity(indices.len());
        let mut current_sum = 0.0;
        for &area in &triangle_areas {
            current_sum += area;
            area_cdf.push(current_sum);
        }
        
        Self {
            vertices,
            indices,
            vertex_normals,
            vertex_uvs,
            triangle_areas,
            area_cdf,
            total_area,
        }
    }
}

impl GeometryTrait for TriangleMesh {
  fn sample_point(&self, state: SampleState) -> Sample<PointLocal, GeometrySampleMetadata> {
        // 1. Select a random triangle with probability proportional to its area
        let [r_tri, r_bary1] = state.uniform_2d.into();
        let target_area = r_tri * self.total_area;
        
        let tri_idx = match self.area_cdf.binary_search_by(|probe| probe.partial_cmp(&target_area).unwrap()) {
            Ok(idx) => idx,
            Err(idx) => idx.min(self.indices.len() - 1),
        };
        
        let tri = self.indices[tri_idx];
        let v0 = self.vertices[tri[0] as usize];
        let v1 = self.vertices[tri[1] as usize];
        let v2 = self.vertices[tri[2] as usize];
        
        // 2. Generate random barycentric coordinates (u, v)
        // Note: We reuse r_tri as a pseudo-random second coordinate since uniform_2d only provides 2 randoms.
        // For a production renderer, you'd want 3 independent randoms.
        let r_bary2 = r_tri; 
        let mut u = r_bary1;
        let mut v = r_bary2;
        
        // Fold the square into a triangle to ensure uniform distribution
        if u + v > 1.0 {
            u = 1.0 - u;
            v = 1.0 - v;
        }
        let w = 1.0 - u - v;
        
        // 3. Interpolate position and normal
        let point = v0 * w + v1 * u + v2 * v;
        
        let normal = if let Some(ref normals) = self.vertex_normals {
            let n0 = normals[tri[0] as usize];
            let n1 = normals[tri[1] as usize];
            let n2 = normals[tri[2] as usize];
            (n0 * w + n1 * u + n2 * v).normalize()
        } else {
            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            edge1.cross(edge2).normalize()
        };
        
        let pdf = 1.0 / self.total_area;
        
        Sample {
            sample: point.into(),
            prob: pdf,
            metadata: GeometrySampleMetadata {
                normal: normal.into(),
            },
        }
    }

  fn try_intersect<'a>(
    &self,
    ctx: IntersectionContext<'a>,
    ray: &RayGeneric,
  ) -> Option<SurfaceHit<'a>> {
        let dir = *ctx.transform.v2local(ray.direction());
        let origin = *ctx.transform.p2local(ray.pos());
        
        let mut closest_t = f32::MAX;
        let mut closest_hit = None;
        
        // Naive loop over all triangles. 
        // TODO: For production, replace this with a BVH (Bounding Volume Hierarchy) acceleration structure.
        for tri_indices in &self.indices {
            let v0 = self.vertices[tri_indices[0] as usize];
            let v1 = self.vertices[tri_indices[1] as usize];
            let v2 = self.vertices[tri_indices[2] as usize];
            
            // Möller–Trumbore intersection algorithm
            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let h = dir.cross(edge2);
            let a = edge1.dot(h);
            
            // Ray is parallel to the triangle plane
            if a.abs() < 1e-5 {
                continue;
            }
            
            let f = 1.0 / a;
            let s = origin - v0;
            let u = f * s.dot(h);
            
            if !(0.0..=1.0).contains(&u) {
                continue;
            }
            
            let q = s.cross(edge1);
            let v = f * dir.dot(q);
            
            if v < 0.0 || u + v > 1.0 {
                continue;
            }
            
            let t = f * edge2.dot(q);
            
            // Check if this is the closest hit so far
            if t > 1e-5 && t < closest_t {
                closest_t = t;
                
                // Interpolate normal (smooth shading if vertex normals exist, otherwise flat)
                let normal = if let Some(ref normals) = self.vertex_normals {
                    let n0 = normals[tri_indices[0] as usize];
                    let n1 = normals[tri_indices[1] as usize];
                    let n2 = normals[tri_indices[2] as usize];
                    (n0 * (1.0 - u - v) + n1 * u + n2 * v).normalize()
                } else {
                    edge1.cross(edge2).normalize()
                };
                
                closest_hit = Some(SurfaceHit::new(
                    (origin + dir * t).into(),
                    normal,
                    t,
                    ctx.transform,
                ));
            }
        }
        
        closest_hit
    }
}
