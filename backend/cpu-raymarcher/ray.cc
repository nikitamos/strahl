#include "include/ray.hpp"

#include <cmath>
#include <glm/geometric.hpp>
#include <random>

#include "material.hpp"

namespace strahl::cpu_raymarcher {
void Ray::Advance(float distance) {
  assert(bounces_ < opts_.bounces);
  cur_pos_ += direction_ * distance;
}
void Ray::Intersect(Material m, glm::vec3 normal) {
  assert(bounces_ < opts_.bounces);
  auto new_dir = Reflect(normal, m);
  MixColor(m, new_dir, -direction_, normal);
  direction_ = new_dir;
  cur_pos_ += 2.0f * 0.01f * direction_;

  // For now let consider all materials absolutely reflective
  bounces_++;
}
glm::vec3 Ray::Reflect(glm::vec3 normal, const Material &m) const {
  thread_local static std::mt19937 engine(std::random_device{}());  // Seed with random_device
  thread_local static std::uniform_real_distribution<float> dist(-1.0f, 1.0f);

  auto h = glm::normalize(glm::cross(direction_, normal));
  auto f = glm::cross(normal, h);
  normal += m.diffuse * (dist(engine) * h + dist(engine) * f);
  return glm::reflect(direction_, normal);
}
void Ray::MixColor(Material m, glm::vec3 light_dir, glm::vec3 eye, glm::vec3 normal) {
  float diffuse = m.diffuse * glm::dot(light_dir, normal);
  float specular = m.specular * glm::dot(-direction_, eye);
  color_ += multiple_ * diffuse * m.color;
  multiple_ *= specular;
}
}  // namespace strahl::cpu_raymarcher
