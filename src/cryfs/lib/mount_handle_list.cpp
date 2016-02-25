#include "mount_handle_list.h"

using cpputils::make_unique_ref;

mount_handle_list::mount_handle_list()
        :_createdHandles() {
}

cryfs_mount_handle *mount_handle_list::create(const std::string &basedir, const boost::optional<std::string> &configFile, const std::string &password) {
    auto handle = make_unique_ref<cryfs_mount_handle>(basedir, configFile, password);
    cryfs_mount_handle *result = handle.get();
    _createdHandles.push_back(std::move(handle));
    return result;
}
