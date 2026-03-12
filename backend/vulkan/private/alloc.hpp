#pragma once
#include <cstdint>
#include <optional>
#include <stdexcept>
#include <type_traits>
#include <vector>
#include <vulkan/vulkan.hpp>
#include <vulkan/vulkan_to_string.hpp>

#define SVDT_DBG_81934243

namespace strahl::vulkan::detail {
template <typename T>
concept pod = std::is_standard_layout_v<T> && std::is_trivial_v<T>;

class Allocator {
 public:
  static constexpr const auto kStagingFlags =
    vk::MemoryPropertyFlagBits::eHostVisible | vk::MemoryPropertyFlagBits::eHostCached;
  Allocator() {}
  Allocator(vk::PhysicalDevice phy, vk::Device dev);

  std::optional<vk::DeviceMemory> allocate(
    vk::DeviceSize size, vk::MemoryPropertyFlags flags, void *next = nullptr) const;

  std::optional<vk::DeviceMemory> allocStagingMem(
    vk::DeviceSize size,
    vk::MemoryPropertyFlags supported_flags = vk::MemoryPropertyFlags(~0),
    void *next = nullptr) const;

  vk::DeviceMemory allocLocalMem(
    vk::DeviceSize size, vk::MemoryPropertyFlags flags, void *next = nullptr);

  void dealloc(vk::DeviceMemory mem, void *next = nullptr) const;
  vk::DeviceMemory realloc(
    vk::DeviceMemory old, vk::DeviceSize new_size, void *next = nullptr) const;

 private:
  vk::PhysicalDeviceMemoryProperties props_;
  vk::Device dev_;
};

#ifdef SVDT_DBG_81934243
#define T int
#else
template <pod T>
#endif
class GpuVector {
 private:
  void allocateMemory() {
    auto mem_req = vk::StructureChain<vk::MemoryRequirements2, vk::MemoryDedicatedRequirements>{
      vk::MemoryRequirements2{}, vk::MemoryDedicatedRequirements{}};
    auto mem_req_info = vk::BufferMemoryRequirementsInfo2{.buffer = buf_};
    dev_.getBufferMemoryRequirements2(&mem_req_info, &mem_req.get());
    auto &mdr = mem_req.get<vk::MemoryDedicatedRequirements>();

    auto mem_type = mem_req.get().memoryRequirements.memoryTypeBits;
    auto mem_size = mem_req.get().memoryRequirements.size;

    auto mdai = *vk::MemoryDedicatedAllocateInfo{.buffer = buf_};
    mem_ = alloc_->allocStagingMem(mem_size, (vk::MemoryPropertyFlags)mem_type, &mdai).value();
    dev_.bindBufferMemory(buf_, mem_, 0);
  }

 public:
  GpuVector() {}
  GpuVector(
    vk::Device dev,
    vk::Queue queue,
    Allocator *alloc,
    vk::ArrayProxy<uint32_t> family_indices,
    vk::BufferUsageFlags usage = vk::BufferUsageFlagBits::eUniformBuffer |
                                 vk::BufferUsageFlagBits::eStorageBuffer,
    size_t initial_capacity = 16)
    : dev_(dev), queue_(queue), alloc_(alloc) {
    vk::BufferCreateInfo bci{
      .size = initial_capacity * sizeof(T),
      .usage = usage,
      .sharingMode = vk::SharingMode::eExclusive,
      .queueFamilyIndexCount = family_indices.size(),
      .pQueueFamilyIndices = family_indices.data()};
    buf_ = dev_.createBuffer(bci);
    allocateMemory();
  }
  ~GpuVector() {
    dev_.destroy(buf_);
    dev_.freeMemory(mem_);
  }

  GpuVector(GpuVector &&rhs) noexcept(false) { throw std::runtime_error("not implemented"); }
  GpuVector(const GpuVector &) = delete;
  GpuVector &operator=(const GpuVector &) = delete;
  GpuVector &operator=(GpuVector &&rhs) noexcept(false) {
    throw std::runtime_error("not implemented");
  }

  vk::Fence scheduleWrite(vk::Fence fence, vk::CommandBuffer buf) { return {}; }
  vk::Fence scheduleRead(vk::Fence fence, vk::CommandBuffer buf) { return {}; }

 private:
  std::vector<T> host_;
  vk::DeviceSize capacity_ = 0;
  vk::DeviceSize size_ = 0;
  vk::Buffer buf_;
  vk::DeviceMemory mem_;
  vk::Device dev_;
  vk::Queue queue_;
  Allocator *alloc_;
  bool dirty_ = false;
};
}  // namespace strahl::vulkan::detail

#undef SVDT_DBG_81934243
