#pragma once
#ifndef MESSMER_CRYFS_LOCALSTATE_LOCALSTATEDIR_H_
#define MESSMER_CRYFS_LOCALSTATE_LOCALSTATEDIR_H_

#include <cpp-utils/macros.h>
#include <boost/filesystem/path.hpp>
#include "../config/CryConfig.h"

namespace cryfs {

    class LocalStateDir final {
    public:
        LocalStateDir(boost::filesystem::path appDir);

        boost::filesystem::path forFilesystemId(const CryConfig::FilesystemID &filesystemId) const;
        boost::filesystem::path forBasedirMetadata() const;

    private:
        boost::filesystem::path _appDir;

        static void _createDirIfNotExists(const boost::filesystem::path &path);
    };
}


#endif
