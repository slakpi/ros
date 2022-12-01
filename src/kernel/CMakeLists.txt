corrosion_import_crate(MANIFEST_PATH Cargo.toml)

target_include_directories(ros_kernel INTERFACE ${CMAKE_CURRENT_BINARY_DIR})
add_library(ROS::Kernel ALIAS ros_kernel)
