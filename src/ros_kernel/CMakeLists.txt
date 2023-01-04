if ("${CMAKE_BUILD_TYPE}" STREQUAL "Release")
  set(cargo_profile "release")
elseif ("${CMAKE_BUILD_TYPE}" STREQUAL "RelWithDebInfo")
  set(cargo_profile "relwithdebinfo")
else()
  set(cargo_profile "")
endif()

corrosion_import_crate(MANIFEST_PATH Cargo.toml PROFILE ${cargo_profile})

add_library(ROS::Kernel ALIAS ros_kernel)
