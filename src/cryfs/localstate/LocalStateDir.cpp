#include "LocalStateDir.h"
#include <cpp-utils/system/homedir.h>
#include <boost/filesystem.hpp>

namespace bf = boost::filesystem;

namespace cryfs {
    namespace {
      // TODO constexpr?
      bf::path& appDir() {
        static bf::path singleton = cpputils::system::HomeDirectory::get() / ".cryfs";
        return singleton;
      }
    }

    bf::path LocalStateDir::forFilesystemId(const CryConfig::FilesystemID &filesystemId) {
      _createDirIfNotExists(appDir());
      bf::path filesystems_dir = appDir() / "filesystems";
      _createDirIfNotExists(filesystems_dir);
      bf::path this_filesystem_dir = filesystems_dir / filesystemId.ToString();
      _createDirIfNotExists(this_filesystem_dir);
      return this_filesystem_dir;
    }

    bf::path LocalStateDir::forBasedirMetadata() {
      _createDirIfNotExists(appDir());
      return appDir() / "basedirs";
    }

    void LocalStateDir::_createDirIfNotExists(const bf::path &path) {
        if (!bf::exists(path)) {
            bf::create_directories(path);
        }
    }

    void LocalStateDir::setAppDir(boost::filesystem::path path) {
      appDir() = std::move(path);
    }
}
