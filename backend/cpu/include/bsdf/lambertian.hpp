#pragma once
#include "bxdf.hpp"
#include "spectrum.hpp"

namespace strahl::cpu {
class LambertianBxDF : public BxDF {
 public:
  explicit LambertianBxDF(Spectrum s) : s(s) {}
  float pdf(glm::vec3 out, glm::vec3 in) override;
  Spectrum f(glm::vec3 in, glm::vec3 out, float out_ior) override;
  SampledBxDF sampleExitent(glm::vec3 in, float uc, glm::vec2 u, float out_ior) const override;
  SampledBxDF sampleIncident(glm::vec3 out, float uc, glm::vec2 u, float out_ior) const override;
  Spectrum s;
};
}  // namespace strahl::cpu
