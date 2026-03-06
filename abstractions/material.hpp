#include "glm/glm.hpp"

namespace strahl {
struct Material {
  glm::vec3 color;
  float roughness;
  static const Material kEmpty;
};
} // namespace strahl