Local CMake Settings
====================

The `cmake-kits.json` file points Code to the toolchain files in the `cmake`
folder. A local `settings.json` can be used to configure the build output
directory and the `TC_PATH` variable to the compiles. For example:

    {
      "cmake.cmakePath": "${env:HOME}/.brew/bin/cmake",
      "cmake.buildDirectory": "${workspaceFolder}/build/${buildType}/${buildKit}",
      "cmake.configureSettings": {
        "TC_PATH": "${env:HOME}/.local/cross",
        "RPI_VERSION": "3",
        "ROS_FEATURES": "module_tests",
      }
    }
