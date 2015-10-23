#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGFILE_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGFILE_H

#include <boost/optional.hpp>
#include <boost/filesystem/path.hpp>
#include "CryConfig.h"

namespace cryfs {
    class CryConfigFile final {
    public:
        CryConfigFile(CryConfigFile &&rhs);
        ~CryConfigFile();

        static CryConfigFile create(const boost::filesystem::path &path, CryConfig config);
        static boost::optional<CryConfigFile> load(const boost::filesystem::path &path);
        void save() const;

        CryConfig *config();

    private:
        CryConfigFile(const boost::filesystem::path &path, CryConfig config);

        boost::filesystem::path _path;
        CryConfig _config;

        DISALLOW_COPY_AND_ASSIGN(CryConfigFile);
    };
}

#endif
