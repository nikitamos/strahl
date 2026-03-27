#pragma once

#include <vulkan/vulkan.hpp>

#include "vulkan/vulkan.hpp"

namespace strahl::vulkan {
class VulkanRenderer;
namespace detail {
struct DeviceQueueInfo;
class GpuVector;
}
/// Renderer is responsible for managing piplines, queues and other related resources (e.g. texture)
class VulkanRenderer final {
 public:
  VulkanRenderer() {}
  explicit VulkanRenderer(detail::DeviceQueueInfo* dqi, detail::GpuVector* vec);
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
  vk::Semaphore tx2com_;
  vk::Semaphore com_end_;
  uint64_t count_ = 0;

  vk::CommandBuffer tx_buffer_;
  vk::CommandBuffer com_buffer_;
  detail::DeviceQueueInfo *dqi_ = nullptr;
  detail::GpuVector* vec_;
};

}  // namespace strahl::vulkan
