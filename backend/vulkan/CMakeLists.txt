include(cmake/slang.cmake)

find_package(Vulkan)
add_library(strahl-vulkan
    SHARED
    strahl-vulkan.cc
    vk-renderer.cc
    private/shader-loader.cc
    private/alloc.cc
)
add_compile_definitions(VULKAN_HPP_NO_STRUCT_CONSTRUCTORS)
target_compile_definitions(
    strahl-vulkan
    PRIVATE VULKAN_HPP_NO_STRUCT_CONSTRUCTORS VULKAN_HPP_USE_STD_EXPECTED
)

target_include_directories(strahl-vulkan PUBLIC include)
target_include_directories(strahl-vulkan PRIVATE private)
# target_precompile_headers()

add_slang_shader(out.spv
    SOURCE test.slang
    ENTRYPOINTS some_calc
    TARGET spirv
    REFLECT ON
    PROFILE spirv_1_5
    SLANG_ARGS -fvk-use-entrypoint-name -fvk-use-c-layout
)

add_dependencies(strahl-vulkan out)
target_link_libraries(strahl-vulkan PUBLIC Vulkan::Headers Vulkan::Vulkan)
