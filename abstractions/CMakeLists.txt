add_library(strahl-abstractions STATIC dummy.cc)
target_link_libraries(strahl-abstractions glm)
target_include_directories(strahl-abstractions PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})
