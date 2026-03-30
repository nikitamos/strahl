#pragma once
#include <glm/gtc/quaternion.hpp>

#include "geometry.hpp"
#include "material.hpp"
#include "nodes.hpp"

namespace strahl::cpu {
class Scene {
 public:
  Scene(glm::vec3 translation, glm::quat rotation)
    : translation_(translation), rotation_(rotation) {}
  Camera* addCamera();
  Body* addBody(Geometry* g, Material* m);

 private:
  glm::vec3 translation_;
  glm::quat rotation_;
};
}  // namespace strahl::cpu
