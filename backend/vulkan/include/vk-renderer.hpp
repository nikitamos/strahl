#pragma once

#include <vulkan/vulkan.hpp>

namespace strahl::vulkan {

/// Renderer is responsible for managing piplines, queues and other related resources (e.g. texture)
class VulkanRenderer {
 public:
  VulkanRenderer(vk::Device dev, vk::Queue com, vk::Queue tx);
  VulkanRenderer() {}

 private:
  vk::Pipeline main_;
  vk::Queue compute_;
  vk::Queue transfer_;
};

}  // namespace strahl::vulkan
