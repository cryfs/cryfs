#include "mount_handle_list.h"

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cryfs::CryDevice;
using cryfs::CryConfigFile;
namespace bf = boost::filesystem;

mount_handle_list::mount_handle_list()
        :_createdHandles() {
}

cryfs_mount_handle *mount_handle_list::create(std::shared_ptr<CryConfigFile> config, const bf::path &basedir) {
    auto handle = make_unique_ref<cryfs_mount_handle>(config, basedir);
    cryfs_mount_handle *result = handle.get();
    _createdHandles.push_back(std::move(handle));
    return result;
}
