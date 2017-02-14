#include <fspp/impl/FilesystemImpl.h>
#include <fspp/fuse/Fuse.h>
#include <cpp-utils/process/daemon/daemonize.h>
#include "cryfs_mount_handle.h"
#include <blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include "utils/filesystem_checks.h"

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cryfs::CryDevice;
using cryfs::CryConfigFile;
using boost::none;
using std::string;
using std::vector;
using std::shared_ptr;
using std::make_shared;
using blockstore::ondisk::OnDiskBlockStore;
namespace bf = boost::filesystem;

cryfs_mount_handle::cryfs_mount_handle(shared_ptr<CryConfigFile> config, const bf::path &basedir)
    : _config(config),
      _basedir(basedir),
      _mountdir(none),
      _unmount_idle(none),
      _run_in_foreground(false),
      _fuse_arguments(),
      _idle_unmounter(none) {
}

const char *cryfs_mount_handle::get_ciphername() const {
    return _config->config()->Cipher().c_str();
}

cryfs_status cryfs_mount_handle::set_mountdir(const string &mountdir) {
    if (!bf::exists(mountdir)) {
        return cryfs_error_MOUNTDIR_DOESNT_EXIST;
    }
    if (!filesystem_checks::check_dir_accessible(mountdir)) {
        return cryfs_error_MOUNTDIR_INACCESSIBLE;
    }
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
    if (bf::exists(logfile) && !filesystem_checks::check_file_appendable(logfile)) {
        return cryfs_error_LOGFILE_NOT_WRITABLE;
    }
    _logfile = logfile;
    return cryfs_success;
}

cryfs_status cryfs_mount_handle::set_unmount_idle(const boost::chrono::milliseconds unmount_idle) {
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

    fspp::fuse::Fuse fuse(std::bind(&cryfs_mount_handle::_init_filesystem, this, std::placeholders::_1), "cryfs", "cryfs@"+_basedir.native());

    if (_run_in_foreground) {
        fuse.runInForeground(*_mountdir, _fuse_arguments);
    } else {
        fuse.runInBackground(*_mountdir, _fuse_arguments);
    }

    return cryfs_success;
}

shared_ptr<fspp::FilesystemImpl> cryfs_mount_handle::_init_filesystem(fspp::fuse::Fuse *fuse) {
    _init_logfile();

    auto blockstore = make_unique_ref<OnDiskBlockStore>(_basedir);
    auto crydevice = make_unique_ref<CryDevice>(_config, std::move(blockstore));

    _create_idle_unmounter(fuse, crydevice.get());

    return make_shared<fspp::FilesystemImpl>(std::move(crydevice));
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

void cryfs_mount_handle::_create_idle_unmounter(fspp::fuse::Fuse *fuse, cryfs::CryDevice *device) {
    if (_unmount_idle == none) {
        return; // Idle unmounter not requested by user
    }

    ASSERT(_idle_unmounter == none, "Tried to create two idle unmounters");

    _idle_unmounter = make_unique_ref<CallAfterTimeout>(*_unmount_idle, [fuse] {
        fuse->stop();
    });
    device->onFsAction(std::bind(&CallAfterTimeout::resetTimer, _idle_unmounter->get()));
}
