#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGCREATOR_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGCREATOR_H

#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <messmer/cpp-utils/random/RandomGenerator.h>
#include <messmer/cpp-utils/io/Console.h>
#include "CryConfig.h"

namespace cryfs {
    class CryConfigCreator final {
    public:
        CryConfigCreator(cpputils::unique_ref<cpputils::Console> console, cpputils::RandomGenerator &encryptionKeyGenerator);

        CryConfig create();
    private:
        std::string _generateCipher();
        std::string _generateEncKey(const std::string &cipher);
        std::string _generateRootBlobKey();
        bool _showWarningForCipherAndReturnIfOk(const std::string &cipherName);

        cpputils::unique_ref<cpputils::Console> _console;
        cpputils::RandomGenerator &_encryptionKeyGenerator;

        DISALLOW_COPY_AND_ASSIGN(CryConfigCreator);
    };
}

#endif
