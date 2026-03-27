#include <stdexcept>

#include "alloc.hpp"
#include "device-queue.hpp"
#include "vulkan/vulkan.hpp"

#define SVDT_DBG_81934243

namespace strahl::vulkan::detail {
template <typename T>
concept pod = std::is_standard_layout_v<T> && std::is_trivial_v<T>;

#ifdef SVDT_DBG_81934243
#define T int
#else
template <pod T>
#endif
class GpuVector {
 private:
  void allocateMemory() {
    // Allocate STAGING buffer & setup some vars
    auto mem_req = vk::StructureChain<vk::MemoryRequirements2, vk::MemoryDedicatedRequirements>{
      vk::MemoryRequirements2{}, vk::MemoryDedicatedRequirements{}};
    auto mem_req_info = vk::BufferMemoryRequirementsInfo2{.buffer = stage_buf_};
    dev_.getBufferMemoryRequirements2(&mem_req_info, &mem_req.get());
    auto mem_type = mem_req.get().memoryRequirements.memoryTypeBits;
    auto mem_size = mem_req.get().memoryRequirements.size;
    auto mdai = *vk::MemoryDedicatedAllocateInfo{.buffer = stage_buf_};

    stage_mem_ = alloc_->allocStagingMem(mem_size, mem_type, &mdai).value();
    dev_.bindBufferMemory(stage_buf_, stage_mem_, 0);

    // Allocate DEVICE buffer
    mem_req_info = {.buffer = dev_buf_};
    mem_req = {};
    dev_.getBufferMemoryRequirements2(&mem_req_info, &mem_req.get());
    mem_type = mem_req.get().memoryRequirements.memoryTypeBits;
    mem_size = mem_req.get().memoryRequirements.size;
    mdai = *vk::MemoryDedicatedAllocateInfo{.buffer = dev_buf_};

    dev_mem_ = alloc_->allocDeviceMem(mem_size, mem_type, &mdai).value();
    dev_.bindBufferMemory(dev_buf_, dev_mem_, 0);
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
    : dev_(dev),
      alloc_(alloc),
      capacity_(initial_capacity),
      dirty_left_(initial_capacity),
      dirty_right_(0) {
    vk::BufferCreateInfo bci{
      .size = initial_capacity * sizeof(T),
      .usage = vk::BufferUsageFlagBits::eTransferDst | vk::BufferUsageFlagBits::eTransferSrc,
      .sharingMode = vk::SharingMode::eExclusive,
      .queueFamilyIndexCount = family_indices.size(),
      .pQueueFamilyIndices = family_indices.data()};
    stage_buf_ = dev_.createBuffer(bci);
    bci = {
      .size = initial_capacity * sizeof(T),
      .usage = vk::BufferUsageFlagBits::eTransferDst | usage,
      .sharingMode = vk::SharingMode::eExclusive,
      .queueFamilyIndexCount = family_indices.size(),
      .pQueueFamilyIndices = family_indices.data()};
    dev_buf_ = dev_.createBuffer(bci);
    allocateMemory();
  }
  ~GpuVector() {
    dev_.destroy(stage_buf_);
    dev_.freeMemory(stage_mem_);
    dev_.destroy(dev_buf_);
    dev_.freeMemory(dev_mem_);
  }

  GpuVector(GpuVector &&rhs) noexcept(false) { throw std::runtime_error("not implemented"); }
  GpuVector(const GpuVector &) = delete;
  GpuVector &operator=(const GpuVector &) = delete;
  GpuVector &operator=(GpuVector &&rhs) noexcept(false) {
    throw std::runtime_error("not implemented");
  }

  void scheduleWrite(
    QueueRecorder tx_buf,
    QueueRecorder work_buf,
    vk::AccessFlagBits2KHR access,
    vk::PipelineStageFlags2 stage = vk::PipelineStageFlagBits2::eAllCommands) {
    if (dirty_left_ >= dirty_right_) {
      return;
    }
    auto copy_size = sizeof(T) * (dirty_right_ - dirty_left_);
    auto offset = sizeof(T) * dirty_left_;
    dev_.flushMappedMemoryRanges(
      vk::MappedMemoryRange{.memory = stage_mem_, .offset = offset, .size = copy_size});

    tx_buf->copyBuffer(stage_buf_, dev_buf_, vk::BufferCopy{0, offset, copy_size});
    transferBufferOwnership(
      dev_buf_,
      tx_buf,
      work_buf,
      vk::PipelineStageFlagBits2::eCopy,
      stage,
      vk::AccessFlagBits2::eTransferWrite,
      access,
      offset,
      copy_size);
    // TODO: don't transfer if the same queue is used for for transfer and compute
  }
  vk::Fence scheduleRead(vk::Fence fence, vk::CommandBuffer buf) {
    auto size = capacity_ * sizeof(T);
    buf.copyBuffer(dev_buf_, stage_buf_, vk::BufferCopy{0, 0, size});
    dev_.invalidateMappedMemoryRanges(
      vk::MappedMemoryRange{.memory = stage_mem_, .offset = 0, .size = size});
    return {};
  }

  static inline void transferBufferOwnership(
    vk::Buffer buf,
    QueueRecorder &src,
    QueueRecorder &dst,
    vk::PipelineStageFlags2 src_stage,
    vk::PipelineStageFlags2 dst_stage,
    vk::AccessFlags2 src_access,
    vk::AccessFlags2 dst_access,
    vk::DeviceSize offset,
    vk::DeviceSize size) {
    vk::BufferMemoryBarrier2 barrier{
      .srcStageMask = src_stage,
      .srcAccessMask = src_access,
      .srcQueueFamilyIndex = src.family(),
      .dstQueueFamilyIndex = dst.family(),
      .buffer = buf,
      .offset = offset,
      .size = size};
    auto dep = vk::DependencyInfo{.bufferMemoryBarrierCount = 1, .pBufferMemoryBarriers = &barrier};
    src->pipelineBarrier2(dep);
    // SRC masks is just ignored by implementation, right?
    barrier.setDstAccessMask(dst_access);
    barrier.setDstStageMask(dst_stage);
    dst->pipelineBarrier2(dep);
  }

  void invalidate() {
    dirty_left_ = 0;
    dirty_right_ = capacity_;
  }
  vk::DeviceSize size() const { return size_; }
  vk::DeviceSize capacity() const { return capacity_; }
  const T &operator[](vk::DeviceSize i) const { return mapped_[i]; }
  T &operator[](vk::DeviceSize i) {
    dirty_left_ = std::min(dirty_left_, i);
    dirty_right_ = std::max(dirty_right_, i + 1);
    return mapped_[i];
  }
  const T &get(vk::DeviceSize i) const { return mapped_[i]; }
  T &get(vk::DeviceSize i) {
    dirty_left_ = std::min(dirty_left_, i);
    dirty_right_ = std::max(dirty_right_, i + 1);
    return mapped_[i];
  }
  vk::Buffer getDeviceBuffer() const { return dev_buf_; }

 private:
  T *mapped_;
  vk::DeviceSize capacity_ = 0;
  vk::DeviceSize size_ = 0;
  uint32_t tx_;
  uint32_t exec_;

  vk::DeviceMemory stage_mem_;
  vk::Buffer stage_buf_;
  vk::DeviceMemory dev_mem_;
  vk::Buffer dev_buf_;

  vk::Device dev_;
  Allocator *alloc_;

  vk::DeviceSize dirty_left_;
  vk::DeviceSize dirty_right_;
};
}  // namespace strahl::vulkan::detail

#undef SVDT_DBG_81934243
