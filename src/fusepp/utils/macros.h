#pragma once
#ifndef FUSEPP_UTILS_MACROS_H_
#define FUSEPP_UTILS_MACROS_H_

#define DISALLOW_COPY_AND_ASSIGN(Class)        \
  Class(const Class &rhs) = delete;            \
  Class &operator=(const Class &rhs) = delete;

#endif /* FUSEPP_UTILS_MACROS_H_ */
