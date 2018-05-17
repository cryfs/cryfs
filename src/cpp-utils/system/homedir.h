#pragma once
#ifndef MESSMER_CPPUTILS_SYSTEM_GETTOTALMEMORY_H
#define MESSMER_CPPUTILS_SYSTEM_GETTOTALMEMORY_H

#include <boost/filesystem/path.hpp>
#include "../macros.h"
#include <cpp-utils/pointer/unique_ref.h>
#include "../tempfile/TempDir.h"

namespace cpputils {
    namespace system {

        class FakeHomeDirectoryRAII;

        class HomeDirectory final {
        public:
            static const boost::filesystem::path &get();

            static const boost::filesystem::path &getXDGDataDir();

        private:
            boost::filesystem::path _home_directory;
			boost::filesystem::path _appdata_directory;

            HomeDirectory();
            static HomeDirectory &singleton();

            friend class FakeHomeDirectoryRAII;

            DISALLOW_COPY_AND_ASSIGN(HomeDirectory);
        };


        class FakeHomeDirectoryRAII final {
        public:
            FakeHomeDirectoryRAII(const boost::filesystem::path &fakeHomeDirectory, const boost::filesystem::path &fakeAppdataDirectory);
            ~FakeHomeDirectoryRAII();

        private:
            boost::filesystem::path _oldHomeDirectory;
			boost::filesystem::path _oldAppdataDirectory;

            DISALLOW_COPY_AND_ASSIGN(FakeHomeDirectoryRAII);
        };

		class FakeTempHomeDirectoryRAII final {
		public:
			FakeTempHomeDirectoryRAII();

		private:
			TempDir _tempDir;
			FakeHomeDirectoryRAII _fakeHome;

			DISALLOW_COPY_AND_ASSIGN(FakeTempHomeDirectoryRAII);
		};

    }
}

#endif
