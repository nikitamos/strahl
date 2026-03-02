#include "backend.hpp"
#include <glm/ext/vector_float3.hpp>
namespace strahl {
// Omnidirectional light
struct LightSource : Node {
  glm::vec3 color;
};
} // namespace strahl