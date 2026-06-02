// POST-VIBECODING CHANGES:
// Remove division of returned sampled values by probability.


use glam::Vec3;

use crate::{
    material::bsdf::{BSDF, BSDFSampleContext, BsdfMetadata},
    Sample, Spectrum, VecHit,
};

/// BSDF that handles transmission (Snell's law) as well as reflection
/// (Fresnel effect)
pub struct Dielectric {
    pub transmission: Spectrum,
    /// Reflection color (usually Spectrum::ONE)
    pub reflection: Spectrum,
}

impl BSDF for Dielectric {
    fn sample_bsdf(
        &self,
        out: VecHit,
        u: crate::SampleState,
        ctx: &BSDFSampleContext,
    ) -> Option<crate::Sample<Spectrum, BsdfMetadata>> {
        let entering = out.z > 0.0;
        
        // NOTE: Compensating for InteractionMedium behavior as discussed previously
        let eta = if entering {
            ctx.interface.relative_ior
        } else {
            1.0 / ctx.interface.relative_ior
        };

        let cos_theta_i = out.z.abs();
        let sin_theta_i_sq = 1.0 - cos_theta_i * cos_theta_i;
        let sin_theta_t_sq = eta * eta * sin_theta_i_sq;

        // 1. Calculate Fresnel term (handles TIR automatically)
        let (fresnel, is_tir) = if sin_theta_t_sq >= 1.0 {
            (1.0, true) // Total Internal Reflection
        } else {
            let cos_theta_t = (1.0 - sin_theta_t_sq).sqrt();
            let r0 = ((1.0 - eta) / (1.0 + eta)).powi(2);
            (r0 + (1.0 - r0) * (1.0 - cos_theta_i).powi(5), false)
        };

        let reflect = is_tir || u.uniform_1d < fresnel; 

        if reflect {
            // Reflected direction in hit space (normal is +Z)
            let inc = Vec3::new(-out.x, -out.y, out.z).into();
            Some(Sample {
                prob: fresnel,
                sample: self.reflection, // Unbiased weight
                metadata: BsdfMetadata {
                    inc,
                    eta: 1.0, // Eta is irrelevant for reflection
                    dirac: true,
                    transmitted: false, // Crucial for solver!
                },
            })
        } else {
            // Refracted direction
            let cos_theta_t = (1.0 - sin_theta_t_sq).sqrt();
            let sign = if out.z > 0.0 { -1.0 } else { 1.0 };
            let inc = Vec3::new(-eta * out.x, -eta * out.y, sign * cos_theta_t).into();
            
            let prob_refract = (1.0 - fresnel).max(1e-5);
            Some(Sample {
                prob: prob_refract,
                sample: self.transmission,
                metadata: BsdfMetadata {
                    inc,
                    eta,
                    dirac: true,
                    transmitted: true,
                },
            })
        }
    }

    fn bsdf2(
        &self,
        out: VecHit,
        inc: VecHit,
        ctx: &BSDFSampleContext,
    ) -> Option<crate::Sample<Spectrum, BsdfMetadata>> {
        let entering = out.z > 0.0;
        let eta = if entering { ctx.interface.relative_ior } else { 1.0 / ctx.interface.relative_ior };
        
        let cos_theta_i = out.z.abs();
        let sin_theta_i_sq = 1.0 - cos_theta_i * cos_theta_i;
        let sin_theta_t_sq = eta * eta * sin_theta_i_sq;

        let fresnel = if sin_theta_t_sq >= 1.0 {
            1.0
        } else {
            let cos_theta_t = (1.0 - sin_theta_t_sq).sqrt();
            let r0 = ((1.0 - eta) / (1.0 + eta)).powi(2);
            r0 + (1.0 - r0) * (1.0 - cos_theta_i).powi(5)
        };

        // Check if `inc` matches the expected reflected direction
        let expected_reflect: VecHit = Vec3::new(-out.x, -out.y, out.z).into();
        if inc.distance_squared(*expected_reflect) < 1e-4 {
            return Some(Sample {
                prob: fresnel,
                sample: self.reflection / fresnel.max(1e-5),
                metadata: BsdfMetadata { inc: expected_reflect, eta: 1.0, dirac: true, transmitted: false },
            });
        }

        // Check if `inc` matches the expected refracted direction
        if sin_theta_t_sq < 1.0 {
            let cos_theta_t = (1.0 - sin_theta_t_sq).sqrt();
            let sign = if out.z > 0.0 { -1.0 } else { 1.0 };
            let expected_refract: VecHit = Vec3::new(-eta * out.x, -eta * out.y, sign * cos_theta_t).into();
            
            if inc.distance_squared(*expected_refract) < 1e-4 {
                let prob_refract = (1.0 - fresnel).max(1e-5);
                return Some(Sample {
                    prob: prob_refract,
                    sample: self.transmission / prob_refract,
                    metadata: BsdfMetadata { inc: expected_refract, eta, dirac: true, transmitted: true },
                });
            }
        }
        
        None
    }

    fn pdf(&self, out: VecHit, inc: VecHit, ctx: &BSDFSampleContext) -> f32 {
        self.bsdf2(out, inc, ctx).map(|s| s.prob).unwrap_or(0.0)
    }
}