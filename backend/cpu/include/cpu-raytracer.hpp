#pragma once
#include <common.hpp>
#include <glm/glm.hpp>
#include <memory>

namespace strahl::cpu {
class Texture;
class Material;
class Sphere;
class Plane;
class Mesh;
class Scene;
class Solver;

class ResourceManager {
 public:
//   ResourceManager(const ResourceManager&) = default;
//   ResourceManager(ResourceManager&&) = default;
  ResourceManager& operator=(const ResourceManager&) = delete;
  ResourceManager& operator=(ResourceManager&&) = delete;

  Texture* createTexture(const TextureCreateInfo& ci);
  Material* createMaterial(const MaterialCreateInfo& ci);

  Sphere* createSphere(const TextureCreateInfo& ci);
  Plane* createPlane(glm::vec3 normal);
  Mesh* createMesh(const MeshCreateInfo& mci);
};
class RayTracer {
 public:
  Scene* createScene();
  Solver* createSolver();
  ResourceManager& getResourceManager() { return resource_mgr_; }
  static std::unique_ptr<RayTracer> create();

 private:
  RayTracer() {}
  ResourceManager resource_mgr_;
};
}  // namespace strahl::cpu