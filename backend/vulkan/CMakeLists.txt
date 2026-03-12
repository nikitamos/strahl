include(cmake/slang.cmake)

find_package(Vulkan)
add_library(strahl-vulkan
    SHARED
    strahl-vulkan.cc
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

# add_slang_shader(out.spv
#     SOURCE test.slang
#     ENTRYPOINTS computeMain
#     TARGET spirv
#     PROFILE glsl_450
# )

# add_dependencies(strahl-vulkan out)
target_link_libraries(strahl-vulkan PUBLIC Vulkan::Headers Vulkan::Vulkan)
