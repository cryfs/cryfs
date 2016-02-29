#include "cryfs_mount_handle.h"

using cpputils::unique_ref;
using cryfs::CryDevice;

cryfs_mount_handle::cryfs_mount_handle(unique_ref<CryDevice> crydevice)
    : _crydevice(std::move(crydevice)),
      // Copy it to make sure we have a valid pointer even if CryDevice invalidates it
      _cipher(_crydevice->config().Cipher()) {
}

const char *cryfs_mount_handle::get_ciphername() const {
    return _cipher.c_str();
}

cryfs_status cryfs_mount_handle::set_mountdir(const std::string &mountdir) {
    //TODO
    //_mountdir = mountdir;
    return cryfs_success;
}

cryfs_status cryfs_mount_handle::set_logfile(const boost::filesystem::path &logfile) {
    //TODO
    //_logfile = logfile;
    return cryfs_success;
}

cryfs_status cryfs_mount_handle::set_unmount_idle(const std::chrono::seconds timeout) {
    //TODO
    //_timeout = timeout;
    return cryfs_success;
}

cryfs_status cryfs_mount_handle::mount() {
    //TODO
    return cryfs_success;
}
