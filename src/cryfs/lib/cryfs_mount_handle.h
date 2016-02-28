#ifndef CRYFS_CRYFS_MOUNT_HANDLE_H
#define CRYFS_CRYFS_MOUNT_HANDLE_H

#include "../cryfs.h"
#include <string>
#include <boost/optional.hpp>
#include "../impl/config/CryConfigFile.h"

struct cryfs_mount_handle final {
public:
    cryfs_mount_handle(const boost::filesystem::path &basedir, cryfs::CryConfigFile config);

private:
    const boost::filesystem::path _basedir;
    cryfs::CryConfigFile _config;

    DISALLOW_COPY_AND_ASSIGN(cryfs_mount_handle);
};

#endif
