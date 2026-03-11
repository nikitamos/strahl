#pragma once
#include <concepts>
#include <glm/glm.hpp>
#include <memory>
#include <span>

#include "scene/scene.hpp"

namespace strahl {
namespace erased {
struct IScene {
  virtual void *AddSphere(glm::vec3 c, float r) = 0;
};

/// Internally manages all required resources.
/// Scenes and etc may contain a weak reference to the backend
struct Backend {
  virtual IScene *CreateScene() = 0;
};
}  // namespace erased

struct BackendOptions {
  glm::vec<2, int> resolution;
  int bounces = 2;
};

template <typename TBackend, typename TScene>
concept rt_backend = requires(TBackend b) {
  { b.DoSomething() } -> std::same_as<TScene>;
};

struct Response {
  const std::vector<glm::vec3> image;
  const glm::vec<2, int> resolution;
  // format -> R8G8B8 by default
};

struct Backend {
  virtual void SetScene(Scene *s) = 0;
  virtual Response Render() = 0;
  [[nodiscard]]
  virtual BackendOptions &GetOptions() = 0;
  virtual ~Backend() {}
};

struct AbstractBackend {};

template <typename TScene, rt_backend<TScene> TBackend>
class BackendWrapper : public AbstractBackend {
 public:
  BackendWrapper(TBackend &&backend)
      : back_(std::make_unique(std::move(backend))) {}

 private:
  std::unique_ptr<TBackend> back_;
};

}  // namespace strahl
