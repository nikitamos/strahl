#include "include/ray.hpp"

namespace strahl::cpu_raymarcher {
void Ray::Advance(float distance) {
  assert(bounces_ < opts_.bounces);
  cur_pos_ += direction_ * distance;
}
void Ray::Intersect(Material m, glm::vec3 normal) {
  assert(bounces_ < opts_.bounces);
  SetColor(m.color);
  bounces_++;
}
} // namespace strahl::cpu_raymarcher
