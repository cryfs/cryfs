#include "cryfs_mount_handle.h"

using cpputils::unique_ref;
using cryfs::CryDevice;
using boost::none;
namespace bf = boost::filesystem;

cryfs_mount_handle::cryfs_mount_handle(unique_ref<CryDevice> crydevice)
    : _crydevice(std::move(crydevice)),
      // Copy it to make sure we have a valid pointer even if CryDevice invalidates it
      _cipher(_crydevice->config().Cipher()),
      _mountdir(none) {
}

const char *cryfs_mount_handle::get_ciphername() const {
    return _cipher.c_str();
}

cryfs_status cryfs_mount_handle::set_mountdir(const std::string &mountdir) {
    if (!bf::is_directory(mountdir)) {
        return cryfs_error_MOUNTDIR_DOESNT_EXIST;
    }
    //TODO Handle (and add test cases for) missing permissions
    _mountdir = mountdir;
    return cryfs_success;
}

cryfs_status cryfs_mount_handle::set_logfile(const boost::filesystem::path &logfile) {
    if (!bf::is_directory(logfile.parent_path())) {
        return cryfs_error_INVALID_LOGFILE;
    }
    //TODO Handle (and add test cases for) missing write permissions (or create file permissions)
    _logfile = logfile;
    return cryfs_success;
}

cryfs_status cryfs_mount_handle::set_unmount_idle(const std::chrono::seconds timeout) {
    //TODO
    //_timeout = timeout;
    return cryfs_success;
}

cryfs_status cryfs_mount_handle::mount() {
    if (_mountdir == none) {
        return cryfs_error_MOUNTDIR_NOT_SET;
    }
    //TODO
    return cryfs_success;
}
