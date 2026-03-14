#include "vk-renderer.hpp"

#include <cstdint>
#include <vulkan/vulkan.hpp>

#include "private/device-queue.hpp"
#include "private/shader-loader.hpp"

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
VulkanRenderer::VulkanRenderer(detail::DeviceQueueInfo *dqi) : dqi_(dqi) {
  auto shader = detail::readShader("out.spv").value();
  vk::ShaderModuleCreateInfo smci{
    .codeSize = shader.size(),
    .pCode = (uint32_t *)shader.data(),
  };
  auto module = dqi_->dev.createShaderModule(smci);
  createCommandPoolBufs();

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
  vk::CommandBufferBeginInfo cbbi{.flags = vk::CommandBufferUsageFlagBits::eOneTimeSubmit};
  com_buffer_.begin(cbbi);
  tx_buffer_.begin(cbbi);

  /// RENDERING HAPPENS HERE

  tx_buffer_.end();
  com_buffer_.end();
  vk::SubmitInfo si{.commandBufferCount = 1, .pCommandBuffers = &tx_buffer_};
  dqi_->tx.submit(si);
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
