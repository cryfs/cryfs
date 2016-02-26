#include "cryfs_load_context.h"

using std::string;
using boost::none;

cryfs_load_context::cryfs_load_context()
        : _basedir(boost::none), _password(boost::none), _configfile(boost::none) {
}

cryfs_status cryfs_load_context::set_basedir(const string &basedir) {
    _basedir = basedir;
    return cryfs_success;
}

cryfs_status cryfs_load_context::set_password(const string &password) {
    _password = password;
    return cryfs_success;
}

cryfs_status cryfs_load_context::set_externalconfig(const string &configfile) {
    _configfile = configfile;
    return cryfs_success;
}

cryfs_status cryfs_load_context::load(cryfs_mount_handle **handle) {
    if (_basedir == none) {
        return cryfs_error_BASEDIR_NOT_SET;
    }
    if (_password == none) {
        return cryfs_error_PASSWORD_NOT_SET;
    }
    //TODO Actually load the file system here and pass the CryDevice instance to cryfs_mount_handle. This way, we can throw loading errors here already.
    *handle = _keepHandleOwnership.create(*_basedir, _configfile, *_password);
    return cryfs_success;
}
