#ifndef CRYFS_CRYFS_MOUNT_HANDLE_H
#define CRYFS_CRYFS_MOUNT_HANDLE_H

#include "../cryfs.h"
#include "../impl/filesystem/CryDevice.h"

struct cryfs_mount_handle final {
public:
    cryfs_mount_handle(cpputils::unique_ref<cryfs::CryDevice> crydevice);

    const char *get_ciphername() const;
    cryfs_status set_mountdir(const std::string &mountdir);
    cryfs_status set_logfile(const boost::filesystem::path &logfile);
    cryfs_status set_unmount_idle(const std::chrono::seconds timeout);

    cryfs_status mount();

private:
    cpputils::unique_ref<cryfs::CryDevice> _crydevice;

    DISALLOW_COPY_AND_ASSIGN(cryfs_mount_handle);
};

#endif
