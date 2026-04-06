#pragma once
#include <glm/gtc/quaternion.hpp>
#include <memory>
#include <optional>
#include <utility>
#include <vector>

#include "geometry.hpp"
#include "material.hpp"
#include "nodes.hpp"
#include "path.hpp"

namespace strahl::cpu {
class Scene : SceneNode {
 public:
  Scene() {}
  template <typename... Args>
  Camera* addCamera(Args&&... args) {
    auto cam = std::make_unique<Camera>(std::forward<Args>(args)...);
    auto ret = cam.get();
    nodes_.push_back(std::move(cam));
    return ret;
  }
  Body* addBody(Geometry* g, Material* m);

 private:
  std::vector<std::unique_ptr<SceneNode>> nodes_;
  std::vector<Light> lights_;
};
}  // namespace strahl::cpu