#pragma once
#include <vector>
#include <string_view>
#include <optional>

namespace strahl::vulkan::detail {
std::optional<std::vector<char>> readShader(std::string_view path);
}