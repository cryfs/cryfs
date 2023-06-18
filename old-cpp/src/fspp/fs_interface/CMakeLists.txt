project (fspp-interface)

set(SOURCES
  Device.cpp
  Dir.cpp
  File.cpp
  Node.cpp
  OpenFile.cpp
  Symlink.cpp
  Types.cpp
  Context.cpp
)

add_library(${PROJECT_NAME} STATIC ${SOURCES})

target_link_libraries(${PROJECT_NAME} PUBLIC cpp-utils)

target_add_boost(${PROJECT_NAME})
target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})
