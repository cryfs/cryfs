#include "cryfs_mount_handle.h"

using boost::optional;
using std::string;
using cryfs::CryConfigFile;
namespace bf = boost::filesystem;

cryfs_mount_handle::cryfs_mount_handle(const bf::path &basedir, CryConfigFile config)
    :_basedir(basedir), _config(std::move(config)) {
}

