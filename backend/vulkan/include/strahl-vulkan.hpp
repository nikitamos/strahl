#pragma once
#include <vulkan/vulkan.h>

#include <array>
#include <vulkan/vulkan.hpp>

#include "vulkan/vulkan.hpp"

namespace strahl::vulkan {
class VulkanBackend final {
 public:
  VulkanBackend();
  VulkanBackend(VkInstance inst) : instance_(inst) {}
  ~VulkanBackend();

 private:
  void findDevice();
  bool owns_instance_ = false;
  vk::Instance instance_;
  vk::Device device_;

  vk::Queue transfer_;
  vk::Queue compute_;
};
}  // namespace strahl::vulkan
