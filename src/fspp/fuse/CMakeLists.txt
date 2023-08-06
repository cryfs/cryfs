project (fspp-fuse)

set(SOURCES
  ../impl/FilesystemImpl.cpp
  ../impl/Profiler.cpp
  ../fuse/Fuse.cpp
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

elseif(${CMAKE_SYSTEM_NAME} MATCHES "Darwin") # macOS
  find_package(PkgConfig REQUIRED)
  pkg_check_modules(Fuse REQUIRED IMPORTED_TARGET fuse)
  target_link_libraries(${PROJECT_NAME} PUBLIC PkgConfig::Fuse)
else() # Linux
  find_package(PkgConfig REQUIRED)
  pkg_check_modules(Fuse3 REQUIRED IMPORTED_TARGET fuse3)
  target_link_libraries(${PROJECT_NAME} PUBLIC PkgConfig::Fuse3)
endif()

if(${CMAKE_SYSTEM_NAME} MATCHES "Darwin")
  set(CMAKE_FIND_FRAMEWORK LAST)
endif()
