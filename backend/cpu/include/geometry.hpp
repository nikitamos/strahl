#pragma once
namespace strahl::cpu {
class Geometry {};
class Plane : public Geometry {};
class Sphere : public Geometry {};
class Mesh : public Geometry {
  class UVMap {};
};
}  // namespace strahl::cpu
