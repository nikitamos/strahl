#pragma once
#include <iostream>
#include <vulkan/vulkan.hpp>

namespace strahl::vulkan::detail {
struct DeviceQueueInfo {
  vk::Device dev;
  vk::Queue tx, com;
  union {
    struct {
      uint32_t tx_family;
      uint32_t com_family;
    };
    uint32_t families[2];
  };
  DeviceQueueInfo() {}
  DeviceQueueInfo(DeviceQueueInfo&& rhs) = delete;
  DeviceQueueInfo(const DeviceQueueInfo& rhs) = delete;
  ~DeviceQueueInfo() {
    std::cout << "destroying dqi" << std::endl;
    dev.destroy();
  }
};
class QueueRecorder {
 public:
  QueueRecorder(vk::CommandBuffer buf, uint32_t family) : buf_(buf), family_(family) {}
  uint32_t family() const { return family_; }
  vk::CommandBuffer* operator*() { return &buf_; }
  vk::CommandBuffer* operator->() { return &buf_; }

 private:
  vk::CommandBuffer buf_;
  uint32_t family_;
};
}  // namespace strahl::vulkan::detail
