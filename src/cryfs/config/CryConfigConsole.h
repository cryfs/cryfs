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
        uint32_t askBlocksizeBytes();

        static constexpr const char *DEFAULT_CIPHER = "aes-256-gcm";
        static constexpr uint32_t DEFAULT_BLOCKSIZE_BYTES = 32 * 1024; // 32KB

    private:

        bool _checkUseDefaultSettings();

        std::string _askCipher() const;
        bool _showWarningForCipherAndReturnIfOk(const std::string &cipherName) const;
        uint32_t _askBlocksizeBytes() const;

        std::shared_ptr<cpputils::Console> _console;
        boost::optional<bool> _useDefaultSettings;

        DISALLOW_COPY_AND_ASSIGN(CryConfigConsole);
    };
}

#endif
