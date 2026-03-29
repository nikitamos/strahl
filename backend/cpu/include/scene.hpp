#pragma once
#include "geometry.hpp"
#include "material.hpp"
#include "nodes.hpp"

namespace strahl::cpu {
class Scene {
 public:
  Camera* addCamera();
  Body* addBody(Geometry* g, Material* m);
};
}
