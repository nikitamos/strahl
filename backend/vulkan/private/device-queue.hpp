#pragma once
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
};
}  // namespace strahl::vulkan::detail
