#pragma once
#ifndef MESSMER_CPPUTILS_LOGGING_LOGGER_H
#define MESSMER_CPPUTILS_LOGGING_LOGGER_H

#include <spdlog/spdlog.h>
#include "../macros.h"

#include <spdlog/sinks/stdout_sinks.h>

namespace cpputils {
namespace logging {
    class Logger final {
    public:
        void setLogger(std::shared_ptr<spdlog::logger> logger) {
            _logger = logger;
            _logger->set_level(_level);
        }

        void reset() {
            _level = spdlog::level::info;
            setLogger(_defaultLogger());
        }

        void setLevel(spdlog::level::level_enum level) {
            _level = level;
            _logger->set_level(_level);
        }

        spdlog::logger *operator->() {
            return _logger.get();
        }

    private:

        static std::shared_ptr<spdlog::logger> _defaultLogger() {
            static auto singleton = spdlog::stderr_logger_mt("Log");
            return singleton;
        }

        Logger() : _logger(), _level() {
            reset();
        }
        friend Logger &logger();

        std::shared_ptr<spdlog::logger> _logger;
        spdlog::level::level_enum _level;

        DISALLOW_COPY_AND_ASSIGN(Logger);
    };

    inline Logger &logger() {
        static Logger singleton;
        return singleton;
    }
}
}

#endif
