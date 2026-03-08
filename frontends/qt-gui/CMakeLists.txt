find_package(Qt6 COMPONENTS Qml Quick Widgets REQUIRED)
add_executable(strahl-qui main.cc)
target_link_libraries(strahl-qui Qt6::Qml Qt6::Quick Qt6::Widgets)