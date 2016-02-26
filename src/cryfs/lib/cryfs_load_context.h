#ifndef CRYFS_CRYFS_LOAD_CONTEXT_H
#define CRYFS_CRYFS_LOAD_CONTEXT_H

#include "../cryfs.h"
#include <boost/optional.hpp>
#include <cpp-utils/macros.h>
#include "mount_handle_list.h"

struct cryfs_load_context final {
public:
    cryfs_load_context();

    cryfs_status set_basedir(const char *basedir);

    cryfs_status set_password(const char *password, size_t password_length);

    cryfs_status set_externalconfig(const char *configfile);

    cryfs_status load(cryfs_mount_handle **handle);

private:
    boost::optional<std::string> _basedir;
    boost::optional<std::string> _password;
    boost::optional<std::string> _configfile;

    mount_handle_list _keepHandleOwnership;

    DISALLOW_COPY_AND_ASSIGN(cryfs_load_context);
};

#endif
