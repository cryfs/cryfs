#pragma once
#ifndef MESSMER_CRYFS_LOCALSTATE_LOCALSTATEDIR_H_
#define MESSMER_CRYFS_LOCALSTATE_LOCALSTATEDIR_H_

#include <cpp-utils/macros.h>
#include <boost/filesystem/path.hpp>
#include "../config/CryConfig.h"

namespace cryfs {

    class LocalStateDir final {
    public:
        static boost::filesystem::path forFilesystemId(const CryConfig::FilesystemID &filesystemId);
        static boost::filesystem::path forMapFromBasedirToConfigFiles();

        // Use this from test cases to not pollute local config
        // TODO Make test cases call this
        static void setAppDir(boost::filesystem::path path);

    private:
        LocalStateDir(); // static functions only

        static void _createDirIfNotExists(const boost::filesystem::path &path);
    };
}


#endif
