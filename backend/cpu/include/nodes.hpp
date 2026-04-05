#pragma once

#include <glm/fwd.hpp>
#include <span>

#include "detail/ray.hpp"
#include "geometry.hpp"
#include "material.hpp"

namespace strahl::cpu {
class SceneNode {
 public:
  explicit SceneNode(glm::vec3 translation = {} /*, glm::quat rotation = {}*/)
    : translation_(translation)
  //, rotation_(rotation)
  {}

  glm::vec3 translation() const { return translation_; }
  void setTranslation(glm::vec3 &&t) { translation_ = t; }
  // glm::quat rotation() const { return rotation_; }
  // void setRotation(glm::quat &&r) { rotation_ = r; }

  glm::vec3 local2world(glm::vec3 local) {
    // FIXME: rotation
    return local + translation_;
  }
  glm::vec3 world2local(glm::vec3 world) {
    // FIXME: rotation
    return world - translation_;
  }

 protected:
  glm::vec3 translation_;
  // glm::quat rotation_;
};
class Camera : public SceneNode {
 public:
  enum class Type { ePerspective, eOrthographic };
  explicit Camera(
    glm::vec<2, size_t> resolution,
    glm::vec3 direction,
    glm::vec3 right,
    glm::vec3 translation = {},
    Type t = Camera::Type::ePerspective)
    : SceneNode(translation),
      resolution_(resolution),
      cam_type_(t),
      dir_(direction),
      right_(right) {}
  void acquireImage(std::span<glm::vec3> image);

 protected:
  std::span<detail::Ray> initRays();

 private:
  glm::vec<2, size_t> resolution_;
  Type cam_type_;
  glm::vec3 right_;
  glm::vec3 dir_;
  // **INVARIANT**: if non-empty, contained rays are valid for the cam_type_ and resolution_
  std::vector<detail::Ray> rays_;
};
class Body : public SceneNode {
 public:
  Body(Geometry *geometry, const Material &material) : geometry_(geometry), material_(material) {}

 private:
  // C++26: switch to std::optional<Geometry&>?
  Geometry *geometry_;
  const Material &material_;
};
}  // namespace strahl::cpu
