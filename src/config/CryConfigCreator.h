#ifndef CRYFS_SRC_CONFIG_CRYCONFIGCREATOR_H
#define CRYFS_SRC_CONFIG_CRYCONFIGCREATOR_H

#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <messmer/cpp-utils/io/Console.h>
#include "CryConfig.h"

namespace cryfs {
    class CryConfigCreator final {
    public:
        CryConfigCreator(cpputils::unique_ref<cpputils::Console> console);

        CryConfig create();
        CryConfig createForTest();
    private:
        std::string _generateCipher();
        std::string _generateEncKey(const std::string &cipher);
        std::string _generateRootBlobKey();
        bool _showWarningForCipherAndReturnIfOk(const std::string &cipherName);

        //TODO Don't have these functions here, but use a CryConfigCreator interface and mock it in the tests
        std::string _generateEncKeyForTest(const std::string &cipher);
        std::string _generateCipherForTest();

        cpputils::unique_ref<cpputils::Console> _console;

        DISALLOW_COPY_AND_ASSIGN(CryConfigCreator);
    };
}

#endif
