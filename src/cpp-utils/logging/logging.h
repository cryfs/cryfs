#pragma once
#ifndef MESSMER_CPPUTILS_LOGGING_LOGGING_H
#define MESSMER_CPPUTILS_LOGGING_LOGGING_H

#include "Logger.h"
#include <stdexcept>
#include <spdlog/fmt/ostr.h>

namespace cpputils {
    namespace logging {

        constexpr struct ERROR_TYPE {} ERROR;
        constexpr struct WARN_TYPE {} WARN;
        constexpr struct INFO_TYPE {} INFO;
        constexpr struct DEBUG_TYPE {} DEBUG;

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

        template<class LogType> inline void LOG(LogType logType, const std::string &msg) {
          LOG(logType, msg.c_str());
        }

        template <typename... Args>
        inline void LOG(ERROR_TYPE, const char* fmt, const Args&... args) {
            logger()->error(fmt, args...);
        }

        template <typename... Args>
        inline void LOG(WARN_TYPE, const char* fmt, const Args&... args) {
            logger()->warn(fmt, args...);
        }

        template <typename... Args>
        inline void LOG(INFO_TYPE, const char* fmt, const Args&... args) {
            logger()->info(fmt, args...);
        }

        template <typename... Args>
        inline void LOG(DEBUG_TYPE, const char* fmt, const Args&... args) {
            logger()->debug(fmt, args...);
        }
    }
}

#endif
