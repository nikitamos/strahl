#include "include/raymarcher.hpp"

namespace strahl::cpu_raymarcher {

CpuRaymarcherBackendOptions &CpuRaymarcherBackend::GetOptions() {
  return opts_;
}
void CpuRaymarcherBackend::SetScene(Scene *s) {}
Response CpuRaymarcherBackend::Render() {}
} // namespace strahl::cpu_raymarcher