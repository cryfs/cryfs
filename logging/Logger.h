#pragma once
#ifndef MESSMER_CPPUTILS_LOGGING_LOGGER_H
#define MESSMER_CPPUTILS_LOGGING_LOGGER_H

#include <messmer/spdlog/include/spdlog/spdlog.h>
#include "../macros.h"

namespace cpputils {
namespace logging {
    class Logger {
    public:
        Logger() : _logger(spdlog::stdout_logger_mt("Log")) { }

        void setLogger(std::shared_ptr<spdlog::logger> logger) {
            _logger = logger;
        }

        spdlog::logger *operator->() {
            return _logger.get();
        }

    private:
        std::shared_ptr<spdlog::logger> _logger;

        DISALLOW_COPY_AND_ASSIGN(Logger);
    };

    inline Logger &logger() {
        static Logger singleton;
        return singleton;
    }
}
}

#endif
