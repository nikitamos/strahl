#pragma once

#include <zlib.h>

#include <glm/glm.hpp>
#include <memory>

#include "spectrum.hpp"

namespace strahl::cpu {
static inline const size_t kSpectrumSize = 3;
struct SampledBxDF {
  float pdf;
  glm::vec3 incident;  // Direction of incident light
  glm::vec3 exitent;   // Direction of exitent light
  Spectrum f;          // Sampled values (per wavelength
};

class BxDF {
 public:
  static bool sameHemisphere(glm::vec3 w, glm::vec3 wp) { return w.z * wp.z > 0; }
  // should normal always point outwards?
  virtual Spectrum f(glm::vec3 in, glm::vec3 out, float out_ior) = 0;
  // To allow BdRT wi
  virtual SampledBxDF sampleIncident(glm::vec3 out, float uc, glm::vec2 u, float out_ior) const = 0;
  virtual SampledBxDF sampleExitent(glm::vec3 in, float uc, glm::vec2 u, float out_ior) const = 0;
  virtual float pdf(glm::vec3 out, glm::vec3 in) = 0;
  Spectrum rho(
    glm::vec3 wo, std::span<const float> uc, std::span<const glm::vec2> u2, float out_ior) const {
    Spectrum s(3);
    for (size_t i = 0; i < uc.size(); ++i) {
      auto bs = sampleIncident(wo, uc[i], u2[i], out_ior);
      s += absCosTheta(bs.incident) * bs.f / bs.pdf;
    }
    return s / (float)uc.size();
  }
  static float absCosTheta(glm::vec3 v) { return std::abs(v.z); }
};


/* It seems to be just type-erasure wrapper */
#if 0
class BSDF {
 public:
  Spectrum f(glm::vec3 out_object, glm::vec3 in_object, float out_ior = 1.0) const {
    glm::vec3 wi = RenderToLocal(in_object);
    glm::vec3 wo = RenderToLocal(out_object);
    // if (wo.z == 0) {
    //   return Spectrum(kSpectrumSize);
    // }
    return bxdf->f(wo, wi, out_ior);
  }
  SampledBxDF sampleF(glm::vec3 out_object, float u, glm::vec2 u2, float out_ior = 0) const {
    glm::vec3 wo = RenderToLocal(out_object);
    SampledBxDF bs = bxdf->sampleIncident(wo, u, u2, out_ior);
    // What's wrong with wi.z == 0?
    bs.incident = LocalToRender(bs.incident);
    return bs;
  }
  float pdf(glm::vec3 out_object, glm::vec3 in_object) const {
    glm::vec3 wo = RenderToLocal(out_object);
    glm::vec3 wi = RenderToLocal(in_object);
    // if (wo.z == 0) {
    //   return 0;
    // }
    return bxdf.pdf(wo, wi);
  }
  Spectrum rho(
    glm::vec3 out, std::span<const float> uc, std::span<const glm::vec2> u2, float out_ior) const {
    return bxdf->rho(out, uc, u2, out_ior);
  }
  std::unique_ptr<BxDF> bxdf;
};
#endif
}  // namespace strahl::cpu