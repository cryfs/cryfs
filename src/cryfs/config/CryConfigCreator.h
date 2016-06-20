#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGCREATOR_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGCREATOR_H

#include <cpp-utils/pointer/unique_ref.h>
#include <cpp-utils/random/RandomGenerator.h>
#include <cpp-utils/io/Console.h>
#include "CryConfig.h"
#include "CryConfigConsole.h"

namespace cryfs {
    class CryConfigCreator final {
    public:
        CryConfigCreator(std::shared_ptr<cpputils::Console> console, cpputils::RandomGenerator &encryptionKeyGenerator, bool noninteractive);
        CryConfigCreator(CryConfigCreator &&rhs) = default;

        CryConfig create(const boost::optional<std::string> &cipherFromCommandLine, const boost::optional<uint32_t> &blocksizeBytesFromCommandLine);
    private:
        std::string _generateCipher(const boost::optional<std::string> &cipherFromCommandLine);
        std::string _generateEncKey(const std::string &cipher);
        std::string _generateRootBlobKey();
        uint32_t _generateBlocksizeBytes(const boost::optional<uint32_t> &blocksizeBytesFromCommandLine);
        CryConfig::FilesystemID _generateFilesystemID();

        std::shared_ptr<cpputils::Console> _console;
        CryConfigConsole _configConsole;
        cpputils::RandomGenerator &_encryptionKeyGenerator;

        DISALLOW_COPY_AND_ASSIGN(CryConfigCreator);
    };
}

#endif
