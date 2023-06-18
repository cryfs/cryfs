#include "CryConfigConsole.h"
#include "CryCipher.h"

using cpputils::Console;
using boost::none;
using std::string;
using std::vector;
using std::shared_ptr;

namespace cryfs {
    constexpr const char *CryConfigConsole::DEFAULT_CIPHER;
    constexpr uint32_t CryConfigConsole::DEFAULT_BLOCKSIZE_BYTES;

    CryConfigConsole::CryConfigConsole(shared_ptr<Console> console)
            : _console(std::move(console)), _useDefaultSettings(none) {
    }

    string CryConfigConsole::askCipher() {
        if (_checkUseDefaultSettings()) {
            return DEFAULT_CIPHER;
        } else {
            return _askCipher();
        }
    }

    string CryConfigConsole::_askCipher() const {
        vector<string> ciphers = CryCiphers::supportedCipherNames();
        string cipherName = "";
        bool askAgain = true;
        while(askAgain) {
            _console->print("\n");
            unsigned int cipherIndex = _console->ask("Which block cipher do you want to use?", ciphers);
            cipherName = ciphers[cipherIndex];
            askAgain = !_showWarningForCipherAndReturnIfOk(cipherName);
        };
        return cipherName;
    }

    bool CryConfigConsole::_showWarningForCipherAndReturnIfOk(const string &cipherName) const {
        auto warning = CryCiphers::find(cipherName).warning();
        if (warning == none) {
            return true;
        }
        return _console->askYesNo(string() + (*warning) + " Do you want to take this cipher nevertheless?", true);
    }

    uint32_t CryConfigConsole::askBlocksizeBytes() {
        if (_checkUseDefaultSettings()) {
            return DEFAULT_BLOCKSIZE_BYTES;
        } else {
            return _askBlocksizeBytes();
        }
    }

    uint32_t CryConfigConsole::_askBlocksizeBytes() const {
        vector<string> sizes = {"4KB", "8KB", "16KB", "32KB", "64KB", "512KB", "1MB", "4MB"};
        unsigned int index = _console->ask("Which block size do you want to use?", sizes);
        switch(index) {
            case 0: return 4*1024;
            case 1: return 8*1024;
            case 2: return 16*1024;
            case 3: return 32*1024;
            case 4: return 64*1024;
            case 5: return 512*1024;
            case 6: return 1024*1024;
            case 7: return 4*1024*1024;
            default: ASSERT(false, "Unhandled case");
        }
    }

    bool CryConfigConsole::askMissingBlockIsIntegrityViolation() {
        if (_checkUseDefaultSettings()) {
            return DEFAULT_MISSINGBLOCKISINTEGRITYVIOLATION;
        } else {
            return _askMissingBlockIsIntegrityViolation();
        }
    }

    bool CryConfigConsole::_askMissingBlockIsIntegrityViolation() const {
        return _console->askYesNo("\nMost integrity checks are enabled by default. However, by default CryFS does not treat missing blocks as integrity violations.\nThat is, if CryFS finds a block missing, it will assume that this is due to a synchronization delay and not because an attacker deleted the block.\nIf you are in a single-client setting, you can let it treat missing blocks as integrity violations, which will ensure that you notice if an attacker deletes one of your files.\nHowever, in this case, you will not be able to use the file system with other devices anymore.\nDo you want to treat missing blocks as integrity violations?", false);
    }

    bool CryConfigConsole::_checkUseDefaultSettings() {
        if (_useDefaultSettings == none) {
            _useDefaultSettings = _console->askYesNo("Use default settings?", true);
        }
        return *_useDefaultSettings;
    }
}
