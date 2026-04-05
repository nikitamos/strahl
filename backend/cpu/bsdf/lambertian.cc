#include "bsdf/lambertian.hpp"

#include <numbers>

#include "bsdf/bxdf.hpp"
#include "dist-samplers.hpp"
#include "spectrum.hpp"

namespace strahl::cpu {
namespace n = std::numbers;
float LambertianBxDF::pdf(glm::vec3 out, glm::vec3 in) {}
Spectrum LambertianBxDF::f(glm::vec3 in, glm::vec3 out, float out_ior) {
  return sameHemisphere(in, out) ? s : Spectrum(s.size()) * n::inv_pi;
}
SampledBxDF LambertianBxDF::sampleExitent(glm::vec3 in, float uc, glm::vec2 u, float out_ior) const {}
SampledBxDF LambertianBxDF::sampleIncident(glm::vec3 out, float uc, glm::vec2 u, float out_ior) const {
  glm::vec3 in = dist::cosHemisphere(u);
  if (out.z < 0) {
    in.z *= -1;
  }
  float pdf = std::abs(in.z) * n::inv_pi;

  return SampledBxDF{
    .pdf = pdf,
    .incident = in,
    .exitent = out,
    .f = s * n::inv_pi,
  };
}
}  // namespace strahl::cpu
