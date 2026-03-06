#pragma once
#include "backend.hpp"
#include "rm-scene.hpp"
#include <glm/glm.hpp>

namespace strahl::cpu_raymarcher {
struct CpuRaymarcherBackendOptions : BackendOptions {
    float epsilon = 1E-4;
    glm::vec3 camera = {0, -2, 0};
    glm::vec3 screen = {0, 2, 0};
    glm::vec3 screen_right = {0, 0, 3};
};

class CpuRaymarcherBackend : Backend {
public:
  [[nodiscard]]
  Response Render() override;
  [[nodiscard]]
  CpuRaymarcherBackendOptions &GetOptions() override;
  void SetScene(Scene *s) override;
  void SetRoot(Node *n) { root_ = n; }
  virtual ~CpuRaymarcherBackend();

private:
  Scene* cur_scene_ = nullptr;
  Node *root_ = nullptr;
  CpuRaymarcherBackendOptions opts_ {};
};
} // namespace strahl::cpu_raymarcher
