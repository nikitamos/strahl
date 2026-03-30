add_library(strahl_cpu SHARED
    cpu-raytracer.cc
    geometry.cc
    material.cc
    nodes.cc
    scene.cc
    solver.cc
)
target_include_directories(strahl_cpu PUBLIC include/)
target_link_libraries(strahl_cpu PUBLIC strahl-abstractions)
