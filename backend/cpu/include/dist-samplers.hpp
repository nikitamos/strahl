#pragma once
#include <cmath>
#include <glm/glm.hpp>
#include <numbers>

/// Sorcery
namespace strahl::cpu::dist {
namespace n = std::numbers;
inline static glm::vec2 uniformDiskConcentric(glm::vec2 u) {
  glm::vec2 u_oft = 2.0f * u - glm::vec2(1, 1);
  if (u_oft.x == 0 && u_oft.y == 0) {
    return {0, 0};
  }

  float theta;
  float r;
  if (std::abs(u_oft.x) > std::abs(u_oft.y)) {
    r = u_oft.x;
    theta = n::pi / 4.0 * (u_oft.y / u_oft.x);
  } else {
    r = u_oft.y;
    theta = n::pi / 2 - n::pi / 4 * (u_oft.x / u_oft.y);
  }
  return r * glm::vec2(std::cos(theta), std::sin(theta));
}
inline static glm::vec3 cosHemisphere(glm::vec2 u) {
  glm::vec2 d = uniformDiskConcentric(u);
  float z = std::sqrt(1 - d.x * d.x - d.y * d.y);
  return {d, z};
}  // namespace dist
}  // namespace strahl::cpu::dist
