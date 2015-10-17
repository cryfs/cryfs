#pragma once
#ifndef MESSMER_CPPUTILS_LOGGING_LOGGING_H
#define MESSMER_CPPUTILS_LOGGING_LOGGING_H

#include "Logger.h"
#include <stdexcept>

namespace cpputils {
    namespace logging {
        //TODO Test whole logging folder

        enum Level {
            ERROR, WARN, INFO, DEBUG
        };

        inline void setLogger(std::shared_ptr<spdlog::logger> newLogger) {
            logger().setLogger(newLogger);
        }

        inline spdlog::details::line_logger LOG(Level level) {
            switch(level) {
                case ERROR: return logger()->error();
                case WARN: return logger()->warn();
                case INFO: return logger()->info();
                case DEBUG: return logger()->debug();
            }
            throw std::logic_error("Unknown logger level");
        }
    }
}

#endif
