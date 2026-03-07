#include "include/ray.hpp"
#include <glm/geometric.hpp>

namespace strahl::cpu_raymarcher {
void Ray::Advance(float distance) {
  assert(bounces_ < opts_.bounces);
  cur_pos_ += direction_ * distance;
}
void Ray::Intersect(Material m, glm::vec3 normal) {
  assert(bounces_ < opts_.bounces);
  if (bounces_ == 0) {
    SetColor(m.color);
  } else {
    MixColor(m.color);
  }
  // For now let consider all materials absolutely reflective
  Reflect(normal);
  bounces_++;
}
void Ray::Reflect(glm::vec3 normal) {
  direction_ -= 2 * glm::dot(direction_, normal) * normal;
  cur_pos_ += 2 * 0.01f * direction_;
}
void Ray::MixColor(glm::vec3 new_color) {
  color_ += new_color;
  color_ = glm::normalize(color_);
}
} // namespace strahl::cpu_raymarcher
