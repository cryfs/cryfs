#include "Environment.h"
#include <cstdlib>
#include <cpp-utils/system/homedir.h>
#include <boost/filesystem.hpp>

using std::string;
namespace bf = boost::filesystem;

namespace cryfs_cli {
    const string Environment::FRONTEND_KEY = "CRYFS_FRONTEND";
    const string Environment::FRONTEND_NONINTERACTIVE = "noninteractive";
    const string Environment::NOUPDATECHECK_KEY = "CRYFS_NO_UPDATE_CHECK";
    const string Environment::LOCALSTATEDIR_KEY = "CRYFS_LOCAL_STATE_DIR";

    bool Environment::isNoninteractive() {
        char *frontend = std::getenv(FRONTEND_KEY.c_str());
        return frontend != nullptr && frontend == FRONTEND_NONINTERACTIVE;
    }

    bool Environment::noUpdateCheck() {
        return nullptr != std::getenv(NOUPDATECHECK_KEY.c_str());
    }

    const bf::path& Environment::defaultLocalStateDir() {
        static const bf::path value = cpputils::system::HomeDirectory::getXDGDataDir() / "cryfs";
        return value;
    }

    bf::path Environment::localStateDir() {
        const char* localStateDir = std::getenv(LOCALSTATEDIR_KEY.c_str());

        if (nullptr == localStateDir) {
            return defaultLocalStateDir();
        }

        return bf::absolute(localStateDir);
    }
}
