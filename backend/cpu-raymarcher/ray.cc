#include "include/ray.hpp"
#include "material.hpp"
#include <cmath>
#include <glm/geometric.hpp>

namespace strahl::cpu_raymarcher {
void Ray::Advance(float distance) {
  assert(bounces_ < opts_.bounces);
  cur_pos_ += direction_ * distance;
}
void Ray::Intersect(Material m, glm::vec3 normal) {
  assert(bounces_ < opts_.bounces);
  MixColor(m, glm::reflect(direction_, normal), -direction_, normal);
  auto old_dir = direction_;
  // For now let consider all materials absolutely reflective
  Reflect(normal);
  bounces_++;
}
void Ray::Reflect(glm::vec3 normal) {
  direction_ -= 2 * glm::dot(direction_, normal) * normal;
  cur_pos_ += 2 * 0.01f * direction_;
}
void Ray::MixColor(Material m, glm::vec3 light_dir, glm::vec3 eye,
                   glm::vec3 normal) {
  float diffuse = m.diffuse * glm::dot(light_dir, normal);
  float specular = m.specular * glm::dot(-direction_, eye);
  color_ += multiple_ * diffuse * m.color;
  multiple_ *= specular;
}
} // namespace strahl::cpu_raymarcher
