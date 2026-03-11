#include "include/strahl-vulkan.hpp"

#include <algorithm>
#include <iostream>
#include <limits>
#include <stdexcept>
#include <vulkan/vulkan.hpp>
#include <vulkan/vulkan_to_string.hpp>

#include "vulkan/vulkan.hpp"

namespace strahl::vulkan {
VulkanBackend::VulkanBackend() {
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
  findDevice();
}
VulkanBackend::~VulkanBackend() {
  if (owns_instance_) {
    instance_.destroy();
  }
  device_.destroy();
  // Queue destruction is impossible
}
void VulkanBackend::findDevice() {
  auto devs = instance_.enumeratePhysicalDevices();
  std::vector<int> scores(devs.size());
  std::vector<int> device_com_queues(devs.size(), -1);
  std::vector<int> device_tx_queues(devs.size(), -1);
  for (size_t i = 0; i < devs.size(); ++i) {
    auto props = devs[i].getProperties();
    std::cout << "device #" << i << "" << props.deviceName << " ("
              << devs[i].getQueueFamilyProperties().size() << " families)"
              << std::endl;
    switch (props.deviceType) {
      case vk::PhysicalDeviceType::eOther:
        scores[i] -= 20;
        break;
      case vk::PhysicalDeviceType::eIntegratedGpu:
        scores[i] += 50;
        break;
      case vk::PhysicalDeviceType::eDiscreteGpu:
        scores[i] += 100;
        break;
      case vk::PhysicalDeviceType::eVirtualGpu:
        scores[i] += 80;
        break;
      case vk::PhysicalDeviceType::eCpu:
        scores[i] += 40;
        break;
    }
    auto queues = devs[i].getQueueFamilyProperties();
    for (size_t q = 0; q < queues.size(); ++q) {
      std::cout << "queue family " << i << ": count=" << queues[i].queueCount
                << "flags=" << vk::to_string(queues[q].queueFlags) << std::endl;
      if (device_com_queues[i] < 0 &&
          (queues[q].queueFlags & vk::QueueFlagBits::eCompute)) {
        device_com_queues[i] = q;
      }
      if (device_tx_queues[i] < 0 &&
          (queues[q].queueFlags & vk::QueueFlagBits::eTransfer)) {
        device_tx_queues[i] = q;
      }
    }
    // TODO: give some points if tx and com queues are distinct
    if (device_tx_queues[i] < 0 || device_com_queues[i] < 0) {
      scores[i] = std::numeric_limits<int>::min();
    }
    std::cout << "-> score:" << scores[i] << " tx: " << device_tx_queues[i]
              << " com: " << device_com_queues[i] << std::endl
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
}
}  // namespace strahl::vulkan