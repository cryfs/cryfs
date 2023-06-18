#pragma once
#ifndef MESSMER_CRYFSCLI_ENVIRONMENT_H
#define MESSMER_CRYFSCLI_ENVIRONMENT_H

#include <string>
#include <boost/filesystem/path.hpp>

namespace cryfs_cli {

    class Environment {
    public:
        static bool isNoninteractive();
        static bool noUpdateCheck();
        static boost::filesystem::path localStateDir();
        static const boost::filesystem::path& defaultLocalStateDir();

        static const std::string FRONTEND_KEY;
        static const std::string FRONTEND_NONINTERACTIVE;
        static const std::string NOUPDATECHECK_KEY;
        static const std::string LOCALSTATEDIR_KEY;

    private:
        Environment() = delete;

    };

}

#endif
