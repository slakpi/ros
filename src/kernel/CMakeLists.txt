corrosion_import_crate(MANIFEST_PATH Cargo.toml)
target_include_directories(ros_kernel INTERFACE "${CMAKE_CURRENT_LIST_DIR}/c")
add_library(ROS::Lib ALIAS ros_kernel)
