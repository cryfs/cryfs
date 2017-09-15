#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_INTEGRITY_INTEGRITYVIOLATIONERROR_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_INTEGRITY_INTEGRITYVIOLATIONERROR_H_

#include <cpp-utils/macros.h>
#include <string>

namespace blockstore {
    namespace integrity {

        class IntegrityViolationError final : public std::exception {
        public:

            const char *what() const throw() override {
                return _reason.c_str();
            }

        private:
            // Constructor is private to make sure that only IntegrityBlockStore can throw this exception.
            // This is because IntegrityBlockStore wants to know about integrity violations and
            // block all further file system access if it happens.
            IntegrityViolationError(const std::string &reason)
                    : _reason("Integrity violation: " + reason) {
            }
            friend class IntegrityBlockStore2;

            std::string _reason;
        };


    }
}

#endif
