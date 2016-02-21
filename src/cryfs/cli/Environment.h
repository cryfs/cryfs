#pragma once
#ifndef MESSMER_CRYFS_CLI_ENVIRONMENT_H
#define MESSMER_CRYFS_CLI_ENVIRONMENT_H

#include <string>

namespace cryfs {

    class Environment {
    public:
        static bool isNoninteractive();
        static bool noUpdateCheck();

    private:
        Environment() = delete;

        static const std::string FRONTEND_KEY;
        static const std::string FRONTEND_NONINTERACTIVE;
        static const std::string NOUPDATECHECK_KEY;
    };

}

#endif
