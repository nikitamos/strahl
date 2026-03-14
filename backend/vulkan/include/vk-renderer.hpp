#pragma once

#include <vulkan/vulkan.hpp>

#include "vulkan/vulkan.hpp"

namespace strahl::vulkan {
class VulkanRenderer;
namespace detail {
struct DeviceQueueInfo;
}
/// Renderer is responsible for managing piplines, queues and other related resources (e.g. texture)
class VulkanRenderer final {
 public:
  VulkanRenderer() {}
  explicit VulkanRenderer(detail::DeviceQueueInfo *dqi);
  ~VulkanRenderer();
  VulkanRenderer(const VulkanRenderer& rhs) = delete;
  VulkanRenderer(VulkanRenderer&& rhs) = delete;
  VulkanRenderer& operator=(const VulkanRenderer& rhs) = delete;
  VulkanRenderer& operator=(VulkanRenderer&& rhs) = delete;

  void doSomeRendering();

 private:
  void createCommandPoolBufs();
  vk::Pipeline pipeline_;
  vk::Pipeline main_;
  vk::CommandPool tx_pool_;
  vk::CommandPool com_pool_;

  vk::CommandBuffer tx_buffer_;
  vk::CommandBuffer com_buffer_;
  detail::DeviceQueueInfo *dqi_ = nullptr;
};

}  // namespace strahl::vulkan
