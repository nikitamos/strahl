#pragma once
#include <glm/gtc/quaternion.hpp>

#include "geometry.hpp"
#include "material.hpp"
#include "nodes.hpp"

namespace strahl::cpu {
class Scene {
 public:
  Scene(glm::vec3 translation, glm::quat rotation) {}
  Camera* addCamera();
  Body* addBody(Geometry* g, Material* m);

 private:
};
}  // namespace strahl::cpu
