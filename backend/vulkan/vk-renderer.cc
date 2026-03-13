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
  auto pipeline = dqi_->dev.createComputePipeline(nullptr, cpci).value;

  dqi_->dev.destroyPipeline(pipeline);
  dqi_->dev.destroy(layout);
  dqi_->dev.destroy(set0);
  dqi_->dev.destroy(module);
}

}  // namespace strahl::vulkan
