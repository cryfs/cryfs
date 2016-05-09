#include "cryfs_mount_handle.h"

using cpputils::unique_ref;
using cryfs::CryDevice;
using boost::none;
using std::string;
namespace bf = boost::filesystem;

cryfs_mount_handle::cryfs_mount_handle(unique_ref<CryDevice> crydevice)
    : _crydevice(std::move(crydevice)),
      // Copy it to make sure we have a valid pointer even if CryDevice invalidates it
      _cipher(_crydevice->config().Cipher()),
      _mountdir(none),
      _unmount_idle(none),
      _run_in_foreground(false),
      _fuse_arguments() {
}

const char *cryfs_mount_handle::get_ciphername() const {
    return _cipher.c_str();
}

cryfs_status cryfs_mount_handle::set_mountdir(const string &mountdir) {
    if (!bf::is_directory(mountdir)) {
        return cryfs_error_MOUNTDIR_DOESNT_EXIST;
    }
    //TODO Handle (and add test cases for) missing permissions
    _mountdir = mountdir;
    return cryfs_success;
}

cryfs_status cryfs_mount_handle::set_run_in_foreground(bool run_in_foreground) {
    _run_in_foreground = run_in_foreground;
    return cryfs_success;
}

cryfs_status cryfs_mount_handle::set_logfile(const bf::path &logfile) {
    if (!bf::is_directory(logfile.parent_path())) {
        return cryfs_error_INVALID_LOGFILE;
    }
    //TODO Handle (and add test cases for) missing write permissions (or create file permissions)
    _logfile = logfile;
    return cryfs_success;
}

cryfs_status cryfs_mount_handle::set_unmount_idle(const std::chrono::seconds unmount_idle) {
    _unmount_idle = unmount_idle;
    return cryfs_success;
}

cryfs_status cryfs_mount_handle::add_fuse_argument(const string &argument) {
    _fuse_arguments.push_back(argument);
    return cryfs_success;
}

cryfs_status cryfs_mount_handle::mount() {
    if (_mountdir == none) {
        return cryfs_error_MOUNTDIR_NOT_SET;
    }
    //TODO
    return cryfs_success;
}
