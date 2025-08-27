cmake_minimum_required(VERSION 3.16)
project(msg VERSION 0.1.0)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Add the bebop library
find_package(PkgConfig REQUIRED)

# Header-only library
add_library(msg INTERFACE)

target_include_directories(msg INTERFACE
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include>
)

# Install headers
install(DIRECTORY include/
    DESTINATION include
    FILES_MATCHING PATTERN "*.hpp"
)

# Install CMake files
install(TARGETS msg
    EXPORT msg-targets
    INCLUDES DESTINATION include
)

install(EXPORT msg-targets
    FILE msg-targets.cmake
    NAMESPACE msg::
    DESTINATION lib/cmake/msg
)

# Create config file
include(CMakePackageConfigHelpers)
configure_package_config_file(
    "${CMAKE_CURRENT_SOURCE_DIR}/msg-config.cmake.in"
    "${CMAKE_CURRENT_BINARY_DIR}/msg-config.cmake"
    INSTALL_DESTINATION lib/cmake/msg
)

write_basic_package_version_file(
    "${CMAKE_CURRENT_BINARY_DIR}/msg-config-version.cmake"
    VERSION ${PROJECT_VERSION}
    COMPATIBILITY SameMajorVersion
)

install(FILES
    "${CMAKE_CURRENT_BINARY_DIR}/msg-config.cmake"
    "${CMAKE_CURRENT_BINARY_DIR}/msg-config-version.cmake"
    DESTINATION lib/cmake/msg
)

# Optional: Build examples and tests
option(MSG_BUILD_EXAMPLES "Build examples" OFF)
option(MSG_BUILD_TESTS "Build tests" OFF)

if(MSG_BUILD_EXAMPLES)
    add_subdirectory(examples)
endif()

if(MSG_BUILD_TESTS)
    enable_testing()
    add_subdirectory(tests)
endif()
