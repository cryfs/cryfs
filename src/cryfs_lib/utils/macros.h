#pragma once
#ifndef CRYFS_LIB_UTILS_MACROS_H_
#define CRYFS_LIB_UTILS_MACROS_H_

#define DISALLOW_COPY_AND_ASSIGN(Class)        \
  Class(const Class &rhs) = delete;            \
  Class &operator=(const Class &rhs) = delete;

#endif /* CRYFS_LIB_UTILS_MACROS_H_ */
