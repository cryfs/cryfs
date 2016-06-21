#include "homedir.h"
#include <sys/types.h>
#include <pwd.h>

namespace bf = boost::filesystem;
using std::string;

namespace cpputils {
    namespace system {
        bf::path home_directory() {
            struct passwd* pwd = getpwuid(getuid());
            string homedir;
            if (pwd) {
                homedir = pwd->pw_dir;
            } else {
                // try the $HOME environment variable
                homedir = getenv("HOME");
            }
            if (homedir == "") {
                throw std::runtime_error("Couldn't determine home directory for user");
            }
            return homedir;
        }
    }
}