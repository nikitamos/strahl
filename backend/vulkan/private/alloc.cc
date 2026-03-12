#include <vulkan/vulkan.hpp>
#include <algorithm>
#include <iostream>

#include "alloc.hpp"

namespace strahl::vulkan::detail {
static bool areFlagsSupported(auto required, auto supported) {
  return static_cast<decltype(required)::MaskType>(required & supported) != 0;
}

std::optional<vk::DeviceMemory> Allocator::allocStagingMem(
  vk::DeviceSize size, vk::MemoryPropertyFlags supported_flags, void *next) const {
  if (areFlagsSupported(kStagingFlags, supported_flags)) {
    return allocate(size, kStagingFlags, next);
  }
  return std::nullopt;
}
std::optional<vk::DeviceMemory> Allocator::allocate(
  vk::DeviceSize size, vk::MemoryPropertyFlags flags, void *next) const {
  std::cout << "requested allocation with flags: " << vk::to_string(flags) << next << std::endl;
  const vk::MemoryType *mem_type = std::find_if(
    props_.memoryTypes.data(),
    props_.memoryTypes + props_.memoryTypeCount,
    [flags](vk::MemoryType props) { return flags == (flags & props.propertyFlags); });
  if (mem_type - props_.memoryTypes.data() >= props_.memoryTypeCount) {
    return std::nullopt;
  }
  return dev_.allocateMemory(
    vk::MemoryAllocateInfo{
      .pNext = next,
      .allocationSize = size,
      .memoryTypeIndex = static_cast<uint32_t>(mem_type - props_.memoryTypes.data())});
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
}  // namespace strahl::vulkan::detail