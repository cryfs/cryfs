#pragma once
#ifndef CRYFS_MOUNT_HANDLE_LIST_H
#define CRYFS_MOUNT_HANDLE_LIST_H

#include <cpp-utils/pointer/unique_ref.h>
#include "cryfs_mount_handle.h"

// This class keeps ownership of created mount handles and destroys them in its destructor.
class mount_handle_list final {
public:
    mount_handle_list();

    cryfs_mount_handle *create(const std::string &basedir, const boost::optional<std::string> &configFile, const std::string &password);

private:
    std::vector<cpputils::unique_ref<cryfs_mount_handle>> _createdHandles;

    DISALLOW_COPY_AND_ASSIGN(mount_handle_list);
};

#endif
