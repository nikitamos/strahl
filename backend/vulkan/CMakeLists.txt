include(cmake/slang.cmake)

find_package(Vulkan)
add_library(strahl-vulkan
    SHARED strahl-vulkan.cc
)

# add_slang_shader(out.spv
#     SOURCE test.slang
#     ENTRYPOINTS computeMain
#     TARGET spirv
#     PROFILE glsl_450
# )

# add_dependencies(strahl-vulkan out)
