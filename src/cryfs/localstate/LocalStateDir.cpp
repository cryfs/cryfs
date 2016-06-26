#include "LocalStateDir.h"
#include <cpp-utils/system/homedir.h>
#include <boost/filesystem.hpp>

namespace bf = boost::filesystem;

namespace cryfs {
    bf::path LocalStateDir::forFilesystemId(const CryConfig::FilesystemID &filesystemId) {
        bf::path app_dir = cpputils::system::HomeDirectory::get() / ".cryfs";
        _createDirIfNotExists(app_dir);
        bf::path filesystems_dir = app_dir / "filesystems";
        _createDirIfNotExists(filesystems_dir);
        bf::path this_filesystem_dir = filesystems_dir / filesystemId.ToString();
        _createDirIfNotExists(this_filesystem_dir);
        return this_filesystem_dir;
    }

    void LocalStateDir::_createDirIfNotExists(const bf::path &path) {
        if (!bf::exists(path)) {
            bf::create_directory(path);
        }
    }
}
