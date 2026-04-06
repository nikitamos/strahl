#pragma once
#include "nodes.hpp"
#include "scene.hpp"

namespace strahl::cpu {
class Solver {
 public:
  void render(Scene* s, Camera* cam);
};
}  // namespace strahl::cpu
