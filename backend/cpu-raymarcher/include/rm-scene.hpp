#pragma once
#include "backend.hpp"
#include <algorithm>
#include <cassert>
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
  Node(bool is_collidable [[deprecated]] = false)
      : collidable_(is_collidable) {}

public:
  bool IsCollidable() { return collidable_; }
  virtual float Distance(glm::vec3 point);
  virtual Node *ClosestNode(glm::vec3 point) { return this; }

private:
  [[deprecated]]
  const bool collidable_;
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
  Plane(glm::vec3 origin, glm::vec3 normal)
      : origin_(origin), normal_(normal), Node(true) {
    abcd_ = glm::vec4(normal, -glm::dot(origin, normal));
    denom_ = glm::length(normal);
    assert(denom_ != 0 && "normal vector seems to be zero");
  }
  float Distance(glm::vec3 point) override;
  virtual ~Plane();

private:
  glm::vec3 origin_;
  glm::vec3 normal_;
  glm::vec4 abcd_;
  float denom_;
};

class Sphere : public Node {
public:
  Sphere(glm::vec3 center, float r) : center_(center), r_(r), Node(true) {}
  float Distance(glm::vec3 point) override;
  virtual ~Sphere();

private:
  glm::vec3 center_;
  float r_;
};

class PointLight : public Sphere {
  virtual ~PointLight();
};

} // namespace strahl::cpu_raymarcher
