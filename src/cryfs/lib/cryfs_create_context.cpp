#include <cpp-utils/assert/assert.h>
#include <cpp-utils/logging/logging.h>
#include <gitversion/gitversion.h>
#include <boost/algorithm/string/predicate.hpp>
#include "../impl/config/CryConfigLoader.h"
#include "../impl/filesystem/CryDir.h"
#include "cryfs_create_context.h"
#include "cryfs_api_context.h"
#include "utils/filesystem_checks.h"

using cpputils::make_unique_ref;
using cpputils::unique_ref;
using cpputils::dynamic_pointer_move;
using cpputils::either;
using std::string;
using std::shared_ptr;
using std::make_shared;
using boost::none;
using boost::optional;
namespace bf = boost::filesystem;

using cryfs::CryDevice;
using cryfs::CryDir;
using cryfs::CryConfig;
using cryfs::CryConfigFile;
using cryfs::CryConfigLoader;

using namespace cpputils::logging;

cryfs_create_context::cryfs_create_context(cryfs_api_context *api_context)
    : _api_context(api_context), _basedir(boost::none), _cipher(boost::none), _password(boost::none), _configfile(boost::none) {
}

cryfs_status cryfs_create_context::free() {
    // This will call the cryfs_create_context destructor since our object is owned by api_context.
    return _api_context->delete_create_context(this);
}

cryfs_status cryfs_create_context::set_basedir(const bf::path& basedir_) {
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

cryfs_status cryfs_create_context::set_cipher(string cipher) {
  //TODO ...
  return cryfs_success;
}

cryfs_status cryfs_create_context::set_password(string password) {
  _password = std::move(password);
  return cryfs_success;
}

cryfs_status cryfs_create_context::set_externalconfig(const bf::path& configfile_) {
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

cryfs_status cryfs_create_context::create(cryfs_mount_handle **handle) {
  // TODO ...
  if (nullptr != handle) {
    //*handle = ...
  }
  return cryfs_success;
}

/* TODO
bf::path cryfs_create_context::_determine_configfile_path() const {
  ASSERT(_basedir != none, "basedir not set");
  if (_configfile != none) {
    return *_configfile;
  }
  return *_basedir / "cryfs.config";
}

bool cryfs_create_context::_sanity_check_filesystem(CryDevice *device) {
  //Try to list contents of base directory
  auto _rootDir = device->Load("/"); // this might throw an exception if the root blob doesn't exist
  if (_rootDir == none) {
    LOG(ERROR, "Couldn't find root blob");
    return false;
  }
  auto rootDir = dynamic_pointer_move<CryDir>(*_rootDir);
  if (rootDir == none) {
    LOG(ERROR, "Root blob isn't a directory");
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
*/