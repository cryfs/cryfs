#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGCONSOLE_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGCONSOLE_H

#include <cpp-utils/pointer/unique_ref.h>
#include <cpp-utils/io/Console.h>
#include <boost/optional.hpp>

namespace cryfs {
    class CryConfigConsole final {
    public:
        CryConfigConsole(std::shared_ptr<cpputils::Console> console);
        CryConfigConsole(CryConfigConsole &&rhs) = default;

        std::string askCipher();
        uint32_t askBlocksizeBytes();
        bool askMissingBlockIsIntegrityViolation();

        static constexpr const char *DEFAULT_CIPHER = "xchacha20-poly1305";
        static constexpr uint32_t DEFAULT_BLOCKSIZE_BYTES = 16 * 1024; // 16KB
        static constexpr uint32_t DEFAULT_MISSINGBLOCKISINTEGRITYVIOLATION = false;

    private:

        bool _checkUseDefaultSettings();

        std::string _askCipher() const;
        bool _showWarningForCipherAndReturnIfOk(const std::string &cipherName) const;
        uint32_t _askBlocksizeBytes() const;
        bool _askMissingBlockIsIntegrityViolation() const;

        std::shared_ptr<cpputils::Console> _console;
        boost::optional<bool> _useDefaultSettings;

        DISALLOW_COPY_AND_ASSIGN(CryConfigConsole);
    };
}

#endif
