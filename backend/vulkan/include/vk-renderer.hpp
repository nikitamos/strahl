#pragma once

#include <vulkan/vulkan.hpp>

namespace strahl::vulkan {
namespace detail {
struct DeviceQueueInfo;
}
/// Renderer is responsible for managing piplines, queues and other related resources (e.g. texture)
class VulkanRenderer {
 public:
  VulkanRenderer() {}
  explicit VulkanRenderer(detail::DeviceQueueInfo *dqi);

  void doSomeRendering() {}

 private:
  vk::Pipeline main_;
  detail::DeviceQueueInfo *dqi_ = nullptr;
};

}  // namespace strahl::vulkan
