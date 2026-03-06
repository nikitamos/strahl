#include <fstream>
#include <glm/geometric.hpp>
#include <glm/glm.hpp>
#include <vector>

#include "include/ray.hpp"
#include "include/raymarcher.hpp"
#include "include/rm-scene.hpp"

namespace strahl::cpu_raymarcher {

CpuRaymarcherBackendOptions &CpuRaymarcherBackend::GetOptions() {
  return opts_;
}
void CpuRaymarcherBackend::SetScene(Scene *s) {}
Response CpuRaymarcherBackend::Render() {
  glm::vec3 cam_pos = opts_.camera;
  glm::vec3 screen_center = cam_pos + opts_.screen;
  glm::vec3 screen_up =
      glm::normalize(glm::cross(opts_.screen_right, opts_.screen)) *
      ((float)opts_.resolution.y / opts_.resolution.x);

  auto viewport = opts_.resolution;
  auto top_left = screen_center + screen_up - opts_.screen_right;
  auto x_step = 2.0f / viewport.x * opts_.screen_right;
  auto y_step = 2.0f / viewport.y * screen_up;

  // Create rays
  // std::ofstream f("rays.csv");
  std::vector<Ray> rays;
  rays.reserve(viewport.x * viewport.y);
  for (int i = 0; i < opts_.resolution.x; ++i) {
    for (int j = 0; j < opts_.resolution.y; ++j) {
      auto point = top_left + (float)i * x_step + (float)j * y_step;
      auto ray_direction = glm::normalize(point - cam_pos);
      rays.emplace_back(opts_.camera, ray_direction, opts_);
      // f << ray_direction.x << "," << ray_direction.y << "," <<
      // ray_direction.z
      //   << '\n';
    }
  }

  for (auto &r : rays) {
    auto pos = r.GetPos();
    Node *victim = root_->ClosestNode(pos);
    auto distance = victim->Distance(pos);
    if (distance < 1.0E-5) { // collision
      // Handle collision
    } else {
      r.Advance(distance);
    }
  }

  return {};
}

CpuRaymarcherBackend::~CpuRaymarcherBackend() {}
} // namespace strahl::cpu_raymarcher