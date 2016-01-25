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

    bool CryConfigConsole::_checkUseDefaultSettings() {
        if (_useDefaultSettings == none) {
            _useDefaultSettings = _console->askYesNo("Use default settings?");
        }
        return *_useDefaultSettings;
    }
}
