#include "material.hpp"
#include <concepts>
#include <functional>

namespace strahl {
struct Node {
  virtual float Distance(glm::vec3 point) = 0;
  virtual float FindClosestChild();
  // virtual float VisitChildren()
  virtual void VisitChildren(std::function<void(Node *)> visitor) = 0;
  void* pimpl_; // + super-effective allocator,
                //   create via factories
};

class Object : Node {
  strahl::Material material;
};
} // namespace strahl
