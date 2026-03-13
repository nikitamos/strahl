#pragma once

#include <vulkan/vulkan.hpp>

namespace strahl::vulkan {
class VulkanBackend;

class Sphere {};
/// Scene is responsible for managing GPU memory and storing primitives.
/// It doesn't present itself.
class VulkanScene {
 public:
  Sphere* addSphere();

 protected:
  friend VulkanBackend;
  VulkanScene(vk::Queue tx, vk::Queue com) : transfer_(tx), compute_(com) {}

 private:
  vk::Queue transfer_, compute_;
};

}  // namespace strahl::vulkan
