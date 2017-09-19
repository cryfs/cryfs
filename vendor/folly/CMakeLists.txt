project (folly)

set(SOURCES
  dummy.cpp
)

# Folly needs a folly-config.h with some macros.
# But luckily none of these macros matter for the subset of files we took from folly.
# So we can just create an empty file.
file(WRITE ${CMAKE_CURRENT_BINARY_DIR}/folly/folly/folly-config.h "")

add_library(${PROJECT_NAME} STATIC ${SOURCES})
target_include_directories(${PROJECT_NAME} SYSTEM PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/folly ${CMAKE_CURRENT_BINARY_DIR}/folly)
