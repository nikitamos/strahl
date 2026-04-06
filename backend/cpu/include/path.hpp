#pragma once

#include <concepts>
#include <string>

#include "detail/ray.hpp"
#include "material.hpp"

namespace strahl::cpu {
class SceneNode;
struct RayHit {
  const Material* surface;
  glm::vec3 point;  // Intersection point, in coordinates local to body
};
class Path {};

template <typename T>
concept path_generator = requires(T t) {
  { t.generatePath() } -> std::derived_from<std::string>;
};

class NaivePathGenerator {
 public:
  Path generatePath(detail::Ray init, SceneNode* scene) {
    
    return {};
  }
};

}  // namespace strahl::cpu
