#include <fstream>
#include <glm/fwd.hpp>
#include <glm/geometric.hpp>
#include <glm/glm.hpp>
#include <iomanip>
#include <iostream>
#include <vector>

#include "backend.hpp"
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
  std::ofstream f("rays.csv");
  std::vector<Ray> rays;
  rays.reserve(viewport.x * viewport.y);
  for (int i = 0; i < opts_.resolution.x; ++i) {
    for (int j = 0; j < opts_.resolution.y; ++j) {
      auto point = top_left + (float)i * x_step + (float)j * y_step;
      auto ray_direction = glm::normalize(point - cam_pos);
      rays.emplace_back(opts_.camera, ray_direction, opts_);
      f << ray_direction.x << "," << ray_direction.y << "," << ray_direction.z
        << '\n';
    }
  }

  const int kMaxIters = 500;
  int count = 0;
  for (auto &r : rays) {
    int iters = 0;
    for (int count = 0; iters < kMaxIters; ++iters) {
      if (r.GetBounces() >= opts_.bounces) {
        break;
      }
      auto pos = r.GetPos();
      Node *victim = root_->ClosestNode(pos);
      auto distance = victim->Distance(pos);
      if (distance < 1.0E-5) { // collision
        r.Intersect(victim->GetMaterial(), victim->GetNormal(pos));
      } else {
        r.Advance(distance);
      }
    }
    if (count % viewport.y == 0) {
      std::cout << std::endl;
    }

    std::cout << std::setw(2) << iters;
    count += 1;
  }
  std::cout << std::endl;

  std::vector<glm::vec3> image(rays.size());
  for (size_t i = 0; i < image.size(); ++i) {
    image[i] = rays[i].GetColor();
  }

  return {std::move(image), viewport};
}

CpuRaymarcherBackend::~CpuRaymarcherBackend() {}
} // namespace strahl::cpu_raymarcher