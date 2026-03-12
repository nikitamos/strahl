#include "include/strahl-vulkan.hpp"

#include <algorithm>
#include <iostream>
#include <limits>
#include <memory>
#include <stdexcept>
#include <vulkan/vulkan.hpp>
#include <vulkan/vulkan_to_string.hpp>

#include "private/alloc.hpp"

namespace strahl::vulkan {
VulkanBackend::VulkanBackend() : owns_instance_(true) {
  vk::ApplicationInfo appInfo{
      .pApplicationName = "strahl",
      .applicationVersion = VK_MAKE_VERSION(1, 0, 0),
      .apiVersion = vk::ApiVersion12,
  };
  vk::InstanceCreateInfo ici{
      .pApplicationInfo = &appInfo,
  };
  instance_ = vk::createInstance(ici);
  // Find physical device
  findDeviceQueue();
  vec_ = std::make_unique<detail::GpuVector>(device_, transfer_, alloc_.get(), 0);
}
VulkanBackend::~VulkanBackend() {
  vec_ = nullptr;
  device_.destroy();
  if (owns_instance_) {
    instance_.destroy();
  }
  // Queue destruction is impossible
}
void VulkanBackend::findDeviceQueue() {
  auto devs = instance_.enumeratePhysicalDevices();
  std::vector<int> scores(devs.size());
  std::vector<int> device_com_queues(devs.size(), -1);
  std::vector<int> device_tx_queues(devs.size(), -1);
  for (size_t d = 0; d < devs.size(); ++d) {
    auto props = devs[d].getProperties();
    std::cout << "device #" << d << " " << props.deviceName << " ("
              << devs[d].getQueueFamilyProperties().size() << " families)" << std::endl;
    switch (props.deviceType) {
      case vk::PhysicalDeviceType::eOther:
        scores[d] -= 20;
        break;
      case vk::PhysicalDeviceType::eIntegratedGpu:
        scores[d] += 50;
        break;
      case vk::PhysicalDeviceType::eDiscreteGpu:
        scores[d] += 100;
        break;
      case vk::PhysicalDeviceType::eVirtualGpu:
        scores[d] += 80;
        break;
      case vk::PhysicalDeviceType::eCpu:
        scores[d] += 40;
        break;
    }
    auto queues = devs[d].getQueueFamilyProperties();
    for (size_t q = 0; q < queues.size(); ++q) {
      std::cout << "queue family " << q << ": count=" << queues[q].queueCount
                << " flags=" << vk::to_string(queues[q].queueFlags) << std::endl;
      if (device_com_queues[d] < 0 && (queues[q].queueFlags & vk::QueueFlagBits::eCompute)) {
        device_com_queues[d] = q;
      }
      if (device_tx_queues[d] < 0 && (queues[q].queueFlags & vk::QueueFlagBits::eTransfer)) {
        device_tx_queues[d] = q;
      }
    }
    // TODO: give some points if tx and com queues are distinct
    if (device_tx_queues[d] < 0 || device_com_queues[d] < 0) {
      scores[d] = std::numeric_limits<int>::min();
    }
    std::cout << "-> score:" << scores[d] << " tx:" << device_tx_queues[d]
              << " com:" << device_com_queues[d] << std::endl
              << std::endl;
  }
  auto target_dev =
      std::max_element(scores.begin(), scores.end()) - scores.begin();
  if (scores[target_dev] < 0) {
    throw std::runtime_error("Failed to find a suitable device.");
  }
  int tx_family = device_tx_queues[target_dev];
  int com_family = device_com_queues[target_dev];
  // TODO: probably override this logic to select distinct queues of the same
  // family if available
  int com_index = 0;
  int tx_index = 0;

  std::vector<vk::DeviceQueueCreateInfo> qci;
  static float priorities[] = {1.0, 1.0};
  if (tx_family == com_family) {
    qci = {{.queueFamilyIndex = (uint32_t)com_family,
            .queueCount = 2,
            .pQueuePriorities = priorities}};
  } else {
    qci = {{.queueFamilyIndex = (uint32_t)com_family,
            .queueCount = 1,
            .pQueuePriorities = priorities},
           {.queueFamilyIndex = (uint32_t)tx_family,
            .queueCount = 1,
            .pQueuePriorities = priorities + 1}};
  }
  auto phy_dev = devs[target_dev];
  device_ = phy_dev.createDevice(vk::DeviceCreateInfo{
      .queueCreateInfoCount = (uint32_t)qci.size(),
      .pQueueCreateInfos = qci.data()
      /* no layers or extensions*/});
  transfer_ = device_.getQueue(tx_family, tx_index);
  compute_ = device_.getQueue(com_family, com_index);

  alloc_ = std::make_unique<detail::Allocator>(phy_dev, device_);
}
VulkanScene* VulkanBackend::createScene() { return nullptr; }
VulkanBackend::VulkanBackend(VkInstance inst) : instance_(inst) {}
}  // namespace strahl::vulkan