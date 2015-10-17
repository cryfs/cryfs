#pragma once
#ifndef MESSMER_CPPUTILS_DAEMON_DAEMONIZE_H
#define MESSMER_CPPUTILS_DAEMON_DAEMONIZE_H

#include <string>

namespace cpputils {
    void daemonize(const std::string &daemonName);
}

#endif
