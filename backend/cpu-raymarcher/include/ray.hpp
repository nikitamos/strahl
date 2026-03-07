#pragma once
#include "backend.hpp"
#include "material.hpp"
#include "raymarcher.hpp"
#include <cassert>
#include <glm/fwd.hpp>
#include <glm/geometric.hpp>
#include <glm/glm.hpp>

namespace strahl::cpu_raymarcher {
struct RayEnvironment {
  glm::vec3 color;
  float ior;
};
class Ray {
public:
  Ray(glm::vec3 pos, glm::vec3 dir, const CpuRaymarcherBackendOptions &opts)
      : cur_pos_(pos), direction_(dir), opts_(opts) {
    assert(0.9999f <= glm::length(dir) && glm::length(dir) <= 1.00001f);
  }
  void Advance(float length);
  void Backtrack();
  void Intersect(Material m, glm::vec3 normal);

  auto GetBounces() const { return bounces_; }
  glm::vec3 GetPos() const { return cur_pos_; }
  void SetColor(glm::vec3 col) { color_ = col; }
  glm::vec3 GetColor() const { return color_; }

private:
  void Reflect(glm::vec3 normal);
  void MixColor(Material m, glm::vec3 light_dir, glm::vec3 eye,
                glm::vec3 normal);

  glm::vec3 cur_pos_;
  glm::vec3 direction_;
  glm::vec3 color_;
  int bounces_ = 0;
  float multiple_ = 1.0;
  glm::vec3 cur_specular_;
  const CpuRaymarcherBackendOptions &opts_;
};
} // namespace strahl::cpu_raymarcher
