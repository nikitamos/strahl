add_library(strahl_cpu SHARED lib.cc)
target_include_directories(strahl_cpu PUBLIC include/)
target_link_libraries(strahl_cpu PUBLIC strahl-abstractions)
