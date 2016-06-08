#include <fspp/impl/FilesystemImpl.h>
#include <fspp/fuse/Fuse.h>
#include <cpp-utils/process/daemon/daemonize.h>
#include "cryfs_mount_handle.h"

using cpputils::unique_ref;
using cryfs::CryDevice;
using boost::none;
using std::string;
using std::vector;
namespace bf = boost::filesystem;

cryfs_mount_handle::cryfs_mount_handle(unique_ref<CryDevice> crydevice, const bf::path &basedir)
    : _crydevice(cpputils::to_unique_ptr(std::move(crydevice))),
      // Copy it to make sure we have a valid pointer even if CryDevice invalidates it
      _cipher(_crydevice->config().Cipher()),
      _basedir(basedir),
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

    if (nullptr == _crydevice) {
        return cryfs_error_MOUNTHANDLE_ALREADY_USED;
    }

    _init_logfile();

    fspp::FilesystemImpl fsimpl(_crydevice.get());
    fspp::fuse::Fuse fuse(&fsimpl, "cryfs", "cryfs@"+_basedir.native());

    //TODO Test auto unmounting after idle timeout
    //TODO This can fail due to a race condition if the filesystem isn't started yet (e.g. passing --unmount-idle 0").
    /*auto idleUnmounter = _createIdleCallback(options.unmountAfterIdleMinutes(), [&fuse] {fuse.stop();});
    if (idleUnmounter != none) {
        device.onFsAction(std::bind(&CallAfterTimeout::resetTimer, idleUnmounter->get()));
    }*/

    if (_run_in_foreground) {
        fuse.runInForeground(*_mountdir, _fuse_arguments);
    } else {
        fuse.runInBackground(*_mountdir, _fuse_arguments);
    }
    
    _crydevice = nullptr; // Free CryDevice in this process

    return cryfs_success;
}

void cryfs_mount_handle::_init_logfile() {
    spdlog::drop("cryfs");
    if (_logfile != none) {
        cpputils::logging::setLogger(
                spdlog::create<spdlog::sinks::simple_file_sink<std::mutex>>("cryfs", _logfile->native()));
    } else if (_run_in_foreground) {
        cpputils::logging::setLogger(spdlog::stderr_logger_mt("cryfs"));
    } else {
        cpputils::logging::setLogger(spdlog::syslog_logger("cryfs", "cryfs", LOG_PID));
    }
}
