#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_INTEGRITY_INTEGRITYVIOLATIONERROR_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_INTEGRITY_INTEGRITYVIOLATIONERROR_H_

#include <cpp-utils/macros.h>
#include <string>
#include <stdexcept>

namespace blockstore {
    namespace integrity {

        class IntegrityViolationError final : public std::runtime_error {
        private:
            // Constructor is private to make sure that only IntegrityBlockStore can throw this exception.
            // This is because IntegrityBlockStore wants to know about integrity violations and
            // block all further file system access if it happens.
            IntegrityViolationError(const std::string &reason)
                    : std::runtime_error("Integrity violation: " + reason) {
            }
            friend class IntegrityBlockStore2;
        };


    }
}

#endif
