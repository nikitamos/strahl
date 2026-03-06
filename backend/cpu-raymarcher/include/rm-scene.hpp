#pragma once
#include "backend.hpp"
#include <algorithm>
#include <cassert>
#include <glm/fwd.hpp>
#include <glm/geometric.hpp>
#include <glm/glm.hpp>

#include <initializer_list>
#include <iterator>
#include <limits>
#include <memory>
#include <ranges>
#include <utility>
#include <vector>

namespace strahl::cpu_raymarcher {

class Node {
protected:
  Node(bool is_collidable [[deprecated]] = false,
       Material mat = Material::kEmpty)
      : collidable_(is_collidable) {}

public:
  bool IsCollidable() { return collidable_; }
  virtual float Distance(glm::vec3 point);
  virtual Node *ClosestNode(glm::vec3 point) { return this; }
  virtual glm::vec3 GetNormal(glm::vec3 point) { return {}; }
  const Material &GetMaterial() const { return material; }

private:
  [[deprecated]]
  const bool collidable_;
  Material material;
};

class CompositionNode : public Node {
public:
  CompositionNode();
  CompositionNode(std::initializer_list<Node *> &&children);
  float Distance(glm::vec3 point) override {
    return std::ranges::min_element(
               nodes_, {},
               [point](std::unique_ptr<Node> &x) { return x->Distance(point); })
        ->get()
        ->Distance(point);
  }

  Node *ClosestNode(glm::vec3 point) override {
    return std::ranges::min_element(
               nodes_, {},
               [point](std::unique_ptr<Node> &x) { return x->Distance(point); })
        ->get();
  }

private:
  std::vector<std::unique_ptr<Node>> nodes_;
};

class Plane : public Node {
public:
  Plane(glm::vec3 origin, glm::vec3 normal, Material m = Material::kEmpty)
      : origin_(origin), normal_(normal), Node(true, m) {
    normal_ = glm::normalize(normal_);
    abcd_ = glm::vec4(normal_, -glm::dot(origin, normal_));
    denom_ = glm::length(normal_);
    assert(denom_ != 0 && "normal vector seems to be zero");
  }
  float Distance(glm::vec3 point) override;
  glm::vec3 GetNormal(glm::vec3 point) override;
  virtual ~Plane();

private:
  glm::vec3 origin_;
  glm::vec3 normal_;
  glm::vec4 abcd_;
  float denom_;
};

class Sphere : public Node {
public:
  Sphere(glm::vec3 center, float r, Material m = Material::kEmpty)
      : center_(center), r_(r), Node(true, m) {}
  float Distance(glm::vec3 point) override;
  glm::vec3 GetNormal(glm::vec3 point) override;
  virtual ~Sphere();

private:
  glm::vec3 center_;
  float r_;
};

class PointLight : public Sphere {
  virtual ~PointLight();
};

} // namespace strahl::cpu_raymarcher
