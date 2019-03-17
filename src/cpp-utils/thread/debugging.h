#pragma once
#ifndef MESSMER_CPPUTILS_DEBUGGING_H
#define MESSMER_CPPUTILS_DEBUGGING_H

#include <string>
#include <thread>

namespace cpputils {

void set_thread_name(const char* name);
std::string get_thread_name();

#if defined(__GLIBC__) || defined(__APPLE__) || defined(_MSC_VER)
// this is not supported on musl systems, that's why we ifdef it for glibc only.
std::string get_thread_name(std::thread* thread);
#endif

}

#endif
