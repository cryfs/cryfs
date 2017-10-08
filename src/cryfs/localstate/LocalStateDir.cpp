#include "LocalStateDir.h"
#include <cpp-utils/system/homedir.h>
#include <boost/filesystem.hpp>

namespace bf = boost::filesystem;

namespace cryfs {
    namespace {
      bf::path appDir() {
        return cpputils::system::HomeDirectory::get() / ".cryfs";
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
}
