#ifndef CRYFS_CRYFS_MOUNT_HANDLE_H
#define CRYFS_CRYFS_MOUNT_HANDLE_H

#include "../cryfs.h"
#include <cpp-utils/macros.h>
#include <string>
#include <boost/optional.hpp>

struct cryfs_mount_handle final {
public:
    cryfs_mount_handle(const std::string &basedir, const boost::optional<std::string> &configFile, const std::string &password);

private:
    const std::string &_basedir;
    const boost::optional<std::string> &_configFile;
    const std::string &_password;

    DISALLOW_COPY_AND_ASSIGN(cryfs_mount_handle);
};

#endif
