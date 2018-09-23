project (parallelaccessstore)

set(SOURCES
  ParallelAccessBaseStore.cpp
  ParallelAccessStore.cpp
)

add_library(${PROJECT_NAME} STATIC ${SOURCES})

target_link_libraries(${PROJECT_NAME} PUBLIC cpp-utils)

target_add_boost(${PROJECT_NAME} thread)
target_enable_style_warnings(${PROJECT_NAME})
target_activate_cpp14(${PROJECT_NAME})

# Needed so it can be linked in to the shared cryfs library
set_target_properties(${PROJECT_NAME} PROPERTIES POSITION_INDEPENDENT_CODE on)
