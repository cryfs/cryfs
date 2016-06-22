#pragma once
#ifndef MESSMER_CPPUTILS_SYSTEM_GETTOTALMEMORY_H
#define MESSMER_CPPUTILS_SYSTEM_GETTOTALMEMORY_H

#include <boost/filesystem/path.hpp>
#include "../macros.h"
#include <cpp-utils/pointer/unique_ref.h>

namespace cpputils {
    namespace system {

        class FakeHomeDirectoryRAII;

        class HomeDirectory final {
        public:
            static const boost::filesystem::path &get();

        private:
            boost::filesystem::path _home_directory;

            HomeDirectory();
            static HomeDirectory &singleton();
            boost::filesystem::path _get_home_directory();

            friend class FakeHomeDirectoryRAII;

            DISALLOW_COPY_AND_ASSIGN(HomeDirectory);
        };


        class FakeHomeDirectoryRAII final {
        public:
            FakeHomeDirectoryRAII(const boost::filesystem::path &fakeHomeDirectory);
            ~FakeHomeDirectoryRAII();

        private:
            boost::filesystem::path _oldHomeDirectory;

            DISALLOW_COPY_AND_ASSIGN(FakeHomeDirectoryRAII);
        };

    }
}

#endif
