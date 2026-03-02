#pragma once
#include "scene/scene.hpp"
#include <glm/glm.hpp>
#include <memory>
#include <span>

namespace strahl {

struct CpuRenderResponse {};
struct WgpuRenderResponse {};
struct VulkanRenderResponse {};

template <typename TIn, typename TOut> struct RaytracingPipelineStep {
  virtual TOut Process(TIn input) = 0;
};

struct BackendOptions {
  glm::vec<2, int> resolution;
  int bounces = 2;
};

// template<typename TResponse>
// struct RenderTarget {
//   virtual void
// };

struct Response {
  const glm::vec<2, int> resolution;
  const std::span<glm::vec<3, float>> GetBuffer();
  // format -> R8G8B8 by default
};

struct Backend {
  virtual void SetScene(Scene *s) = 0;
  virtual Response Render() = 0;
  virtual BackendOptions &GetOptions() = 0;
  virtual ~Backend() = 0;
};

} // namespace strahl