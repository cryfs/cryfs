#pragma once
#ifndef FSPP_UTILS_MACROS_H_
#define FSPP_UTILS_MACROS_H_

#define DISALLOW_COPY_AND_ASSIGN(Class)        \
  Class(const Class &rhs) = delete;            \
  Class &operator=(const Class &rhs) = delete;

#define UNUSED(expr) (void)(expr)

#endif /* FSPP_UTILS_MACROS_H_ */
