#include <cpp-utils/assert/assert.h>
#include <cpp-utils/logging/logging.h>
#include <cpp-utils/system/homedir.h>
#include <cpp-utils/crypto/kdf/Scrypt.h>
#include <cpp-utils/system/path.h>
#include <gitversion/gitversion.h>
#include <blockstore/implementations/ondisk/OnDiskBlockStore2.h>
#include <boost/algorithm/string/predicate.hpp>
#include "../impl/config/CryConfigLoader.h"
#include "../impl/config/CryPresetPasswordBasedKeyProvider.h"
#include "../impl/filesystem/CryDir.h"
#include "cryfs_load_context.h"
#include "cryfs_api_context.h"
#include "cryfs_mount_handle.h"
#include "utils/filesystem_checks.h"
#include <cstdlib>

using cpputils::make_unique_ref;
using cpputils::unique_ref;
using cpputils::dynamic_pointer_move;
using cpputils::either;
using std::string;
using std::shared_ptr;
using std::make_shared;
using std::unique_ptr;
using std::make_unique;
using boost::none;
using boost::optional;
namespace bf = boost::filesystem;

using cryfs::CryDevice;
using cryfs::CryDir;
using cryfs::CryConfig;
using cryfs::CryConfigFile;
using cryfs::CryConfigLoader;
using cryfs::LocalStateDir;
using cryfs::CryPresetPasswordBasedKeyProvider;
using cpputils::SCrypt;
using blockstore::ondisk::OnDiskBlockStore2;


using namespace cpputils::logging;

cryfs_load_context::cryfs_load_context(cryfs_api_context *api_context)
    : _api_context(api_context), _basedir(boost::none), _password(boost::none), _configfile(boost::none),
      _mount_handles() {
}

cryfs_status cryfs_load_context::free() {
    // This will call the cryfs_load_context destructor since our object is owned by api_context.
    return _api_context->delete_load_context(this);
}

cryfs_status cryfs_load_context::set_basedir(const bf::path& basedir_) {
    if (!bf::exists(basedir_)) {
        return cryfs_error_BASEDIR_DOESNT_EXIST;
    }

    bf::path basedir = bf::canonical(basedir_);

    if (!cryfs::filesystem_checks::check_dir_accessible(basedir)) {
        return cryfs_error_BASEDIR_INACCESSIBLE;
    }
    _basedir = std::move(basedir);
    return cryfs_success;
}

cryfs_status cryfs_load_context::set_password(string password) {
    _password = std::move(password);
    return cryfs_success;
}

cryfs_status cryfs_load_context::set_externalconfig(const bf::path& configfile_) {
    if (!bf::exists(configfile_)) {
        return cryfs_error_CONFIGFILE_DOESNT_EXIST;
    }

    bf::path configfile = bf::canonical(configfile_);

    if (!cryfs::filesystem_checks::check_file_readable(configfile)) {
        return cryfs_error_CONFIGFILE_NOT_READABLE;
    }
    _configfile = std::move(configfile);
    return cryfs_success;
}

cryfs_status cryfs_load_context::set_localstatedir(const bf::path& localstatedir_) {
    bf::path localstatedir = bf::weakly_canonical(localstatedir_);

    bf::path longest_existing_prefix = cpputils::find_longest_existing_path_prefix(localstatedir);

    if (!cryfs::filesystem_checks::check_dir_accessible(longest_existing_prefix)) {
        // either localstatedir_ exists and is not writeable,
        // or it doesn't exist but we can't create it because the longest existing prefix isn't writeable.
        return cryfs_error_LOCALSTATEDIR_NOT_WRITEABLE;
    }

    _localstatedir = std::move(localstatedir);
    return cryfs_success;
}

cryfs_status cryfs_load_context::load(cryfs_mount_handle **handle) {
    if (_basedir == none) {
        return cryfs_error_BASEDIR_NOT_SET;
    }
    if (_password == none) {
        return cryfs_error_PASSWORD_NOT_SET;
    }
    if (_localstatedir == none) {
        return cryfs_error_LOCALSTATEDIR_NOT_SET;
    }
    auto configfileEither = _load_configfile();
    if (configfileEither.is_left()) {
        switch (configfileEither.left()) {
            case CryConfigFile::LoadError::ConfigFileNotFound:
                return cryfs_error_CONFIGFILE_DOESNT_EXIST;
            case CryConfigFile::LoadError::DecryptionFailed:
                return cryfs_error_DECRYPTION_FAILED;
        }
    }
    std::shared_ptr<CryConfigFile> configfile = std::move(configfileEither.right());
    if(!_check_version(*configfile->config())) {
        return cryfs_error_FILESYSTEM_INCOMPATIBLE_VERSION;
    }
    //TODO CLI caller needs to check cipher if specified on command line

    auto blockstore = make_unique_ref<OnDiskBlockStore2>(*_basedir);
    LocalStateDir localStateDir(*_localstatedir);
    uint32_t myClientId = 0x12345678; // TODO Get the correct client id instead, use pattern like in CryConfigLoader for Cli.cpp.
    bool allowIntegrityViolation = false; // TODO Make this configurable
    bool missingBlockIsIntegrityViolation = false; // TODO Make this configurable

    unique_ptr<CryDevice> crydevice;
    try {
        auto onIntegrityViolation = [] {}; // TODO Make this configurable
        crydevice = make_unique<CryDevice>(configfile, std::move(blockstore), localStateDir, myClientId, allowIntegrityViolation, missingBlockIsIntegrityViolation, std::move(onIntegrityViolation));
    } catch (const std::runtime_error& e) {
        // this might be thrown if the file system tries to migrate to a newer version and the root block doesn't exist
        return cryfs_error_FILESYSTEM_INVALID;
    }
    if (!_sanity_check_filesystem(crydevice.get())) {
        return cryfs_error_FILESYSTEM_INVALID;
    }

    if (nullptr != handle) {
        // TODO Why don't we pass the CryDevice to the mount handle?
        *handle = _mount_handles.create(configfile, *_basedir, std::move(localStateDir));
    }
    return cryfs_success;
}

either<CryConfigFile::LoadError, unique_ref<CryConfigFile>> cryfs_load_context::_load_configfile() const {
    bf::path configfilePath = _determine_configfile_path();
    ASSERT(_password != none, "password not set");
    CryPresetPasswordBasedKeyProvider keyProvider(*_password, make_unique_ref<SCrypt>(SCrypt::DefaultSettings));
    return CryConfigFile::load(configfilePath, &keyProvider);
}

bf::path cryfs_load_context::_determine_configfile_path() const {
    ASSERT(_basedir != none, "basedir not set");
    if (_configfile != none) {
        return *_configfile;
    }
    return *_basedir / "cryfs.config";
}

bool cryfs_load_context::_check_version(const CryConfig &config) {
    // TODO Allow overriding this like in CryConfigLoader with allowFilesystemUpgrades?
    return config.Version() == CryConfig::FilesystemFormatVersion;
}

bool cryfs_load_context::_sanity_check_filesystem(CryDevice *device) {
    //Try to list contents of base directory
    auto _rootDir = device->Load("/"); // this might throw an exception if the root blob doesn't exist
    if (_rootDir == none) {
        LOG(ERR, "Couldn't find root blob");
        return false;
    }
    auto rootDir = dynamic_pointer_move<CryDir>(*_rootDir);
    if (rootDir == none) {
        LOG(ERR, "Root blob isn't a directory");
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
