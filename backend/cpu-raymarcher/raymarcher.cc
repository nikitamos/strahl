#include "include/raymarcher.hpp"

#include <algorithm>
#include <fstream>
#include <glm/common.hpp>
#include <glm/fwd.hpp>
#include <glm/geometric.hpp>
#include <glm/glm.hpp>
#include <vector>

#include "backend.hpp"
#include "include/ray.hpp"
#include "include/rm-scene.hpp"

namespace strahl::cpu_raymarcher {

CpuRaymarcherBackendOptions &CpuRaymarcherBackend::GetOptions() { return opts_; }
void CpuRaymarcherBackend::SetScene(Scene *s) {}
Response CpuRaymarcherBackend::Render() {
  glm::vec3 cam_pos = opts_.camera;
  glm::vec3 screen_center = cam_pos + opts_.screen;
  glm::vec3 screen_up = glm::normalize(glm::cross(opts_.screen_right, opts_.screen)) *
                        glm::length(opts_.screen_right) *
                        ((float)opts_.resolution.y / opts_.resolution.x);

  auto viewport = opts_.resolution;
  auto top_left = screen_center + screen_up - opts_.screen_right;
  auto x_step = 2.0f / viewport.x * opts_.screen_right;
  auto y_step = -2.0f / viewport.y * screen_up;

  // Create rays
  const float kMaxDist = 5.0;
  std::vector<Ray> rays;
  rays.reserve(viewport.x * viewport.y);
  for (int j = 0; j < viewport.y; ++j) {
    for (int i = 0; i < viewport.x; ++i) {
      auto point = top_left + (float)i * x_step + (float)j * y_step;
      auto ray_direction = glm::normalize(point - cam_pos);
      rays.emplace_back(opts_.camera, ray_direction, opts_);
    }
  }

  const int kMaxIters = 500;
#pragma omp parallel for
  for (auto &rt : rays) {
    glm::vec3 color{0, 0, 0};
    for (int s = 0; s < opts_.per_pixel_sample; ++s) {
      Ray r = rt;
      for (int iters = 0; iters < kMaxIters; ++iters) {
        if (r.GetBounces() >= opts_.bounces) {
          float s = 1.0;  // glm::clamp(glm::distance(r.GetPos(), cam_pos) / kMaxDist,
                          //      0.0f, 1.0f);
          break;
        }
        auto pos = r.GetPos();
        Node *victim = root_->ClosestNode(pos);
        auto distance = victim->Distance(pos);
        if (distance < 1.0E-5) {  // collision
          r.Intersect(victim->GetMaterial(), victim->GetNormal(pos));
        } else {
          r.Advance(distance);
        }
      }
      color += r.GetColor();
    }
    rt.SetColor(color);
    // TODO: Ambient material
    // if (iters >= kMaxIters) { // No light source hit
    // r.SetColor(glm::vec3{0.2, 0.3, 0.4} * 0.43f);
    // }
  }

  std::vector<glm::vec3> image(rays.size());
  for (size_t i = 0; i < image.size(); ++i) {
    image[i] = rays[i].GetColor();
  }

  return {std::move(image), viewport};
}

CpuRaymarcherBackend::~CpuRaymarcherBackend() {}
}  // namespace strahl::cpu_raymarcher