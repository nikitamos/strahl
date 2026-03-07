#pragma once

#include "collision-context.hpp"
#include "glm/glm.hpp"

namespace strahl {
struct Material {
  glm::vec3 color;
  float diffuse = 0.5;
  float specular = 0.5;
  float emittance = 0.0;
  float alpha = 2.0;
  static const Material kEmpty;
};
} // namespace strahl