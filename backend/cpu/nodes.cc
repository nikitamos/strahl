#pragma once

#include "geometry.hpp"
#include "material.hpp"

namespace strahl::cpu {
class SceneNode {};
class Camera : public SceneNode {};
class Body : public SceneNode {
 public:
  Body(Geometry *geometry, Material *material) : geometry_(geometry), material_(material) {}

 private:
  Geometry *geometry_;
  Material *material_;
};
}  // namespace strahl::cpu
