#include "cryfs_unmounter.h"

namespace bf = boost::filesystem;

namespace cryfs {

cryfs_status cryfs_unmounter::unmount(const bf::path &mountdir) {
#ifdef __APPLE__
    int ret = system(("umount " + mountdir.native()).c_str());
#else
    int ret = system(("fusermount -z -u " + mountdir.native()).c_str()); // "-z" takes care that if the filesystem can't be unmounted right now because something is opened, it will be unmounted as soon as it can be.
#endif
    if (ret != 0) {
        return cryfs_error_UNMOUNT_FAILED;
    } else {
        return cryfs_success;
    }
}

}
