#pragma once

#include "glm/glm.hpp"

namespace strahl {
struct Material {
  glm::vec3 color;
  float diffuse = 0.65;
  float specular = 0.35;
  float emittance = 0.0;
  float alpha = 2.0;
  static const Material kEmpty;
};
} // namespace strahl