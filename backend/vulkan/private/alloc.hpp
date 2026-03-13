#pragma once
#include <cstdint>
#include <optional>
#include <vulkan/vulkan.hpp>
#include <vulkan/vulkan_to_string.hpp>

namespace strahl::vulkan::detail {
class Allocator {
 private:
  /// Finds the first suitable memory type and returns its index. If no such type is found, returns
  /// UINT32_MAX
  uint32_t findMemType(vk::MemoryPropertyFlags properties, uint32_t allowed_mem_types) const;

 public:
  static constexpr const auto kStagingFlags =
    vk::MemoryPropertyFlagBits::eHostVisible | vk::MemoryPropertyFlagBits::eHostCached;
  static constexpr const auto kDeviceFlags = vk::MemoryPropertyFlagBits::eDeviceLocal;
  Allocator() {}
  Allocator(vk::PhysicalDevice phy, vk::Device dev);

  std::optional<vk::DeviceMemory> allocate(
    vk::DeviceSize size,
    vk::MemoryPropertyFlags flags,
    uint32_t supported_mem_types,
    void *next = nullptr) const;

  std::optional<vk::DeviceMemory> allocStagingMem(
    vk::DeviceSize size, uint32_t supported_mem_types = 0xFFFFFFFF, void *next = nullptr) const;

  std::optional<vk::DeviceMemory> allocDeviceMem(
    vk::DeviceSize size, uint32_t supported_mem_types, void *next = nullptr) const {
    return allocate(size, kDeviceFlags, supported_mem_types);
  }

  void dealloc(vk::DeviceMemory mem, void *next = nullptr) const;
  vk::DeviceMemory realloc(
    vk::DeviceMemory old, vk::DeviceSize new_size, void *next = nullptr) const;

 private:
  vk::PhysicalDeviceMemoryProperties props_;
  vk::Device dev_;
};

}  // namespace strahl::vulkan::detail
