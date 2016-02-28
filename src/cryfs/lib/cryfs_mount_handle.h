#ifndef CRYFS_CRYFS_MOUNT_HANDLE_H
#define CRYFS_CRYFS_MOUNT_HANDLE_H

#include "../cryfs.h"
#include "../impl/filesystem/CryDevice.h"

struct cryfs_mount_handle final {
public:
    cryfs_mount_handle(cpputils::unique_ref<cryfs::CryDevice> crydevice);

private:
    cpputils::unique_ref<cryfs::CryDevice> _crydevice;

    DISALLOW_COPY_AND_ASSIGN(cryfs_mount_handle);
};

#endif
