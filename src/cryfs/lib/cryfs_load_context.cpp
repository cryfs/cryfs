#include <cpp-utils/assert/assert.h>
#include "cryfs_load_context.h"
#include "../impl/config/CryConfigLoader.h"
#include <boost/algorithm/string/predicate.hpp>
#include <gitversion/version.h>

using std::string;
using boost::none;
using boost::optional;
namespace bf = boost::filesystem;

using cryfs::CryConfig;
using cryfs::CryConfigFile;
using cryfs::CryConfigLoader;

cryfs_load_context::cryfs_load_context()
        : _basedir(boost::none), _password(boost::none), _configfile(boost::none) {
}

cryfs_status cryfs_load_context::set_basedir(const string &basedir) {
    if (!bf::is_directory(basedir)) {
        return cryfs_error_BASEDIR_DOESNT_EXIST;
    }
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
    auto config_file = _load_config_file();
    if (config_file == none) {
        //TODO More detailed error reporting. Config file not found? Invalid config file header (i.e. invalid config file)? Decryption failed (i.e. wrong password)?
        return cryfs_error_FILESYSTEM_NOT_FOUND;
    }
    if(!_check_version(*config_file->config())) {
        return cryfs_error_FILESYSTEM_INCOMPATIBLE_VERSION;
    }
    //TODO CLI caller needs to check cipher if specified on command line

    //TODO Actually load the file system here and pass the CryDevice instance to cryfs_mount_handle. This way, we can throw loading errors here already.
    *handle = _keepHandleOwnership.create(*_basedir, std::move(*config_file));
    return cryfs_success;
}

bool cryfs_load_context::_check_version(const CryConfig &config) {
    const string allowedVersionPrefix = string() + version::VERSION_COMPONENTS[0] + "." + version::VERSION_COMPONENTS[1] + ".";
    return boost::starts_with(config.Version(), allowedVersionPrefix);
}

optional<CryConfigFile> cryfs_load_context::_load_config_file() const {
    bf::path configfilePath = _determine_configfile_path();
    if (!bf::is_regular_file(configfilePath)) {
        return none;
    }
    ASSERT(_password != none, "password not set");
    auto config = CryConfigFile::load(configfilePath, *_password);
    return config;
}

bf::path cryfs_load_context::_determine_configfile_path() const {
    ASSERT(_basedir != none, "basedir not set");
    if (_configfile != none) {
        return *_configfile;
    }
    return *_basedir / "cryfs.config";
}
