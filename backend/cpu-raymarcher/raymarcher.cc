#include "include/raymarcher.hpp"
#include "include/rm-scene.hpp"
#include <glm/geometric.hpp>
#include <glm/glm.hpp>

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

  int x_right = opts_.resolution.x - 1;
  int y_up = opts_.resolution.y - 1;
  for (int i = 0; i < opts_.resolution.x; ++i) {
    for (int j = 0; j < opts_.resolution.y; ++j) {
    }
  }

  return {};
}

CpuRaymarcherBackend::~CpuRaymarcherBackend() {}
} // namespace strahl::cpu_raymarcher