add_library(cpu-raymarcher SHARED ray.cc raymarcher.cc)
target_include_directories(cpu-raymarcher PUBLIC include/)
target_link_libraries(cpu-raymarcher strahl-abstractions)