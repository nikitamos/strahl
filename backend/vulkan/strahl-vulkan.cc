#include "include/strahl-vulkan.hpp"

#include <algorithm>
#include <cstdint>
#include <iostream>
#include <limits>
#include <memory>
#include <stdexcept>
#include <vulkan/vulkan.hpp>
#include <vulkan/vulkan_to_string.hpp>

#include "device-queue.hpp"
#include "private/alloc.hpp"
#include "private/gpu-vector.hpp"
#include "vk-renderer.hpp"

namespace strahl::vulkan {
VulkanBackend::VulkanBackend() : owns_instance_(true) {
  dqi_ = std::make_unique<detail::DeviceQueueInfo>();
  vk::ApplicationInfo app_info{
    .pApplicationName = "strahl",
    .applicationVersion = VK_MAKE_VERSION(1, 0, 0),
    .apiVersion = vk::ApiVersion12,
  };
  vk::InstanceCreateInfo ici{
    .pApplicationInfo = &app_info,
  };
  instance_ = vk::createInstance(ici);
  // Find physical device
  findDeviceQueue();
  vec_ = std::make_unique<detail::GpuVector>(dqi_->dev, dqi_->tx, alloc_.get(), dqi_->families);
  renderer_ = std::make_unique<VulkanRenderer>(dqi_.get());
}
VulkanBackend::~VulkanBackend() {
  vec_ = nullptr;
  renderer_ = nullptr;
  dqi_ = nullptr;
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
        device_com_queues[d] = (int)q;
      }
      if (device_tx_queues[d] < 0 && (queues[q].queueFlags & vk::QueueFlagBits::eTransfer)) {
        device_tx_queues[d] = (int)q;
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
  auto target_dev = std::max_element(scores.begin(), scores.end()) - scores.begin();
  if (scores[target_dev] < 0) {
    throw std::runtime_error("Failed to find a suitable device.");
  }
  int tx_family = device_tx_queues[target_dev];
  int com_family = device_com_queues[target_dev];
  uint32_t com_index = 0;
  uint32_t tx_index = 0;

  std::vector<vk::DeviceQueueCreateInfo> qci;
  static float priorities[] = {1.0, 1.0};
  if (tx_family == com_family) {
    uint32_t queue_count =
      std::min(2U, devs[target_dev].getQueueFamilyProperties()[com_family].queueCount);
    qci = {
      {.queueFamilyIndex = (uint32_t)com_family,
       .queueCount = queue_count,
       .pQueuePriorities = priorities}};
    tx_index = queue_count - 1;
  } else {
    qci = {
      {.queueFamilyIndex = (uint32_t)com_family, .queueCount = 1, .pQueuePriorities = priorities},
      {.queueFamilyIndex = (uint32_t)tx_family,
       .queueCount = 1,
       .pQueuePriorities = priorities + 1}};
  }
  dqi_->tx_family = tx_family;
  dqi_->com_family = com_family;
  auto phy_dev = devs[target_dev];
  static const char* const kDevExtensions[] = {"VK_GOOGLE_user_type"};
  dqi_->dev = phy_dev.createDevice(
    vk::DeviceCreateInfo{
      .queueCreateInfoCount = (uint32_t)qci.size(),
      .pQueueCreateInfos = qci.data(),
      .enabledExtensionCount = 1,
      .ppEnabledExtensionNames = kDevExtensions});
  dqi_->tx = dqi_->dev.getQueue(tx_family, tx_index);
  dqi_->com = dqi_->dev.getQueue(com_family, com_index);

  alloc_ = std::make_unique<detail::Allocator>(phy_dev, dqi_->dev);
}
VulkanScene* VulkanBackend::createScene() { return nullptr; }

VulkanBackend::VulkanBackend(VkInstance inst) : instance_(inst) {}
}  // namespace strahl::vulkan
