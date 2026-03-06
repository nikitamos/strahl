#pragma once
#include "raymarcher.hpp"
#include <cassert>
#include <glm/geometric.hpp>
#include <glm/glm.hpp>

namespace strahl::cpu_raymarcher {
class Ray {
public:
  Ray(glm::vec3 pos, glm::vec3 dir, const CpuRaymarcherBackendOptions &opts)
      : cur_pos_(pos), direction_(dir), opts_(opts) {
    assert(0.9999f <= glm::length(dir) && glm::length(dir) <= 1.00001f);
  }
  void Advance(float length);
  void Backtrack();

  glm::vec3 GetPos() const { return cur_pos_; }
  void SetColor(glm::vec3 col) { color_ = col; }

private:
  void Reflect(glm::vec3 normal);

  glm::vec3 cur_pos_;
  glm::vec3 direction_;
  glm::vec3 color_;
  int bounces_ = 0;
  const CpuRaymarcherBackendOptions &opts_;
};
} // namespace strahl::cpu_raymarcher
