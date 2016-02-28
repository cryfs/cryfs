#include "mount_handle_list.h"

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cryfs::CryDevice;

mount_handle_list::mount_handle_list()
        :_createdHandles() {
}

cryfs_mount_handle *mount_handle_list::create(unique_ref<CryDevice> crydevice) {
    auto handle = make_unique_ref<cryfs_mount_handle>(std::move(crydevice));
    cryfs_mount_handle *result = handle.get();
    _createdHandles.push_back(std::move(handle));
    return result;
}
