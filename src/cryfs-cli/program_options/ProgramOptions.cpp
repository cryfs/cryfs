#include "ProgramOptions.h"
#include <cstring>
#include <cpp-utils/assert/assert.h>

using namespace cryfs::program_options;
using std::string;
using std::vector;
using boost::optional;
namespace bf = boost::filesystem;

ProgramOptions::ProgramOptions(const bf::path &baseDir, const bf::path &mountDir, const optional<bf::path> &configFile,
                               bool foreground, const optional<double> &unmountAfterIdleMinutes,
                               const optional<bf::path> &logFile, const optional<string> &cipher,
                               const optional<uint32_t> &blocksizeBytes,
                               const vector<string> &fuseOptions)
    :_baseDir(baseDir), _mountDir(mountDir), _configFile(configFile), _foreground(foreground),
     _cipher(cipher), _blocksizeBytes(blocksizeBytes), _unmountAfterIdleMinutes(unmountAfterIdleMinutes),
     _logFile(logFile), _fuseOptions(fuseOptions) {

    auto hasNoOption = [&](const char *opt) {
        for (const string& it : _fuseOptions) {
            if (std::strncmp(it.c_str(), opt, std::strlen(opt))) {
                return false;
            }
        }
        return true;
    };

    if (hasNoOption("subtype=cryfs") && hasNoOption("fsname=cryfs@")) {
               _fuseOptions.push_back("-ofsname=cryfs@"+baseDir.native());
               _fuseOptions.push_back("-osubtype=cryfs");
    }
}

const bf::path &ProgramOptions::baseDir() const {
    return _baseDir;
}

const bf::path &ProgramOptions::mountDir() const {
    return _mountDir;
}

const optional<bf::path> &ProgramOptions::configFile() const {
    return _configFile;
}

bool ProgramOptions::foreground() const {
    return _foreground;
}

const optional<double> &ProgramOptions::unmountAfterIdleMinutes() const {
    return _unmountAfterIdleMinutes;
}

const optional<bf::path> &ProgramOptions::logFile() const {
    return _logFile;
}

const optional<string> &ProgramOptions::cipher() const {
    return _cipher;
}

const optional<uint32_t> &ProgramOptions::blocksizeBytes() const {
    return _blocksizeBytes;
}

const vector<string> &ProgramOptions::fuseOptions() const {
    return _fuseOptions;
}
