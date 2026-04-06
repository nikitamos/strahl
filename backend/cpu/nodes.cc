#include "include/nodes.hpp"

#include "detail/ray.hpp"
#include "path.hpp"

namespace strahl::cpu {
std::span<detail::Ray> Camera::initRays() {
  if (!rays_.empty()) {
    return rays_;
  }
  rays_.reserve(resolution_.x * resolution_.y);

  // TODO(m0sni): move raygen logic into a policy-like class
  glm::vec3 cam_pos = translation_;
  glm::vec3 screen_center = cam_pos + dir_;
  glm::vec3 screen_up = glm::normalize(glm::cross(right_, dir_)) * glm::length(right_) *
                        ((float)resolution_.y / (float)resolution_.x);

  auto top_left = screen_center + screen_up - right_;
  auto x_step = 2.0f / (float)resolution_.x * right_;
  auto y_step = -2.0f / (float)resolution_.y * screen_up;

  // Create rays
  for (int j = 0; j < resolution_.y; ++j) {
    for (int i = 0; i < resolution_.x; ++i) {
      auto point = top_left + (float)i * x_step + (float)j * y_step;
      auto ray_direction = glm::normalize(point - cam_pos);
      rays_.emplace_back(point, ray_direction);
    }
  }
  // <--- Raygen logic ends here

  return rays_;
}
std::optional<RayHit> Body::intersect(const detail::Ray &r) {
  auto origin = world2local(r.origin);
  auto local_ray = detail::Ray{origin, r.direction};
  return geometry_->intersect(local_ray).transform(
    [this](glm::vec3 isect) { return RayHit{&material_, geom2local(isect)}; });
}
}  // namespace strahl::cpu
