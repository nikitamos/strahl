#include "alloc.hpp"

#include <cstdint>
#include <ios>
#include <iostream>
#include <limits>
#include <vulkan/vulkan.hpp>

namespace strahl::vulkan::detail {
std::optional<vk::DeviceMemory> Allocator::allocStagingMem(
  vk::DeviceSize size, uint32_t supported_mem_types, void *next) const {
  return allocate(size, kStagingFlags, supported_mem_types, next);
}
std::optional<vk::DeviceMemory> Allocator::allocate(
  vk::DeviceSize size,
  vk::MemoryPropertyFlags flags,
  uint32_t allowed_mem_types,
  void *next) const {
  std::cout << "requested allocation with flags: " << vk::to_string(flags)
            << ", supported types: 0x" << std::hex << allowed_mem_types << " :: ";
  uint32_t mem_type = findMemType(flags, allowed_mem_types);
  if (mem_type == std::numeric_limits<uint32_t>::max()) {
    std::cout << " no suitable memory type" << std::endl;
    return std::nullopt;
  }
  std::cout << "using mem type " << mem_type << std::endl;
  return dev_.allocateMemory(
    vk::MemoryAllocateInfo{.pNext = next, .allocationSize = size, .memoryTypeIndex = mem_type});
}

strahl::vulkan::detail::Allocator::Allocator(vk::PhysicalDevice phy, vk::Device dev) : dev_(dev) {
  props_ = phy.getMemoryProperties();
  std::cout << "allocator: available memory types: \n";
  for (size_t i = 0; i < props_.memoryTypeCount; ++i) {
    std::cout << "type " << i << ": heap=" << props_.memoryTypes[i].heapIndex
              << " flags=" << vk::to_string(props_.memoryTypes[i].propertyFlags) << '\n';
  }
  for (size_t i = 0; i < props_.memoryHeapCount; ++i) {
    std::cout << "heap " << i << ": size=" << props_.memoryHeaps[i].size
              << " flags=" << vk::to_string(props_.memoryHeaps[i].flags) << '\n';
  }
}
uint32_t Allocator::findMemType(
  vk::MemoryPropertyFlags properties, uint32_t allowed_mem_types) const {
  for (size_t i = 0; allowed_mem_types != 0; ++i, allowed_mem_types >>= 1) {
    if (
      (allowed_mem_types & 0x01) != 0 &&
      ((properties & props_.memoryTypes[i].propertyFlags) == properties)) {
      return i;
    }
  }
  return std::numeric_limits<uint32_t>::max();
}
}  // namespace strahl::vulkan::detail