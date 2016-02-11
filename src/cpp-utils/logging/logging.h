#pragma once
#ifndef MESSMER_CPPUTILS_LOGGING_LOGGING_H
#define MESSMER_CPPUTILS_LOGGING_LOGGING_H

#include "Logger.h"
#include <stdexcept>

namespace cpputils {
    namespace logging {

        extern struct ERROR_TYPE {} ERROR;
        extern struct WARN_TYPE {} WARN;
        extern struct INFO_TYPE {} INFO;
        extern struct DEBUG_TYPE {} DEBUG;

        inline void setLogger(std::shared_ptr<spdlog::logger> newLogger) {
            logger().setLogger(newLogger);
        }

        inline void reset() {
            logger().reset();
        }

        inline void setLevel(ERROR_TYPE) {
            logger().setLevel(spdlog::level::err);
        }

        inline void setLevel(WARN_TYPE) {
            logger().setLevel(spdlog::level::warn);
        }

        inline void setLevel(INFO_TYPE) {
            logger().setLevel(spdlog::level::info);
        }

        inline void setLevel(DEBUG_TYPE) {
            logger().setLevel(spdlog::level::debug);
        }

        inline spdlog::details::line_logger LOG(ERROR_TYPE) {
            return logger()->error();
        }

        inline spdlog::details::line_logger LOG(WARN_TYPE) {
            return logger()->warn();
        }

        inline spdlog::details::line_logger LOG(INFO_TYPE) {
            return logger()->info();
        }

        inline spdlog::details::line_logger LOG(DEBUG_TYPE) {
            return logger()->debug();
        }
    }
}

#endif
