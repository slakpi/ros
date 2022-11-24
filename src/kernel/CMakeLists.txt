add_subdirectory(arch)

corrosion_import_crate(MANIFEST_PATH Cargo.toml)
target_include_directories(ros_kernel INTERFACE "${CMAKE_CURRENT_LIST_DIR}/c")
target_link_libraries(ros_kernel INTERFACE ROS::Arch)
add_library(ROS::Kernel ALIAS ros_kernel)
