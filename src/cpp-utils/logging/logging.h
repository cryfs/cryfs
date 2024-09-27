#pragma once
#ifndef MESSMER_CPPUTILS_LOGGING_LOGGING_H
#define MESSMER_CPPUTILS_LOGGING_LOGGING_H

#include "Logger.h"
#include <stdexcept>
#include <spdlog/fmt/ostr.h>
#include <spdlog/sinks/basic_file_sink.h>

#if defined(_MSC_VER)
#include <spdlog/sinks/msvc_sink.h>
#else
#include <spdlog/sinks/syslog_sink.h>
#endif

namespace cpputils {
    namespace logging {

		struct ERROR_TYPE {};
		struct WARN_TYPE {};
		struct INFO_TYPE {};
		struct DEBUG_TYPE {};

        constexpr ERROR_TYPE ERR {};
        constexpr WARN_TYPE WARN {};
        constexpr INFO_TYPE INFO {};
        constexpr DEBUG_TYPE DEBUG {};

        inline void setLogger(std::shared_ptr<spdlog::logger> newLogger) {
            logger().setLogger(newLogger);
        }

        inline void reset() {
            logger().reset();
        }

        inline void flush() {
            logger()->flush();
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

        template <typename... Args>
        inline void LOG(ERROR_TYPE, spdlog::format_string_t<Args...> fmt, Args&&... args) {
            logger()->error(fmt, std::forward<Args>(args)...);
        }

        template <typename... Args>
        inline void LOG(WARN_TYPE, spdlog::format_string_t<Args...> fmt, Args&&... args) {
            logger()->warn(fmt, std::forward<Args>(args)...);
        }

        template <typename... Args>
        inline void LOG(INFO_TYPE, spdlog::format_string_t<Args...> fmt, Args&&... args) {
            logger()->info(fmt, std::forward<Args>(args)...);
        }

        template <typename... Args>
        inline void LOG(DEBUG_TYPE, spdlog::format_string_t<Args...> fmt, Args&&... args) {
            logger()->debug(fmt, std::forward<Args>(args)...);
        }

        inline std::shared_ptr<spdlog::logger> system_logger(const std::string& name) {
#if defined(_MSC_VER)
          return spdlog::create<spdlog::sinks::msvc_sink_mt>(name);
#else
          return spdlog::syslog_logger_mt(name, name, LOG_PID);
#endif
        }
    }
}

#endif
