project (fspp-fuse)

set(SOURCES
  ../impl/FilesystemImpl.cpp
  ../impl/Profiler.cpp
  ../fuse/Fuse.cpp
)

add_library(${PROJECT_NAME} STATIC ${SOURCES})

target_compile_definitions(${PROJECT_NAME} PUBLIC _FILE_OFFSET_BITS=64)
target_link_libraries(${PROJECT_NAME} PUBLIC cpp-utils fspp-interface)

target_add_boost(${PROJECT_NAME} filesystem system thread chrono)
target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})

if(${CMAKE_SYSTEM_NAME} MATCHES "Windows")
  get_target_architecture(TARGET_ARCH)
  if ("${TARGET_ARCH}" STREQUAL "x86_64")
	set(DOKAN_LIB_PATH "${DOKAN_PATH}\\lib")
  elseif("${TARGET_ARCH}" STREQUAL "i386")
    set(DOKAN_LIB_PATH "${DOKAN_PATH}\\x86\\lib")
  else()
    message(FATAL_ERROR "Unsupported architecture: ${TARGET_ARCH}")
  endif()

  target_include_directories(${PROJECT_NAME} PUBLIC "${DOKAN_PATH}\\include")
  target_link_libraries(${PROJECT_NAME} PUBLIC "${DOKAN_LIB_PATH}\\dokan1.lib")
  #target_link_libraries(${PROJECT_NAME} PUBLIC "${DOKAN_LIB_PATH}\\dokannp1.lib")
  target_link_libraries(${PROJECT_NAME} PUBLIC "${DOKAN_LIB_PATH}\\dokanfuse1.lib")

elseif(${CMAKE_SYSTEM_NAME} MATCHES "Darwin")
  set(CMAKE_FIND_FRAMEWORK LAST)
  find_library_with_path(FUSE "osxfuse" FUSE_LIB_PATH)
  target_link_libraries(${PROJECT_NAME} PUBLIC ${FUSE})
else() # Linux
  find_library_with_path(FUSE "fuse" FUSE_LIB_PATH)
  target_link_libraries(${PROJECT_NAME} PUBLIC ${FUSE})
endif()

