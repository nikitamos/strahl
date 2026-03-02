#pragma once
#include "backend.hpp"

namespace strahl::cpu_raymarcher {
struct CpuRaymarcherBackendOptions : BackendOptions {
    float epsilon = 1E-4;
};

class CpuRaymarcherBackend : Backend {
public:
  Response Render() override;
  CpuRaymarcherBackendOptions &GetOptions() override;
  void SetScene(Scene *s) override;

private:
  Scene* cur_scene_ = nullptr;
  CpuRaymarcherBackendOptions opts_ {};
};
} // namespace strahl::cpu_raymarcher
