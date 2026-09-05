project (fspp-fuse)

set(SOURCES
  ../impl/FilesystemImpl.cpp
  ../impl/Profiler.cpp
  ../fuse/Fuse.cpp
  ../fuse/AtimeOptions.cpp
)

add_library(${PROJECT_NAME} STATIC ${SOURCES})

target_compile_definitions(${PROJECT_NAME} PUBLIC _FILE_OFFSET_BITS=64)
target_link_libraries(${PROJECT_NAME} PUBLIC cpp-utils fspp-interface)

target_add_boost(${PROJECT_NAME})
target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})

if(${CMAKE_SYSTEM_NAME} MATCHES "Windows")
  get_target_architecture(TARGET_ARCH)
  message(STATUS "Linking to Dokan ${TARGET_ARCH}")
  if ("${TARGET_ARCH}" STREQUAL "x86_64")
	set(DOKAN_LIB_PATH "${DOKAN_PATH}")
  elseif("${TARGET_ARCH}" STREQUAL "i386")
    set(DOKAN_LIB_PATH "${DOKAN_PATH}/x86")
  else()
    message(FATAL_ERROR "Unsupported architecture: ${TARGET_ARCH}")
  endif()

  target_include_directories(${PROJECT_NAME} PUBLIC "${DOKAN_PATH}/include")
  target_link_libraries(${PROJECT_NAME} PUBLIC "${DOKAN_LIB_PATH}/lib/dokan2.lib")
  #target_link_libraries(${PROJECT_NAME} PUBLIC "${DOKAN_LIB_PATH}/lib/dokannp2.lib")
  target_link_libraries(${PROJECT_NAME} PUBLIC "${DOKAN_LIB_PATH}/lib/dokanfuse2.lib")

  install(FILES "${DOKAN_LIB_PATH}/dokan2.dll" "${DOKAN_LIB_PATH}/dokanfuse2.dll"
    DESTINATION "${CMAKE_INSTALL_BINDIR}"
  )

else() # Linux and macOS
  find_package(PkgConfig REQUIRED)
  if(${CMAKE_SYSTEM_NAME} MATCHES "Darwin")
    # macFUSE installs into /usr/local, which isn't on pkg-config's default search path when
    # pkg-config itself comes from Homebrew on Apple Silicon.
    set(ENV{PKG_CONFIG_PATH} "/usr/local/lib/pkgconfig:$ENV{PKG_CONFIG_PATH}")
  endif()
  pkg_check_modules(Fuse3 IMPORTED_TARGET fuse3)
  if(NOT Fuse3_FOUND)
    if(${CMAKE_SYSTEM_NAME} MATCHES "Darwin")
      message(FATAL_ERROR "Didn't find libfuse 3. On macOS it comes with macFUSE 4.10.0 and newer - install it from https://osxfuse.github.io/ or with 'brew install --cask macfuse'. If macFUSE is already installed, point pkg-config at it with PKG_CONFIG_PATH=/usr/local/lib/pkgconfig.")
    else()
      message(FATAL_ERROR "Didn't find libfuse 3. Install it, e.g. 'apt install libfuse3-dev' or 'dnf install fuse3-devel'.")
    endif()
  endif()
  target_link_libraries(${PROJECT_NAME} PUBLIC PkgConfig::Fuse3)
endif()

if(${CMAKE_SYSTEM_NAME} MATCHES "Darwin")
  set(CMAKE_FIND_FRAMEWORK LAST)
endif()
