#pragma once
#ifndef MESSMER_CPPUTILS_PROCESS_DAEMONIZE_H
#define MESSMER_CPPUTILS_PROCESS_DAEMONIZE_H

#include <functional>

namespace cpputils {
    void daemonize(std::function<void ()> childProgram);
}

#endif
