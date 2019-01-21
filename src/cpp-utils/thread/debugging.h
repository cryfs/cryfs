#pragma once
#ifndef MESSMER_CPPUTILS_DEBUGGING_H
#define MESSMER_CPPUTILS_DEBUGGING_H

#include <string>
#include <thread>

namespace cpputils {

void set_thread_name(const char* name);
std::string get_thread_name();
std::string get_thread_name(std::thread* thread);

}

#endif
