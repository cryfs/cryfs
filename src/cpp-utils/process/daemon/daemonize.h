#pragma once
#ifndef MESSMER_CPPUTILS_PROCESS_DAEMON_DAEMONIZE_H
#define MESSMER_CPPUTILS_PROCESS_DAEMON_DAEMONIZE_H

#include <functional>
#include "PipeFromChild.h"
#include "PipeToParent.h"

namespace cpputils {
    namespace process {
        PipeFromChild daemonize(std::function<void (PipeToParent *)> childProgram);
    }
}

#endif
