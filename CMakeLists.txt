cmake_minimum_required(VERSION 4.0)
project(Strahl LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
find_package(glm REQUIRED)

add_library(CImg INTERFACE)
target_sources(CImg
    INTERFACE FILE_SET
        HEADERS BASE_DIRS ${CMAKE_CURRENT_SOURCE_DIR}/3rdparty FILES 3rdparty/CImg.h
)
target_precompile_headers(CImg
     INTERFACE 3rdparty/CImg.h
)

add_subdirectory(abstractions)
add_subdirectory(backend/cpu-raymarcher)

add_executable(strahl-cli frontends/cli/main.cc)
target_link_libraries(strahl-cli
    strahl-abstractions
    cpu-raymarcher
    CImg
)
