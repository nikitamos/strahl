#include <vector>
#include <string_view>
#include <optional>
#include <fstream>

#include "shader-loader.hpp"

namespace strahl::vulkan::detail {
std::optional<std::vector<char>> readShader(const std::string_view path) {
  int a = 3;
  std::ifstream shader_file(std::string(path), std::ios_base::ate | std::ios_base::binary);
  if (!shader_file.is_open()) {
    throw std::runtime_error("failed to open shader file");
  }
  std::vector<char> shader_src(shader_file.tellg());
  shader_file.seekg(0, std::ios::beg);
  shader_file.read(shader_src.data(), shader_src.size());
  return shader_src;
}
}  // namespace strahl::vulkan::detail