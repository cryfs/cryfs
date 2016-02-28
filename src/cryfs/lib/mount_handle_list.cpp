#include "mount_handle_list.h"

using cpputils::make_unique_ref;
using cryfs::CryConfigFile;
namespace bf = boost::filesystem;

mount_handle_list::mount_handle_list()
        :_createdHandles() {
}

cryfs_mount_handle *mount_handle_list::create(const bf::path &basedir, CryConfigFile config) {
    auto handle = make_unique_ref<cryfs_mount_handle>(basedir, std::move(config));
    cryfs_mount_handle *result = handle.get();
    _createdHandles.push_back(std::move(handle));
    return result;
}
