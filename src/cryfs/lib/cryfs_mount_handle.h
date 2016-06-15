#pragma once
#ifndef CRYFS_CRYFS_MOUNT_HANDLE_H
#define CRYFS_CRYFS_MOUNT_HANDLE_H

#include "../cryfs.h"
#include "../impl/filesystem/CryDevice.h"
#include <fspp/impl/FilesystemImpl.h>

struct cryfs_mount_handle final {
public:
    cryfs_mount_handle(std::shared_ptr<cryfs::CryConfigFile> config, const boost::filesystem::path &basedir);

    const char *get_ciphername() const;
    cryfs_status set_mountdir(const std::string &mountdir);
    cryfs_status set_run_in_foreground(bool foreground);
    cryfs_status set_logfile(const boost::filesystem::path &logfile);
    cryfs_status set_unmount_idle(const std::chrono::seconds timeout);
    cryfs_status add_fuse_argument(const std::string &argument);

    cryfs_status mount();

private:
    void _init_logfile();
    std::shared_ptr<fspp::FilesystemImpl> _init_filesystem();

    std::shared_ptr<cryfs::CryConfigFile> _config;
    boost::filesystem::path _basedir;
    boost::optional<boost::filesystem::path> _mountdir;
    boost::optional<boost::filesystem::path> _logfile;
    boost::optional<std::chrono::seconds> _unmount_idle;
    bool _run_in_foreground;
    std::vector<std::string> _fuse_arguments;

    DISALLOW_COPY_AND_ASSIGN(cryfs_mount_handle);
};

#endif
