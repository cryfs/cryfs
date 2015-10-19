#include "CryConfigCreator.h"
#include "CryCipher.h"
#include <messmer/blockstore/implementations/encrypted/ciphers/ciphers.h>

using cpputils::Console;
using cpputils::unique_ref;
using std::string;
using std::vector;

namespace cryfs {

    CryConfigCreator::CryConfigCreator(unique_ref<Console> console)
        :_console(std::move(console)) {
    }

    CryConfig CryConfigCreator::create() {
        CryConfig config;
        config.SetCipher(_generateCipher());
        config.SetEncryptionKey(_generateEncKey(config.Cipher()));
        config.SetRootBlob(_generateRootBlobKey());
        return config;
    }

    CryConfig CryConfigCreator::createForTest() {
        CryConfig config;
        config.SetCipher(_generateCipherForTest());
        config.SetEncryptionKey(_generateEncKeyForTest(config.Cipher()));
        config.SetRootBlob(_generateRootBlobKey());
        return config;
    }

    string CryConfigCreator::_generateCipher() {
        vector<string> ciphers = CryCiphers::supportedCipherNames();
        string cipherName = "";
        bool askAgain = true;
        while(askAgain) {
            int cipherIndex = _console->ask("Which block cipher do you want to use?", ciphers);
            cipherName = ciphers[cipherIndex];
            askAgain = !_showWarningForCipherAndReturnIfOk(cipherName);
        };
        return cipherName;
    }

    bool CryConfigCreator::_showWarningForCipherAndReturnIfOk(const string &cipherName) {
        auto warning = CryCiphers::find(cipherName).warning();
        if (warning == boost::none) {
            return true;
        }
        return _console->askYesNo(string() + (*warning) + " Do you want to take this cipher nevertheless?");
    }

    string CryConfigCreator::_generateEncKey(const std::string &cipher) {
        _console->print("\nGenerating secure encryption key...");
        auto key = CryCiphers::find(cipher).createKey();
        _console->print("done\n");
        return key;
    }

    string CryConfigCreator::_generateCipherForTest() {
        return "aes-256-gcm";
    }

    string CryConfigCreator::_generateEncKeyForTest(const std::string &) {
        return blockstore::encrypted::AES256_GCM::EncryptionKey::CreatePseudoRandom().ToString();
    }

    string CryConfigCreator::_generateRootBlobKey() {
        //An empty root blob entry will tell CryDevice to create a new root blob
        return "";
    }

}