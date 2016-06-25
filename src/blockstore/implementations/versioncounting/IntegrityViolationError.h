#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_INTEGRITYVIOLATIONERROR_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_INTEGRITYVIOLATIONERROR_H_

#include <cpp-utils/macros.h>
#include <string>

namespace blockstore {
    namespace versioncounting {

        class IntegrityViolationError final : public std::exception {
        public:
            IntegrityViolationError(const std::string &reason)
                    : _reason("Integrity violation: " + reason) {
            }

            const char *what() const throw() override {
                return _reason.c_str();
            }

        private:
            std::string _reason;
        };


    }
}

#endif
