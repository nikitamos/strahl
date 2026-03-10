#pragma once

#include "material.hpp"
#include <glm/glm.hpp>
namespace strahl {

template <typename TNode> struct NodeAlgebra {
  virtual TNode *CreateSphere() = 0;
  virtual TNode *CreatePlane() = 0;
  virtual TNode *CreateComposite() = 0;
};

struct RenderNode {
//   virtual glm::vec3 GetPosition() = 0;
  virtual Material &GetMaterial() = 0;
  virtual const Material &GetMaterial() const = 0;
};

template <typename Backend, typename Node> struct RenderBackendAlgebra {};

} // namespace strahl