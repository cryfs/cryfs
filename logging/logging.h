#pragma once
#ifndef MESSMER_CPPUTILS_LOGGING_LOGGING_H
#define MESSMER_CPPUTILS_LOGGING_LOGGING_H

#include "Logger.h"
#include <stdexcept>

namespace cpputils {
    namespace logging {
        
        enum Level {
            ERROR, WARN, INFO, DEBUG
        };

        inline void setLogger(std::shared_ptr<spdlog::logger> newLogger) {
            logger().setLogger(newLogger);
        }

        inline void reset() {
            logger().reset();
        }

        inline void setLevel(Level level) {
            switch(level) {
                case ERROR: logger().setLevel(spdlog::level::err); return;
                case WARN: logger().setLevel(spdlog::level::warn); return;
                case INFO: logger().setLevel(spdlog::level::info); return;
                case DEBUG: logger().setLevel(spdlog::level::debug); return;
            }
            throw std::logic_error("Unknown logger level");
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
