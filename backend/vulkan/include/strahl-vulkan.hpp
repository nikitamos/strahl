#pragma once
#include <memory>
#include <vulkan/vulkan.hpp>

#include "vk-renderer.hpp"
#include "vk-scene.hpp"

namespace strahl::vulkan {
namespace detail {
class GpuVector;
class Allocator;
}  // namespace detail

/// `VulkanBackend` is an entry point to the API.
class VulkanBackend final {
 public:
  VulkanBackend();
  explicit VulkanBackend(VkInstance inst);
  VulkanScene* createScene();
  ~VulkanBackend();

 private:
  // Note: the members are destroyed in the reverse declaration order
  std::unique_ptr<detail::Allocator> alloc_;
  std::unique_ptr<detail::GpuVector> vec_;
  void findDeviceQueue();
  bool owns_instance_ = false;
  vk::Instance instance_;
  vk::Device device_;

  vk::Queue transfer_;
  vk::Queue compute_;

  VulkanRenderer renderer_;
};
}  // namespace strahl::vulkan
