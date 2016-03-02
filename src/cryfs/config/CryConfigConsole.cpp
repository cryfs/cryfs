#include "CryConfigConsole.h"
#include "CryCipher.h"

using cpputils::unique_ref;
using cpputils::Console;
using boost::optional;
using boost::none;
using std::string;
using std::vector;
using std::shared_ptr;

namespace cryfs {
    constexpr const char *CryConfigConsole::DEFAULT_CIPHER;
    constexpr uint32_t CryConfigConsole::DEFAULT_BLOCKSIZE_BYTES;

    CryConfigConsole::CryConfigConsole(shared_ptr<Console> console, bool noninteractive)
            : _console(std::move(console)), _useDefaultSettings(noninteractive ? optional<bool>(true) : none) {
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
            int cipherIndex = _console->ask("Which block cipher do you want to use?", ciphers);
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
        return _console->askYesNo(string() + (*warning) + " Do you want to take this cipher nevertheless?");
    }

    uint32_t CryConfigConsole::askBlocksizeBytes() {
        if (_checkUseDefaultSettings()) {
            return DEFAULT_BLOCKSIZE_BYTES;
        } else {
            return _askBlocksizeBytes();
        }
    }

    uint32_t CryConfigConsole::_askBlocksizeBytes() const {
        vector<string> sizes = {"8KB", "32KB", "64KB", "512KB", "1MB", "4MB"};
        int index = _console->ask("Which block size do you want to use?", sizes);
        switch(index) {
            case 0: return 8*1024;
            case 1: return 32*1024;
            case 2: return 64*1024;
            case 3: return 512*1024;
            case 4: return 1024*1024;
            case 5: return 4*1024*1024;
            default: ASSERT(false, "Unhandled case");
        }
    }

    bool CryConfigConsole::_checkUseDefaultSettings() {
        if (_useDefaultSettings == none) {
            _useDefaultSettings = _console->askYesNo("Use default settings?");
        }
        return *_useDefaultSettings;
    }
}
