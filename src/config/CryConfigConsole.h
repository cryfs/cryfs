#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGCONSOLE_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGCONSOLE_H

#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <messmer/cpp-utils/io/Console.h>
#include <boost/optional.hpp>

namespace cryfs {
    class CryConfigConsole final {
    public:
        CryConfigConsole(std::shared_ptr<cpputils::Console> console);
        CryConfigConsole(CryConfigConsole &&rhs) = default;

        std::string askCipher() const;

    private:
        static constexpr const char *DEFAULT_CIPHER = "aes-256-gcm";

        bool _showWarningForCipherAndReturnIfOk(const std::string &cipherName) const;

        std::shared_ptr<cpputils::Console> _console;

        DISALLOW_COPY_AND_ASSIGN(CryConfigConsole);
    };
}

#endif
