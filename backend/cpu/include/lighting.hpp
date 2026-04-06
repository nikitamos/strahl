#include <optional>

#include "geometry.hpp"
#include "nodes.hpp"
#include "spectrum.hpp"

namespace strahl::cpu {
struct DiffuseAreaLight {
  Spectrum l(glm::vec3  /*p*/, glm::vec3  /*n*/) {
    return scale * le;
  }
  Geometry* g;
  Spectrum le;
  float scale = 1.0;
};

struct SampledLight {
  const Light *l;
  float pmf = 1.0;
};

class LightSampler {
 public:
  std::optional<SampledLight> sampleLight(float u) {
    if (lights_.empty()) {
      return {};
    }
    int light_index = std::min<int>((int)(u * (float)lights_.size()), (int)lights_.size() - 1);
    return SampledLight{&lights_[light_index], 1.f / (float)lights_.size()};
  }

  float pmf(Light light) const {
    if (lights_.empty()) {
      return 0;
    }
    return 1.f / (float)lights_.size();
  }

 private:
  std::span<Light> lights_;
};

}  // namespace strahl::cpu