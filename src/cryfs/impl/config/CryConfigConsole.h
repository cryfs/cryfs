#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGCONSOLE_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGCONSOLE_H

#include <cpp-utils/pointer/unique_ref.h>
#include <cpp-utils/io/Console.h>
#include <boost/optional.hpp>

namespace cryfs {
    class CryConfigConsole final {
    public:
        CryConfigConsole(std::shared_ptr<cpputils::Console> console, bool noninteractive);
        CryConfigConsole(CryConfigConsole &&rhs) = default;

        std::string askCipher();

        static constexpr const char *DEFAULT_CIPHER = "aes-256-gcm";

    private:

        bool _checkUseDefaultSettings();

        std::string _askCipher() const;
        bool _showWarningForCipherAndReturnIfOk(const std::string &cipherName) const;

        std::shared_ptr<cpputils::Console> _console;
        boost::optional<bool> _useDefaultSettings;

        DISALLOW_COPY_AND_ASSIGN(CryConfigConsole);
    };
}

#endif
