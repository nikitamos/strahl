#pragma once
#include <glm/glm.hpp>
#include <glm/gtc/quaternion.hpp>
#include <optional>

#include "detail/ray.hpp"
#include "path.hpp"

namespace strahl::cpu {
class Geometry {
 public:
  Geometry() {}
  /// Returns the intersection point in coordinate system local to the geometry.
  /// Origin and direction of the ray are also expressed in the geometry coordinates.
  virtual std::optional<glm::vec3> intersect(const detail::Ray& ray) = 0;
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
  virtual ~Plane() {}
  std::optional<glm::vec3> intersect(const detail::Ray& ray) override;

 private:
  glm::vec3 origin_;
  glm::vec3 normal_;
  glm::vec4 abcd_;
  float denom_;
};
class Sphere : public Geometry {
 public:
  float r;
  std::optional<glm::vec3> intersect(const detail::Ray& ray) override;
};
class Mesh : public Geometry {
  class UVMap {};
};
}  // namespace strahl::cpu
