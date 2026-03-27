#include "vk-renderer.hpp"

#include <cstdint>
#include <vulkan/vulkan.hpp>

#include "private/device-queue.hpp"
#include "private/gpu-vector.hpp"
#include "private/shader-loader.hpp"
#include "vulkan/vulkan.hpp"

// NOLINTBEGIN
struct DescriptorSet0Layout {
  vk::DescriptorSetLayoutBinding in___Array_std140_int8{
    .binding = 0,
    .descriptorType = vk::DescriptorType(6),
    .descriptorCount = 1,
    .stageFlags = vk::ShaderStageFlags(2147483647)};
  vk::DescriptorSetLayoutBinding out__RWStructuredBuffer{
    .binding = 1,
    .descriptorType = vk::DescriptorType(7),
    .descriptorCount = 1,
    .stageFlags = vk::ShaderStageFlags(2147483647)};
  operator vk::ArrayProxy<vk::DescriptorSetLayoutBinding>() const {
    return {2, (vk::DescriptorSetLayoutBinding *)this};
  }
  vk::DescriptorSetLayoutCreateInfo CreateInfo(vk::DescriptorSetLayoutCreateFlags flags = {}) {
    return {.flags = flags, .bindingCount = 2, .pBindings = (vk::DescriptorSetLayoutBinding *)this};
  }
};
// NOLINTEND

namespace strahl::vulkan {
VulkanRenderer::VulkanRenderer(detail::DeviceQueueInfo *dqi, detail::GpuVector *vec)
  : dqi_(dqi), vec_(vec) {
  vec_->get(0) = 13;
  vec_->get(vec_->capacity() - 1) = 251;

  auto shader = detail::readShader("out.spv").value();
  vk::ShaderModuleCreateInfo smci{
    .codeSize = shader.size(),
    .pCode = (uint32_t *)shader.data(),
  };
  auto module = dqi_->dev.createShaderModule(smci);
  createCommandPoolBufs();

  vk::StructureChain<vk::SemaphoreCreateInfo, vk::SemaphoreTypeCreateInfo> sci{
    {},
    vk::SemaphoreTypeCreateInfo{
      .semaphoreType = vk::SemaphoreType::eTimeline,
      .initialValue = 1,
    }};
  com_end_ = dqi_->dev.createSemaphore(sci.get());
  sci.get<vk::SemaphoreTypeCreateInfo>().initialValue = 0;
  tx2com_ = dqi->dev.createSemaphore(sci.get());
  dqi_->dev.signalSemaphore(vk::SemaphoreSignalInfo{.semaphore = com_end_, .value = count_});

  DescriptorSet0Layout l;
  auto dslci = l.CreateInfo();
  auto set0 = dqi_->dev.createDescriptorSetLayout(dslci);
  vk::PipelineLayoutCreateInfo plci{.setLayoutCount = 1, .pSetLayouts = &set0};
  auto layout = dqi_->dev.createPipelineLayout(plci);

  vk::PipelineShaderStageCreateInfo sci{
    .stage = vk::ShaderStageFlagBits::eCompute, .module = module, .pName = "main"};
  vk::ComputePipelineCreateInfo cpci{
    .stage = sci,
    .layout = layout,
  };
  pipeline_ = dqi_->dev.createComputePipeline(nullptr, cpci).value;

  // dqi_->dev.destroyPipeline(pipeline);
  dqi_->dev.destroy(layout);
  dqi_->dev.destroy(set0);
  dqi_->dev.destroy(module);
}

void VulkanRenderer::doSomeRendering() {
  // PRE-CONDITION: this->count_ == counter of tx2com_ == counter of com_end_
  vk::CommandBufferBeginInfo cbbi{.flags = vk::CommandBufferUsageFlagBits::eOneTimeSubmit};
  com_buffer_.begin(cbbi);
  tx_buffer_.begin(cbbi);

  /// RENDERING HAPPENS HERE

  tx_buffer_.end();
  com_buffer_.end();
  uint64_t one = 1;
  vk::PipelineStageFlags wait_dst = vk::PipelineStageFlagBits::eAllCommands;
  vk::StructureChain<vk::SubmitInfo, vk::TimelineSemaphoreSubmitInfo> si{
    vk::SubmitInfo{
      .waitSemaphoreCount = 1,
      .pWaitSemaphores = &com_end_,
      .pWaitDstStageMask = &wait_dst,
      .commandBufferCount = 1,
      .pCommandBuffers = &tx_buffer_,
      .signalSemaphoreCount = 1,
      .pSignalSemaphores = &tx2com_,
    },
    vk::TimelineSemaphoreSubmitInfo{
      .waitSemaphoreValueCount = 1,
      .pWaitSemaphoreValues = &count_,
      .signalSemaphoreValueCount = 1,
      .pSignalSemaphoreValues = &one}};
  dqi_->tx.submit(si.get());
  count_ += 1;
  // Swap wait and signal semaphores
  // clang-format off
  si.get() // How to make normal wrapping with clang-format?
    .setPSignalSemaphores(&com_end_)
    .setPWaitSemaphores(&tx2com_)
    .setPCommandBuffers(&com_buffer_);
  // clang-format on
  dqi_->com.submit(si.get());
}
void VulkanRenderer::createCommandPoolBufs() {
  vk::CommandPoolCreateInfo cpci = {
    .flags = vk::CommandPoolCreateFlagBits::eResetCommandBuffer,
    .queueFamilyIndex = dqi_->com_family,
  };
  com_pool_ = dqi_->dev.createCommandPool(cpci);
  cpci.setQueueFamilyIndex(dqi_->tx_family);
  tx_pool_ = dqi_->dev.createCommandPool(cpci);

  vk::CommandBufferAllocateInfo cbai{
    .commandPool = com_pool_, .level = vk::CommandBufferLevel::ePrimary, .commandBufferCount = 1};
  com_buffer_ = dqi_->dev.allocateCommandBuffers(cbai)[0];
  cbai.setCommandPool(tx_pool_);
  tx_buffer_ = dqi_->dev.allocateCommandBuffers(cbai)[0];
}
VulkanRenderer::~VulkanRenderer() {
  std::cout << "destroying renderer" << std::endl;
  dqi_->dev.destroy(tx_pool_);
  dqi_->dev.destroy(com_pool_);
  dqi_->dev.destroy(pipeline_);
}
}  // namespace strahl::vulkan
