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
}  // namespace strahl::vulkan::detail
