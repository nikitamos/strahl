cmake_minimum_required(VERSION 4.0)
project(Strahl LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 23)
find_package(glm REQUIRED)

add_subdirectory(abstractions)
add_subdirectory(backend/cpu-raymarcher)
add_subdirectory(backend/vulkan)

add_subdirectory(frontends/qt-gui)

add_executable(strahl-cli frontends/cli/main.cc)
target_link_libraries(strahl-cli
    strahl-abstractions
    cpu-raymarcher
    png
)
