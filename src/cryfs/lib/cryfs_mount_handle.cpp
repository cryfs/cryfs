#include "cryfs_mount_handle.h"

using boost::optional;
using std::string;

cryfs_mount_handle::cryfs_mount_handle(const string &basedir, const optional<string> &configFile, const string &password)
    :_basedir(basedir), _configFile(configFile), _password(password) {
}

