#ifndef CRYFS_CRYFS_LOAD_CONTEXT_H
#define CRYFS_CRYFS_LOAD_CONTEXT_H

#include "../cryfs.h"
#include <string>
#include <boost/optional.hpp>
#include <boost/filesystem/path.hpp>
#include "../impl/config/CryConfigFile.h"
#include "mount_handle_list.h"

struct cryfs_load_context final {
public:
    cryfs_load_context();

    cryfs_status set_basedir(const std::string &basedir);

    cryfs_status set_password(const std::string &password);

    cryfs_status set_externalconfig(const std::string &configfile);

    cryfs_status load(cryfs_mount_handle **handle);

private:
    boost::optional<boost::filesystem::path> _basedir;
    boost::optional<std::string> _password;
    boost::optional<boost::filesystem::path> _configfile;

    boost::optional<cryfs::CryConfigFile> _load_config_file() const;
    boost::filesystem::path _determine_configfile_path() const;
    static bool _check_version(const cryfs::CryConfig &config);

    mount_handle_list _keepHandleOwnership;

    DISALLOW_COPY_AND_ASSIGN(cryfs_load_context);
};

#endif
