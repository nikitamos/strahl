find_package(OpenMP REQUIRED)

add_library(cpu-raymarcher SHARED
    ray.cc raymarcher.cc rm-scene.cc)
target_include_directories(cpu-raymarcher PUBLIC include/)
target_link_libraries(cpu-raymarcher strahl-abstractions glm::glm OpenMP::OpenMP_CXX)