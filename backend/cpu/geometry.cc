#include "geometry.hpp"

#include <glm/fwd.hpp>
#include <optional>

#include "detail/ray.hpp"

namespace strahl::cpu {
std::optional<glm::vec3> Plane::intersect(const detail::Ray& ray) {
  float denom = glm::dot(normal_, ray.direction);
  if (std::abs(denom) < 1e-6f) {
    return std::nullopt;
  }
  glm::vec3 diff = origin_ - ray.origin;
  float t = glm::dot(normal_, diff) / denom;
  if (t < 0) {
    return std::nullopt;
  }
  return ray.origin + t * ray.direction;
}
std::optional<glm::vec3> Sphere::intersect(const detail::Ray& ray) {
  // Vector from ray origin to sphere center (assuming sphere is at origin)
  // Note: This implementation assumes sphere is centered at (0,0,0)
  // If sphere has a center member, adjust accordingly
  // m0sni: Shouldn't it be negated?
  glm::vec3 oc = ray.origin;  // Since sphere is at origin

  float a = glm::dot(ray.direction, ray.direction);
  float b = 2.0f * glm::dot(oc, ray.direction);
  float c = glm::dot(oc, oc) - r * r;

  float discriminant = b * b - 4 * a * c;

  if (discriminant < 0) {
    return std::nullopt;
  }

  float sqrt_disc = std::sqrt(discriminant);
  float t1 = (-b - sqrt_disc) / (2 * a);
  float t2 = (-b + sqrt_disc) / (2 * a);

  float t = -1.0f;
  if (t1 > 0 && t2 > 0) {
    t = std::min(t1, t2);
  } else if (t1 > 0) {
    t = t1;
  } else if (t2 > 0) {
    t = t2;
  } else {
    return std::nullopt;
  }

  return ray.origin + t * ray.direction;
}
}  // namespace strahl::cpu