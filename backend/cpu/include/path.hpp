#pragma once

#include <concepts>
#include <string>

#include "detail/ray.hpp"
#include "material.hpp"
#include "nodes.hpp"

namespace strahl::cpu {
struct Interaction {
  const Material& surface;
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
