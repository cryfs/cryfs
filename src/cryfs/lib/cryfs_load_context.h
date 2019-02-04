#pragma once
#ifndef CRYFS_CRYFS_LOAD_CONTEXT_H
#define CRYFS_CRYFS_LOAD_CONTEXT_H

#include "../cryfs.h"
#include <string>
#include <boost/optional.hpp>
#include <boost/filesystem/path.hpp>
#include "../impl/config/CryConfigFile.h"
#include "../impl/filesystem/CryDevice.h"
#include "context_list.h"

struct cryfs_load_context final {
public:
    cryfs_load_context(cryfs_api_context *api_context);

    cryfs_status set_basedir(const boost::filesystem::path& basedir);
    cryfs_status set_password(std::string password);
    cryfs_status set_externalconfig(const boost::filesystem::path& configfile);

    // TODO Test set_localstatedir
    cryfs_status set_localstatedir(const boost::filesystem::path& localstatedir);
    
    cryfs_status load(cryfs_mount_handle **handle);

    cryfs_status free();

private:
    cryfs_api_context *_api_context;

    boost::optional<boost::filesystem::path> _basedir;
    boost::optional<std::string> _password;
    boost::optional<boost::filesystem::path> _configfile;
    boost::optional<boost::filesystem::path> _localstatedir;
    cryfs::context_list<cryfs_mount_handle> _mount_handles;

    cpputils::either<cryfs::CryConfigFile::LoadError, cpputils::unique_ref<cryfs::CryConfigFile>> _load_configfile() const;
    boost::filesystem::path _determine_configfile_path() const;
    static bool _check_version(const cryfs::CryConfig &config);
    static bool _sanity_check_filesystem(cryfs::CryDevice *device);

    DISALLOW_COPY_AND_ASSIGN(cryfs_load_context);
};

#endif
