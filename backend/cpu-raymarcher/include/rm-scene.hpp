#pragma once
#include <algorithm>
#include <cassert>
#include <glm/fwd.hpp>
#include <glm/geometric.hpp>
#include <glm/glm.hpp>

#include <initializer_list>
#include <memory>
#include <vector>

#include "factory.hpp"

namespace strahl::cpu_raymarcher {

class Node : public RenderNode {
protected:
  Node(bool is_terminal = false, Material mat = Material::kEmpty)
      : is_terminal(is_terminal), material_(mat) {}

public:
  bool IsCollidable() { return is_terminal; }
  virtual float Distance(glm::vec3 point);
  virtual Node *ClosestNode(glm::vec3 point) { return this; }
  virtual glm::vec3 GetNormal(glm::vec3 point) { return {}; }
  const Material &GetMaterial() const override { return material_; }
  Material &GetMaterial() override { return material_; }

protected:
  void SetTerminal(bool t) { is_terminal = t; }

private:
  bool is_terminal;
  Material material_;
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
      : origin_(origin), normal_(normal), Node(false, m) {
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
      : center_(center), r_(r), Node(false, m) {}
  float Distance(glm::vec3 point) override;
  glm::vec3 GetNormal(glm::vec3 point) override;
  virtual ~Sphere();

private:
  glm::vec3 center_;
  float r_;
};

class PointLight : public Sphere {
public:
  PointLight(glm::vec3 pos, glm::vec3 color, float r = 0.2f) : Sphere(pos, r) {
    SetTerminal(true);
  }
  virtual ~PointLight();
};

} // namespace strahl::cpu_raymarcher
