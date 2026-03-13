#pragma once
#include <memory>
#include <vulkan/vulkan.hpp>

#include "vk-renderer.hpp"
#include "vk-scene.hpp"

namespace strahl::vulkan {
namespace detail {
class GpuVector;
class Allocator;
class DeviceQueueInfo;
}  // namespace detail

/// `VulkanBackend` is an entry point to the API.
class VulkanBackend final {
 public:
  VulkanBackend();
  explicit VulkanBackend(VkInstance inst);
  VulkanScene* createScene();
  ~VulkanBackend();

 private:
  void findDeviceQueue();
  vk::CommandPool createCommandPool(uint32_t queue_family);
  // Note: the members are destroyed in the reverse declaration order
  std::unique_ptr<detail::Allocator> alloc_;
  std::unique_ptr<detail::GpuVector> vec_;
  bool owns_instance_ = false;
  vk::Instance instance_;

  vk::CommandPool tx_pool_;
  vk::CommandPool com_pool_;
  std::unique_ptr<detail::DeviceQueueInfo> dqi_;

  VulkanRenderer renderer_;
};
}  // namespace strahl::vulkan
