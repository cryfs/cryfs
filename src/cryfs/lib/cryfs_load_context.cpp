#include <cpp-utils/assert/assert.h>
#include <cpp-utils/logging/logging.h>
#include <gitversion/version.h>
#include <blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include <boost/algorithm/string/predicate.hpp>
#include "../impl/config/CryConfigLoader.h"
#include "../impl/filesystem/CryDir.h"
#include "cryfs_load_context.h"

using cpputils::make_unique_ref;
using cpputils::dynamic_pointer_move;
using cpputils::either;
using std::string;
using boost::none;
using boost::optional;
namespace bf = boost::filesystem;

using cryfs::CryDevice;
using cryfs::CryDir;
using cryfs::CryConfig;
using cryfs::CryConfigFile;
using cryfs::CryConfigLoader;
using blockstore::ondisk::OnDiskBlockStore;

using namespace cpputils::logging;

cryfs_load_context::cryfs_load_context()
        : _basedir(boost::none), _password(boost::none), _configfile(boost::none) {
}

cryfs_status cryfs_load_context::set_basedir(const string &basedir) {
    if (!bf::is_directory(basedir)) {
        return cryfs_error_BASEDIR_DOESNT_EXIST;
    }
    //TODO Handle (and add test cases for) missing permissions
    _basedir = basedir;
    return cryfs_success;
}

cryfs_status cryfs_load_context::set_password(const string &password) {
    _password = password;
    return cryfs_success;
}

cryfs_status cryfs_load_context::set_externalconfig(const string &configfile) {
    if (!bf::is_regular_file(configfile)) {
        return cryfs_error_CONFIGFILE_DOESNT_EXIST;
    }
    //TODO Handle (and add test cases for) missing permissions
    _configfile = configfile;
    return cryfs_success;
}

cryfs_status cryfs_load_context::load(cryfs_mount_handle **handle) {
    if (_basedir == none) {
        return cryfs_error_BASEDIR_NOT_SET;
    }
    if (_password == none) {
        return cryfs_error_PASSWORD_NOT_SET;
    }
    auto configfile = _load_configfile();
    if (configfile.is_left()) {
        switch (configfile.left()) {
            case CryConfigFile::LoadError::ConfigFileNotFound:
                return cryfs_error_CONFIGFILE_DOESNT_EXIST;
            case CryConfigFile::LoadError::DecryptionFailed:
                return cryfs_error_DECRYPTION_FAILED;
        }
    }
    if(!_check_version(*configfile.right().config())) {
        return cryfs_error_FILESYSTEM_INCOMPATIBLE_VERSION;
    }
    //TODO CLI caller needs to check cipher if specified on command line

    auto blockstore = make_unique_ref<OnDiskBlockStore>(*_basedir);
    auto crydevice = make_unique_ref<CryDevice>(std::move(configfile.right()), std::move(blockstore));
    if (!_sanity_check_filesystem(crydevice.get())) {
        return cryfs_error_FILESYSTEM_INVALID;
    }

    *handle = _keepHandleOwnership.create(std::move(crydevice));
    return cryfs_success;
}

either<CryConfigFile::LoadError, CryConfigFile> cryfs_load_context::_load_configfile() const {
    bf::path configfilePath = _determine_configfile_path();
    ASSERT(_password != none, "password not set");
    return CryConfigFile::load(configfilePath, *_password);
}

bf::path cryfs_load_context::_determine_configfile_path() const {
    ASSERT(_basedir != none, "basedir not set");
    if (_configfile != none) {
        return *_configfile;
    }
    return *_basedir / "cryfs.config";
}

bool cryfs_load_context::_check_version(const CryConfig &config) {
    const string allowedVersionPrefix = string() + version::VERSION_COMPONENTS[0] + "." + version::VERSION_COMPONENTS[1] + ".";
    return boost::starts_with(config.Version(), allowedVersionPrefix);
}

bool cryfs_load_context::_sanity_check_filesystem(CryDevice *device) {
    //Try to list contents of base directory
    auto _rootDir = device->Load("/"); // this might throw an exception if the root blob doesn't exist
    if (_rootDir == none) {
        LOG(ERROR) << "Couldn't find root blob";
        return false;
    }
    auto rootDir = dynamic_pointer_move<CryDir>(*_rootDir);
    if (rootDir == none) {
        LOG(ERROR) << "Root blob isn't a directory";
        return false;
    }
    try {
        (*rootDir)->children();
    } catch (const std::exception &e) {
        // Couldn't load root blob
        return false;
    }
    return true;
}
