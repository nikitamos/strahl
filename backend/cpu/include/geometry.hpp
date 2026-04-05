#pragma once
#include <glm/glm.hpp>
#include <glm/gtc/quaternion.hpp>

namespace strahl::cpu {
class Geometry {
 public:
  Geometry() {}
};
class Plane : public Geometry {
 public:
  Plane(glm::vec3 origin, glm::vec3 normal) : origin_(origin), normal_(normal) {
    normal_ = glm::normalize(normal_);
    abcd_ = glm::vec4(normal_, -glm::dot(origin, normal_));
    denom_ = glm::length(normal_);
    assert(denom_ != 0 && "normal vector seems to be zero");
  }
  float distance(glm::vec3 point) { return glm::dot(glm::vec4(point, 1.0), abcd_) / denom_; }
  glm::vec3 getNormal(glm::vec3 /*point*/) { return normal_; }
  virtual ~Plane();

 private:
  glm::vec3 origin_;
  glm::vec3 normal_;
  glm::vec4 abcd_;
  float denom_;
};
class Sphere : public Geometry {
 public:
  float r;
};
class Mesh : public Geometry {
  class UVMap {};
};
}  // namespace strahl::cpu
