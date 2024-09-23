#include "LocalStateDir.h"
#include "cryfs/impl/config/CryConfig.h"
#include <boost/filesystem/operations.hpp>
#include <boost/filesystem/path.hpp>
#include <utility>

namespace bf = boost::filesystem;

namespace cryfs {
    LocalStateDir::LocalStateDir(bf::path appDir): _appDir(std::move(appDir)) {}

    bf::path LocalStateDir::forFilesystemId(const CryConfig::FilesystemID &filesystemId) const {
      _createDirIfNotExists(_appDir);
      const bf::path filesystems_dir = _appDir / "filesystems";
      _createDirIfNotExists(filesystems_dir);
      const bf::path this_filesystem_dir = filesystems_dir / filesystemId.ToString();
      _createDirIfNotExists(this_filesystem_dir);
      return this_filesystem_dir;
    }

    bf::path LocalStateDir::forBasedirMetadata() const {
      _createDirIfNotExists(_appDir);
      return _appDir / "basedirs";
    }

    void LocalStateDir::_createDirIfNotExists(const bf::path &path) {
        if (!bf::exists(path)) {
            bf::create_directories(path);
        }
    }
}
