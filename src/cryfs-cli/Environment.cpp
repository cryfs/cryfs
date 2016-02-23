#include "Environment.h"
#include <cstdlib>

using std::string;

namespace cryfs {
    const string Environment::FRONTEND_KEY = "CRYFS_FRONTEND";
    const string Environment::FRONTEND_NONINTERACTIVE = "noninteractive";
    const string Environment::NOUPDATECHECK_KEY = "CRYFS_NO_UPDATE_CHECK";

    bool Environment::isNoninteractive() {
        char *frontend = std::getenv(FRONTEND_KEY.c_str());
        return frontend != nullptr && frontend == FRONTEND_NONINTERACTIVE;
    }

    bool Environment::noUpdateCheck() {
        return nullptr != std::getenv(NOUPDATECHECK_KEY.c_str());
    }
}
