#include "cryfs_mount_handle.h"

using cpputils::unique_ref;
using cryfs::CryDevice;

cryfs_mount_handle::cryfs_mount_handle(unique_ref<CryDevice> crydevice)
    :_crydevice(std::move(crydevice)) {
}

