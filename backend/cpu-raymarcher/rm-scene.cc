#include "rm-scene.hpp"

namespace strahl::cpu_raymarcher {
CompositionNode::CompositionNode() {
  
}
CompositionNode::CompositionNode(std::initializer_list<Node *> &&children) {
  for (auto n : children) {
    nodes_.push_back(std::unique_ptr<Node>(n));
  }
}
float Node::Distance(glm::vec3 point) {
  return std::numeric_limits<float>::infinity();
}
float Plane::Distance(glm::vec3 point) {
  return glm::dot(glm::vec4(point, 1.0), abcd_) / denom_;
}
float Sphere::Distance(glm::vec3 point) {
  return std::abs(glm::distance(center_, point) - r_);
}
Sphere::~Sphere() {}
PointLight::~PointLight() {}
Plane::~Plane() {}
glm::vec3 Plane::GetNormal(glm::vec3 point) { return normal_; }
glm::vec3 Sphere::GetNormal(glm::vec3 point) {
  return glm::normalize(point - center_);
}
} // namespace strahl::cpu_raymarcher