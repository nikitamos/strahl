#pragma once
#include <glm/glm.hpp>
#include <glm/gtc/quaternion.hpp>

namespace strahl::cpu {
class Geometry {
 public:
  Geometry() {}
};
class Plane : public Geometry {};
class Sphere : public Geometry {};
class Mesh : public Geometry {
  class UVMap {};
};
}  // namespace strahl::cpu
