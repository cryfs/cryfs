// NOMINMAX works around an MSVC issue, see https://github.com/microsoft/cppwinrt/issues/479
#include "cpp-utils/assert/AssertFailed.h"
#include "cpp-utils/logging/Logger.h"
#include "cpp-utils/macros.h"
#include "fspp/fs_interface/Context.h"
#include "fspp/fs_interface/Dir.h"
#include "fspp/fs_interface/Types.h"
#include "fspp/fuse/stat_compatibility.h"
#include <algorithm>
#include <array>
#include <boost/filesystem/path.hpp>
#include <boost/none.hpp>
#include <boost/optional/optional.hpp>
#include <cerrno>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <ctime>
#include <exception>
#include <functional>
#include <locale>
#include <range/v3/iterator/operations.hpp>
#include <range/v3/view/subrange.hpp>
#include <stdexcept>
#include <string>
#include <sys/stat.h>
#include <sys/statvfs.h>
#include <sys/types.h>
#include <utility>
#include <vector>
#if defined(_MSC_VER)
#define NOMINMAX
#endif

#include "Fuse.h"
#include <memory>

#include "../fs_interface/FuseErrnoException.h"
#include "Filesystem.h"
#include "InvalidFilesystem.h"
#include <boost/algorithm/string/replace.hpp>
#include <codecvt>
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/logging/logging.h>
#include <cpp-utils/process/subprocess.h>
#include <cpp-utils/thread/debugging.h>

#include <range/v3/view/split.hpp>
#include <range/v3/view/join.hpp>
#include <range/v3/view/filter.hpp>
#include <range/v3/range/conversion.hpp>

#if defined(_MSC_VER)
#include <dokan/dokan.h>
#endif

using std::vector;
using std::string;

namespace bf = boost::filesystem;
using namespace cpputils::logging;
using std::make_shared;
using std::shared_ptr;
using std::string;
using namespace fspp::fuse;
using cpputils::set_thread_name;

namespace {
bool is_valid_fspp_path(const bf::path& path) {
  // TODO In boost 1.63, we can use path.generic() or path.generic_path() instead of path.generic_string()
  return path.has_root_directory()                     // must be absolute path
         && !path.has_root_name()                      // on Windows, it shouldn't have a device specifier (i.e. no "C:")
         && (path.string() == path.generic_string());  // must use portable '/' as directory separator
}

class ThreadNameForDebugging final {
public:
  ThreadNameForDebugging(const string& threadName) {
    const std::string name = "fspp_" + threadName;
    set_thread_name(name.c_str());
  }

  ~ThreadNameForDebugging() {
    set_thread_name("fspp_idle");
  }
};
}

#define FUSE_OBJ (static_cast<Fuse *>(fuse_get_context()->private_data))

// Remove the following line, if you don't want to output each fuse operation on the console
//#define FSPP_LOG 1

namespace {
int fusepp_getattr(const char *path, fspp::fuse::STAT *stbuf) {
  const int rs = FUSE_OBJ->getattr(bf::path(path), stbuf);
  return rs;
}

int fusepp_fgetattr(const char *path, fspp::fuse::STAT *stbuf, fuse_file_info *fileinfo) {
  return FUSE_OBJ->fgetattr(bf::path(path), stbuf, fileinfo);
}

int fusepp_readlink(const char *path, char *buf, size_t size) {
  return FUSE_OBJ->readlink(bf::path(path), buf, size);
}

int fusepp_mknod(const char *path, ::mode_t mode, dev_t rdev) {
  return FUSE_OBJ->mknod(bf::path(path), mode, rdev);
}

int fusepp_mkdir(const char *path, ::mode_t mode) {
  return FUSE_OBJ->mkdir(bf::path(path), mode);
}

int fusepp_unlink(const char *path) {
  return FUSE_OBJ->unlink(bf::path(path));
}

int fusepp_rmdir(const char *path) {
  return FUSE_OBJ->rmdir(bf::path(path));
}

int fusepp_symlink(const char *to, const char *from) {
  return FUSE_OBJ->symlink(bf::path(to), bf::path(from));
}

int fusepp_rename(const char *from, const char *to) {
  return FUSE_OBJ->rename(bf::path(from), bf::path(to));
}

int fusepp_link(const char *from, const char *to) {
  return FUSE_OBJ->link(bf::path(from), bf::path(to));
}

int fusepp_chmod(const char *path, ::mode_t mode) {
  return FUSE_OBJ->chmod(bf::path(path), mode);
}

int fusepp_chown(const char *path, ::uid_t uid, ::gid_t gid) {
  return FUSE_OBJ->chown(bf::path(path), uid, gid);
}

int fusepp_truncate(const char *path, int64_t size) {
  return FUSE_OBJ->truncate(bf::path(path), size);
}

int fusepp_ftruncate(const char *path, int64_t size, fuse_file_info *fileinfo) {
  return FUSE_OBJ->ftruncate(bf::path(path), size, fileinfo);
}

int fusepp_utimens(const char *path, const timespec times[2]) {  // NOLINT(cppcoreguidelines-avoid-c-arrays)
  return FUSE_OBJ->utimens(bf::path(path), {times[0], times[1]});
}

int fusepp_open(const char *path, fuse_file_info *fileinfo) {
  return FUSE_OBJ->open(bf::path(path), fileinfo);
}

int fusepp_release(const char *path, fuse_file_info *fileinfo) {
  return FUSE_OBJ->release(bf::path(path), fileinfo);
}

int fusepp_read(const char *path, char *buf, size_t size, int64_t offset, fuse_file_info *fileinfo) {
  return FUSE_OBJ->read(bf::path(path), buf, size, offset, fileinfo);
}

int fusepp_write(const char *path, const char *buf, size_t size, int64_t offset, fuse_file_info *fileinfo) {
  return FUSE_OBJ->write(bf::path(path), buf, size, offset, fileinfo);
}

int fusepp_statfs(const char *path, struct statvfs *fsstat) {
  return FUSE_OBJ->statfs(bf::path(path), fsstat);
}

int fusepp_flush(const char *path, fuse_file_info *fileinfo) {
  return FUSE_OBJ->flush(bf::path(path), fileinfo);
}

int fusepp_fsync(const char *path, int datasync, fuse_file_info *fileinfo) {
  return FUSE_OBJ->fsync(bf::path(path), datasync, fileinfo);
}

//int fusepp_setxattr(const char*, const char*, const char*, size_t, int)
//int fusepp_getxattr(const char*, const char*, char*, size_t)
//int fusepp_listxattr(const char*, char*, size_t)
//int fusepp_removexattr(const char*, const char*)

int fusepp_opendir(const char *path, fuse_file_info *fileinfo) {
  return FUSE_OBJ->opendir(bf::path(path), fileinfo);
}

int fusepp_readdir(const char *path, void *buf, fuse_fill_dir_t filler, int64_t offset, fuse_file_info *fileinfo) {
  return FUSE_OBJ->readdir(bf::path(path), buf, filler, offset, fileinfo);
}

int fusepp_releasedir(const char *path, fuse_file_info *fileinfo) {
  return FUSE_OBJ->releasedir(bf::path(path), fileinfo);
}

int fusepp_fsyncdir(const char *path, int datasync, fuse_file_info *fileinfo) {
  return FUSE_OBJ->fsyncdir(bf::path(path), datasync, fileinfo);
}

void* fusepp_init(fuse_conn_info *conn) {
  auto f = FUSE_OBJ;
  f->init(conn);
  return f;
}

void fusepp_destroy(void *userdata) {
  auto f = FUSE_OBJ;
  ASSERT(userdata == f, "Wrong userdata set");
  UNUSED(userdata); //In case the assert is disabled
  f->destroy();
}

int fusepp_access(const char *path, int mask) {
  return FUSE_OBJ->access(bf::path(path), mask);
}

int fusepp_create(const char *path, ::mode_t mode, fuse_file_info *fileinfo) {
  return FUSE_OBJ->create(bf::path(path), mode, fileinfo);
}

/*int fusepp_lock(const char*, fuse_file_info*, int cmd, flock*)
int fusepp_bmap(const char*, size_t blocksize, uint64_t *idx)
int fusepp_ioctl(const char*, int cmd, void *arg, fuse_file_info*, unsigned int flags, void *data)
int fusepp_poll(const char*, fuse_file_info*, fuse_pollhandle *ph, unsigned *reventsp)
int fusepp_write_buf(const char*, fuse_bufvec *buf, int64_t off, fuse_file_info*)
int fusepp_read_buf(const chas*, struct fuse_bufvec **bufp, size_t size, int64_t off, fuse_file_info*)
int fusepp_flock(const char*, fuse_file_info*, int op)
int fusepp_fallocate(const char*, int, int64_t, int64_t, fuse_file_info*)*/

fuse_operations *operations() {
  static std::unique_ptr<fuse_operations> singleton(nullptr);

  if (!singleton) {
    singleton = std::make_unique<fuse_operations>();
    singleton->getattr = &fusepp_getattr;
    singleton->fgetattr = &fusepp_fgetattr;
    singleton->readlink = &fusepp_readlink;
    singleton->mknod = &fusepp_mknod;
    singleton->mkdir = &fusepp_mkdir;
    singleton->unlink = &fusepp_unlink;
    singleton->rmdir = &fusepp_rmdir;
    singleton->symlink = &fusepp_symlink;
    singleton->rename = &fusepp_rename;
    singleton->link = &fusepp_link;
    singleton->chmod = &fusepp_chmod;
    singleton->chown = &fusepp_chown;
    singleton->truncate = &fusepp_truncate;
    singleton->utimens = &fusepp_utimens;
    singleton->open = &fusepp_open;
    singleton->read = &fusepp_read;
    singleton->write = &fusepp_write;
    singleton->statfs = &fusepp_statfs;
    singleton->flush = &fusepp_flush;
    singleton->release = &fusepp_release;
    singleton->fsync = &fusepp_fsync;
  /*#ifdef HAVE_SYS_XATTR_H
    singleton->setxattr = &fusepp_setxattr;
    singleton->getxattr = &fusepp_getxattr;
    singleton->listxattr = &fusepp_listxattr;
    singleton->removexattr = &fusepp_removexattr;
  #endif*/
    singleton->opendir = &fusepp_opendir;
    singleton->readdir = &fusepp_readdir;
    singleton->releasedir = &fusepp_releasedir;
    singleton->fsyncdir = &fusepp_fsyncdir;
    singleton->init = &fusepp_init;
    singleton->destroy = &fusepp_destroy;
    singleton->access = &fusepp_access;
    singleton->create = &fusepp_create;
    singleton->ftruncate = &fusepp_ftruncate;
  }

  return singleton.get();
}
}

Fuse::~Fuse() {
  for(char *arg : _argv) {
    delete[] arg;
    arg = nullptr;
  }
  _argv.clear();
}

Fuse::Fuse(std::function<shared_ptr<Filesystem> (Fuse *fuse)> init, std::function<void()> onMounted, std::string fstype, boost::optional<std::string> fsname)
  :_init(std::move(init)), _onMounted(std::move(onMounted)), _fs(make_shared<InvalidFilesystem>()), _mountdir(), _running(false), _fstype(std::move(fstype)), _fsname(std::move(fsname)) {
  ASSERT(static_cast<bool>(_init), "Invalid init given");
  ASSERT(static_cast<bool>(_onMounted), "Invalid onMounted given");
}

void Fuse::_logException(const std::exception &e) {
  LOG(ERR, "Exception thrown: {}", e.what());
}

void Fuse::_logUnknownException() {
  LOG(ERR, "Unknown exception thrown");
}

void Fuse::runInForeground(const bf::path &mountdir, vector<string> fuseOptions) {
  vector<string> realFuseOptions = std::move(fuseOptions);
  if (std::find(realFuseOptions.begin(), realFuseOptions.end(), "-f") == realFuseOptions.end()) {
    realFuseOptions.push_back("-f");
  }
  _run(mountdir, std::move(realFuseOptions));
}

void Fuse::runInBackground(const bf::path &mountdir, vector<string> fuseOptions) {
  vector<string> realFuseOptions = std::move(fuseOptions);
  _removeAndWarnIfExists(&realFuseOptions, "-f");
  _removeAndWarnIfExists(&realFuseOptions, "-d");
  _run(mountdir, std::move(realFuseOptions));
}

void Fuse::_removeAndWarnIfExists(vector<string> *fuseOptions, const std::string &option) {
  auto found = std::find(fuseOptions->begin(), fuseOptions->end(), option);
  while (found != fuseOptions->end()) {
    LOG(WARN, "The fuse option {} only works when running in foreground. Removing fuse option.", option);
    fuseOptions->erase(found);
    found = std::find(fuseOptions->begin(), fuseOptions->end(), option);
  }
}

namespace {
void extractAllAtimeOptionsAndRemoveOnesUnknownToLibfuse_(string* csv_options, vector<string>* result) {
    const auto is_fuse_supported_atime_flag = [] (const std::string& flag) {
        constexpr std::array<const char*, 2> flags = {"noatime", "atime"};
        return flags.end() != std::find(flags.begin(), flags.end(), flag);
    };
    const auto is_fuse_unsupported_atime_flag = [] (const std::string& flag) {
        constexpr std::array<const char*, 3> flags = {"strictatime", "relatime", "nodiratime"};
        return flags.end() != std::find(flags.begin(), flags.end(), flag);
    };
    *csv_options = ranges::make_subrange(csv_options->begin(), csv_options->end()) | ranges::views::split(',') | ranges::views::filter(
      [&](auto &&elem_) {
          // TODO string_view would be better
          const std::string elem(&*elem_.begin(), ranges::distance(elem_));
          if (is_fuse_unsupported_atime_flag(elem)) {
              result->push_back(elem);
              return false;
          }
          if (is_fuse_supported_atime_flag(elem)) {
              result->push_back(elem);
          }
          return true;
      }) | ranges::views::join(',') | ranges::to<string>();
  }

// Return a list of all atime options (e.g. atime, noatime, relatime, strictatime, nodiratime) that occur in the
// fuseOptions input. They must be preceded by a '-o', i.e. {..., '-o', 'noatime', ...} and multiple ones can be
// csv-concatenated, i.e. {..., '-o', 'atime,nodiratime', ...}.
// Also, this function removes all of these atime options that are unknown to libfuse (i.e. all except atime and noatime)
// from the input fuseOptions so we can pass it on to libfuse without crashing.
vector<string> extractAllAtimeOptionsAndRemoveOnesUnknownToLibfuse_(vector<string>* fuseOptions) {
    vector<string> result;
    bool lastOptionWasDashO = false;
    for (size_t i = 0; i < fuseOptions->size(); ++i)
    {
      string &option = (*fuseOptions)[i];
      if (lastOptionWasDashO)
      {
        extractAllAtimeOptionsAndRemoveOnesUnknownToLibfuse_(&option, &result);
        if (option.empty()) {
          // All options were removed, remove the empty argument
          fuseOptions->erase(fuseOptions->begin() + i);
          --i;
          // And also remove the now value-less '-o' before it
          fuseOptions->erase(fuseOptions->begin() + i);
          --i;
        }
      }
      lastOptionWasDashO = (option == "-o");
    }

    return result;
}
}

void Fuse::_run(const bf::path &mountdir, vector<string> fuseOptions) {
#if defined(__GLIBC__)|| defined(__APPLE__) || defined(_MSC_VER)
  // Avoid encoding errors for non-utf8 characters, see https://github.com/cryfs/cryfs/issues/247
  // this is ifdef'd out for non-glibc linux, because musl doesn't handle this correctly.
  #ifdef __clang__
  #pragma clang diagnostic push
  #pragma clang diagnostic ignored "-Wdeprecated-declarations"
  #endif
  bf::path::imbue(std::locale(std::locale(), new std::codecvt_utf8_utf16<wchar_t>()));
  #ifdef __clang__
  #pragma clang diagnostic pop
  #endif
#endif

  _mountdir = mountdir;

  ASSERT(_argv.size() == 0, "Filesystem already started");

  const vector<string> atimeOptions = extractAllAtimeOptionsAndRemoveOnesUnknownToLibfuse_(&fuseOptions);
  _createContext(atimeOptions);

  _argv = _build_argv(mountdir, fuseOptions);

  fuse_main(_argv.size(), _argv.data(), operations(), this);
}

void Fuse::_createContext(const vector<string> &fuseOptions) {
    const bool has_atime_flag = fuseOptions.end() != std::find(fuseOptions.begin(), fuseOptions.end(), "atime");
    const bool has_noatime_flag = fuseOptions.end() != std::find(fuseOptions.begin(), fuseOptions.end(), "noatime");
    const bool has_relatime_flag = fuseOptions.end() != std::find(fuseOptions.begin(), fuseOptions.end(), "relatime");
    const bool has_strictatime_flag = fuseOptions.end() != std::find(fuseOptions.begin(), fuseOptions.end(), "strictatime");
    const bool has_nodiratime_flag = fuseOptions.end() != std::find(fuseOptions.begin(), fuseOptions.end(), "nodiratime");

    // Default is NOATIME, this reduces the probability for synchronization conflicts
    _context = Context(noatime());

    if (has_noatime_flag) {
        ASSERT(!has_atime_flag, "Cannot have both, noatime and atime flags set.");
        ASSERT(!has_relatime_flag, "Cannot have both, noatime and relatime flags set.");
        ASSERT(!has_strictatime_flag, "Cannot have both, noatime and strictatime flags set.");
        // note: can have nodiratime flag set but that is ignored because it is already included in the noatime policy.
        _context->setTimestampUpdateBehavior(noatime());
    } else if (has_relatime_flag) {
        // note: can have atime and relatime both set, they're identical
        ASSERT(!has_noatime_flag, "This shouldn't happen, or we would have hit a case above.");
        ASSERT(!has_strictatime_flag, "Cannot have both, relatime and strictatime flags set.");
        if (has_nodiratime_flag) {
            _context->setTimestampUpdateBehavior(nodiratime_relatime());
        } else {
            _context->setTimestampUpdateBehavior(relatime());
        }
    } else if (has_atime_flag) {
        // note: can have atime and relatime both set, they're identical
        ASSERT(!has_noatime_flag, "This shouldn't happen, or we would have hit a case above");
        ASSERT(!has_strictatime_flag, "Cannot have both, atime and strictatime flags set.");
        if (has_nodiratime_flag) {
            _context->setTimestampUpdateBehavior(nodiratime_relatime());
        } else {
            _context->setTimestampUpdateBehavior(relatime());
        }
    } else if (has_strictatime_flag) {
        ASSERT(!has_noatime_flag, "This shouldn't happen, or we would have hit a case above");
        ASSERT(!has_atime_flag, "This shouldn't happen, or we would have hit a case above");
        ASSERT(!has_relatime_flag, "This shouldn't happen, or we would have hit a case above");
        if (has_nodiratime_flag) {
            _context->setTimestampUpdateBehavior(nodiratime_strictatime());
        } else {
            _context->setTimestampUpdateBehavior(strictatime());
        }
    } else if (has_nodiratime_flag) {
        ASSERT(!has_noatime_flag, "This shouldn't happen, or we would have hit a case above");
        ASSERT(!has_atime_flag, "This shouldn't happen, or we would have hit a case above");
        ASSERT(!has_relatime_flag, "This shouldn't happen, or we would have hit a case above");
        ASSERT(!has_strictatime_flag, "This shouldn't happen, or we would have hit a case above");
        _context->setTimestampUpdateBehavior(noatime()); // use noatime by default
    }
}

vector<char *> Fuse::_build_argv(const bf::path &mountdir, const vector<string> &fuseOptions) {
  vector<char *> argv;
  argv.reserve(6 + fuseOptions.size()); // fuseOptions + executable name + mountdir + 2x fuse options (subtype, fsname), each taking 2 entries ("-o", "key=value").
  argv.push_back(_create_c_string(_fstype)); // The first argument (executable name) is the file system type
  argv.push_back(_create_c_string(mountdir.string())); // The second argument is the mountdir
  for (const string &option : fuseOptions) {
    argv.push_back(_create_c_string(option));
  }
  _add_fuse_option_if_not_exists(&argv, "subtype", _fstype);
  auto fsname = _fsname.get_value_or(_fstype);
  boost::replace_all(fsname, ",", "\\,"); // Avoid fuse options parser bug where a comma in the fsname is misinterpreted as an options delimiter, see https://github.com/cryfs/cryfs/issues/326
  _add_fuse_option_if_not_exists(&argv, "fsname", fsname);
#ifdef __APPLE__
  // Make volume name default to mountdir on macOS
  _add_fuse_option_if_not_exists(&argv, "volname", mountdir.filename().string());
#endif
  // TODO Also set read/write size for macFUSE. The options there are called differently.
  // large_read not necessary because reads are large anyhow. This option is only important for 2.4.
  //argv.push_back(_create_c_string("-o"));
  //argv.push_back(_create_c_string("large_read"));
  argv.push_back(_create_c_string("-o"));
  argv.push_back(_create_c_string("big_writes"));
  return argv;
}

void Fuse::_add_fuse_option_if_not_exists(vector<char *> *argv, const string &key, const string &value) {
  if(!_has_option(*argv, key)) {
    argv->push_back(_create_c_string("-o"));
    argv->push_back(_create_c_string(key + "=" + value));
  }
}

bool Fuse::_has_option(const vector<char *> &vec, const string &key) {
  // The fuse option can either be present as "-okey=value" or as "-o key=value", we have to check both.
  return _has_entry_with_prefix(key + "=", vec) || _has_entry_with_prefix("-o" + key + "=", vec);
}

bool Fuse::_has_entry_with_prefix(const string &prefix, const vector<char *> &vec) {
  auto found = std::find_if(vec.begin(), vec.end(), [&prefix](const char *entry) {
      return 0 == std::strncmp(prefix.c_str(), entry, prefix.size());
  });
  return found != vec.end();
}

char *Fuse::_create_c_string(const string &str) {
  // The memory allocated here is destroyed in the destructor of the Fuse class.
  char *c_str = new char[str.size()+1];
  std::memcpy(c_str, str.c_str(), str.size()+1);
  return c_str;
}

bool Fuse::running() const {
  return _running;
}

void Fuse::stop() {
  unmount(_mountdir, false);
}

void Fuse::unmount(const bf::path& mountdir, bool force) {
  //TODO Find better way to unmount (i.e. don't use external fusermount). Unmounting by kill(getpid(), SIGINT) worked, but left the mount directory transport endpoint as not connected.
#if defined(__APPLE__)
  UNUSED(force);
  int returncode = cpputils::Subprocess::call("umount", {mountdir.string()}, "").exitcode;
#elif defined(_MSC_VER)
  UNUSED(force);
  std::wstring mountdir_ = std::wstring_convert<std::codecvt_utf8_utf16<wchar_t>>().from_bytes(mountdir.string());
  BOOL success = DokanRemoveMountPoint(mountdir_.c_str());
  int returncode = success ? 0 : -1;
#else
  const std::vector<std::string> args = force ? std::vector<std::string>({"-u", mountdir.string()}) : std::vector<std::string>({"-u", "-z", mountdir.string()});  // "-z" takes care that if the filesystem can't be unmounted right now because something is opened, it will be unmounted as soon as it can be.
  const int returncode = cpputils::Subprocess::call("fusermount", args, "").exitcode;
#endif
  if (returncode != 0) {
    throw std::runtime_error("Could not unmount filesystem");
  }
}

int Fuse::getattr(const bf::path &path, fspp::fuse::STAT *stbuf) {
  const ThreadNameForDebugging _threadName("getattr");
#ifdef FSPP_LOG
  LOG(DEBUG, "getattr({}, _, _)", path);
#endif
  try {
    ASSERT(is_valid_fspp_path(path), "has to be an absolute path");
    _fs->lstat(path, stbuf);
#ifdef FSPP_LOG
    LOG(DEBUG, "getattr({}, _, _): success", path);
#endif
    return 0;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::getattr: {}", e.what());
    return -EIO;
  } catch(const fspp::fuse::FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "getattr({}, _, _): failed with errno {}", path, e.getErrno());
#endif
    return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::fgetattr(const bf::path &path, fspp::fuse::STAT *stbuf, fuse_file_info *fileinfo) {
  const ThreadNameForDebugging _threadName("fgetattr");
#ifdef FSPP_LOG
  LOG(DEBUG, "fgetattr({}, _, _)", path);
#endif

  // On FreeBSD, trying to do anything with the mountpoint ends up
  // opening it, and then using the FD for an fgetattr.  So in the
  // special case of a path of "/", I need to do a getattr on the
  // underlying base directory instead of doing the fgetattr().
  // TODO Check if necessary
  if (path.string() == "/") {
    const int result = getattr(path, stbuf);
#ifdef FSPP_LOG
    LOG(DEBUG, "fgetattr({}, _, _): success", path);
#endif
    return result;
  }

  try {
    ASSERT(is_valid_fspp_path(path), "has to be an absolute path");
    _fs->fstat(fileinfo->fh, stbuf);
#ifdef FSPP_LOG
    LOG(DEBUG, "fgetattr({}, _, _): success", path);
#endif
    return 0;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::fgetattr: {}", e.what());
    return -EIO;
  } catch(const fspp::fuse::FuseErrnoException &e) {
#ifdef FSPP_LOG
      LOG(ERR, "fgetattr({}, _, _): error", path);
#endif
    return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::readlink(const bf::path &path, char *buf, size_t size) {
  const ThreadNameForDebugging _threadName("readlink");
#ifdef FSPP_LOG
  LOG(DEBUG, "readlink({}, _, {})", path, size);
#endif
  try {
    ASSERT(is_valid_fspp_path(path), "has to be an absolute path");
    _fs->readSymlink(path, buf, fspp::num_bytes_t(size));
#ifdef FSPP_LOG
    LOG(DEBUG, "readlink({}, _, {}): success", path, size);
#endif
    return 0;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::readlink: {}", e.what());
    return -EIO;
  } catch (fspp::fuse::FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "readlink({}, _, {}): failed with errno {}", path, size, e.getErrno());
#endif
    return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::mknod(const bf::path &path, ::mode_t mode, dev_t rdev) {
  UNUSED(rdev);
  UNUSED(mode);
  UNUSED(path);
  const ThreadNameForDebugging _threadName("mknod");
  LOG(WARN, "Called non-implemented mknod({}, {}, _)", path.string(), mode);
  return ENOSYS;
}

int Fuse::mkdir(const bf::path &path, ::mode_t mode) {
  const ThreadNameForDebugging _threadName("mkdir");
#ifdef FSPP_LOG
  LOG(DEBUG, "mkdir({}, {})", path.string(), mode);
#endif
  try {
    ASSERT(is_valid_fspp_path(path), "has to be an absolute path");
	// DokanY seems to call mkdir("/"). Ignore that
	if ("/" == path) {
#ifdef FSPP_LOG
        LOG(DEBUG, "mkdir({}, {}): ignored", path.string(), mode);
#endif
		return 0;
	}

    auto context = fuse_get_context();
    _fs->mkdir(path, mode, context->uid, context->gid);
#ifdef FSPP_LOG
    LOG(DEBUG, "mkdir({}, {}): success", path, mode);
#endif
    return 0;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::mkdir: {}", e.what());
    return -EIO;
  } catch(const fspp::fuse::FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "mkdir({}, {}): failed with errno {}", path, mode, e.getErrno());
#endif
    return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::unlink(const bf::path &path) {
  const ThreadNameForDebugging _threadName("unlink");
#ifdef FSPP_LOG
  LOG(DEBUG, "unlink({})", path);
#endif
  try {
    ASSERT(is_valid_fspp_path(path), "has to be an absolute path");
    _fs->unlink(path);
#ifdef FSPP_LOG
    LOG(DEBUG, "unlink({}): success", path);
#endif
    return 0;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::unlink: {}", e.what());
    return -EIO;
  } catch(const fspp::fuse::FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "unlink({}): failed with errno {}", path, e.getErrno());
#endif
    return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::rmdir(const bf::path &path) {
  const ThreadNameForDebugging _threadName("rmdir");
#ifdef FSPP_LOG
  LOG(DEBUG, "rmdir({})", path);
#endif
  try {
    ASSERT(is_valid_fspp_path(path), "has to be an absolute path");
    _fs->rmdir(path);
#ifdef FSPP_LOG
    LOG(DEBUG, "rmdir({}): success", path);
#endif
    return 0;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::rmdir: {}", e.what());
    return -EIO;
  } catch(const fspp::fuse::FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "rmdir({}): failed with errno {}", path, e.getErrno());
#endif
    return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::symlink(const bf::path &to, const bf::path &from) {
  const ThreadNameForDebugging _threadName("symlink");
#ifdef FSPP_LOG
  LOG(DEBUG, "symlink({}, {})", to, from);
#endif
  try {
    ASSERT(is_valid_fspp_path(from), "has to be an absolute path");
	auto context = fuse_get_context();
    _fs->createSymlink(to, from, context->uid, context->gid);
#ifdef FSPP_LOG
    LOG(DEBUG, "symlink({}, {}): success", to, from);
#endif
    return 0;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::symlink: {}", e.what());
    return -EIO;
  } catch(const fspp::fuse::FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "symlink({}, {}): failed with errno {}", to, from, e.getErrno());
#endif
    return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::rename(const bf::path &from, const bf::path &to) {
  const ThreadNameForDebugging _threadName("rename");
#ifdef FSPP_LOG
  LOG(DEBUG, "rename({}, {})", from, to);
#endif
  try {
    ASSERT(is_valid_fspp_path(from), "from has to be an absolute path");
    ASSERT(is_valid_fspp_path(to), "rename target has to be an absolute path. If this assert throws, we have to add code here that makes the path absolute.");
    _fs->rename(from, to);
#ifdef FSPP_LOG
    LOG(DEBUG, "rename({}, {}): success", from, to);
#endif
    return 0;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::rename: {}", e.what());
    return -EIO;
  } catch(const fspp::fuse::FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "rename({}, {}): failed with errno {}", from, to, e.getErrno());
#endif
    return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

//TODO
int Fuse::link(const bf::path &from, const bf::path &to) {
  const ThreadNameForDebugging _threadName("link");
  LOG(WARN, "NOT IMPLEMENTED: link({}, {})", from.string(), to.string());
  //auto real_from = _impl->RootDir() / from;
  //auto real_to = _impl->RootDir() / to;
  //int retstat = ::link(real_from.string().c_str(), real_to.string().c_str());
  //return errcode_map(retstat);
  return ENOSYS;
}

int Fuse::chmod(const bf::path &path, ::mode_t mode) {
  const ThreadNameForDebugging _threadName("chmod");
#ifdef FSPP_LOG
  LOG(DEBUG, "chmod({}, {})", path, mode);
#endif
  try {
    ASSERT(is_valid_fspp_path(path), "has to be an absolute path");
	_fs->chmod(path, mode);
#ifdef FSPP_LOG
    LOG(DEBUG, "chmod({}, {}): success", path, mode);
#endif
	return 0;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::chmod: {}", e.what());
    return -EIO;
  } catch (fspp::fuse::FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "chmod({}, {}): failed with errno {}", path, mode, e.getErrno());
#endif
	return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::chown(const bf::path &path, ::uid_t uid, ::gid_t gid) {
  const ThreadNameForDebugging _threadName("chown");
#ifdef FSPP_LOG
  LOG(DEBUG, "chown({}, {}, {})", path, uid, gid);
#endif
  try {
    ASSERT(is_valid_fspp_path(path), "has to be an absolute path");
	_fs->chown(path, uid, gid);
#ifdef FSPP_LOG
    LOG(DEBUG, "chown({}, {}, {}): success", path, uid, gid);
#endif
	return 0;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::chown: {}", e.what());
    return -EIO;
  } catch (fspp::fuse::FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "chown({}, {}, {}): failed with errno {}", path, uid, gid, e.getErrno());
#endif
	return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::truncate(const bf::path &path, int64_t size) {
  const ThreadNameForDebugging _threadName("truncate");
#ifdef FSPP_LOG
  LOG(DEBUG, "truncate({}, {})", path, size);
#endif
  try {
    ASSERT(is_valid_fspp_path(path), "has to be an absolute path");
    _fs->truncate(path, fspp::num_bytes_t(size));
#ifdef FSPP_LOG
    LOG(DEBUG, "truncate({}, {}): success", path, size);
#endif
    return 0;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::truncate: {}", e.what());
    return -EIO;
  } catch (FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "truncate({}, {}): failed with errno {}", path, size, e.getErrno());
#endif
    return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::ftruncate(const bf::path &path, int64_t size, fuse_file_info *fileinfo) {
  const ThreadNameForDebugging _threadName("ftruncate");
#ifdef FSPP_LOG
  LOG(DEBUG, "ftruncate({}, {})", path, size);
#endif
  UNUSED(path);
  try {
    _fs->ftruncate(fileinfo->fh, fspp::num_bytes_t(size));
#ifdef FSPP_LOG
    LOG(DEBUG, "ftruncate({}, {}): success", path, size);
#endif
    return 0;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::ftruncate: {}", e.what());
    return -EIO;
  } catch (FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "ftruncate({}, {}): failed with errno {}", path, size, e.getErrno());
#endif
    return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::utimens(const bf::path &path, const std::array<timespec, 2> times) {
  const ThreadNameForDebugging _threadName("utimens");
#ifdef FSPP_LOG
  LOG(DEBUG, "utimens({}, _)", path);
#endif
  try {
    ASSERT(is_valid_fspp_path(path), "has to be an absolute path");
    _fs->utimens(path, times[0], times[1]);
#ifdef FSPP_LOG
    LOG(DEBUG, "utimens({}, _): success", path);
#endif
    return 0;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::utimens: {}", e.what());
    return -EIO;
  } catch (FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "utimens({}, _): failed with errno {}", path, e.getErrno());
#endif
    return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::open(const bf::path &path, fuse_file_info *fileinfo) {
  const ThreadNameForDebugging _threadName("open");
#ifdef FSPP_LOG
  LOG(DEBUG, "open({}, _)", path);
#endif
  try {
    ASSERT(is_valid_fspp_path(path), "has to be an absolute path");
    fileinfo->fh = _fs->openFile(path, fileinfo->flags);
#ifdef FSPP_LOG
    LOG(DEBUG, "open({}, _): success", path);
#endif
    return 0;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::open: {}", e.what());
    return -EIO;
  } catch (FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "open({}, _): failed with errno {}", path, e.getErrno());
#endif
    return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::release(const bf::path &path, fuse_file_info *fileinfo) {
  const ThreadNameForDebugging _threadName("release");
#ifdef FSPP_LOG
  LOG(DEBUG, "release({}, _)", path);
#endif
  UNUSED(path);
  try {
    _fs->closeFile(fileinfo->fh);
#ifdef FSPP_LOG
    LOG(DEBUG, "release({}, _): success", path);
#endif
    return 0;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::release: {}", e.what());
    return -EIO;
  } catch (FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "release({}, _): failed with errno {}", path, e.getErrno());
#endif
    return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::read(const bf::path &path, char *buf, size_t size, int64_t offset, fuse_file_info *fileinfo) {
  const ThreadNameForDebugging _threadName("read");
#ifdef FSPP_LOG
  LOG(DEBUG, "read({}, _, {}, {}, _)", path, size, offset);
#endif
  UNUSED(path);
  try {
    const int result = _fs->read(fileinfo->fh, buf, fspp::num_bytes_t(size), fspp::num_bytes_t(offset)).value();
#ifdef FSPP_LOG
    LOG(DEBUG, "read({}, _, {}, {}, _): success with {}", path, size, offset, result);
#endif
    return result;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::read: {}", e.what());
    return -EIO;
  } catch (FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "read({}, _, {}, {}, _): failed with errno {}", path, size, offset, e.getErrno());
#endif
    return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::write(const bf::path &path, const char *buf, size_t size, int64_t offset, fuse_file_info *fileinfo) {
  const ThreadNameForDebugging _threadName("write");
#ifdef FSPP_LOG
  LOG(DEBUG, "write({}, _, {}, {}, _)", path, size, offset);
#endif
  UNUSED(path);
  try {
    _fs->write(fileinfo->fh, buf, fspp::num_bytes_t(size), fspp::num_bytes_t(offset));
#ifdef FSPP_LOG
    LOG(DEBUG, "write({}, _, {}, {}, _): success", path, size, offset);
#endif
    return size;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::write: {}", e.what());
    return -EIO;
  } catch (FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "write({}, _, {}, {}, _): failed with errno {}", path, size, offset, e.getErrno());
#endif
    return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::statfs(const bf::path &path, struct ::statvfs *fsstat) {
  const ThreadNameForDebugging _threadName("statfs");
#ifdef FSPP_LOG
  LOG(DEBUG, "statfs({}, _)", path);
#endif
  UNUSED(path);
  try {
    ASSERT(is_valid_fspp_path(path), "has to be an absolute path");
    _fs->statfs(fsstat);
#ifdef FSPP_LOG
    LOG(DEBUG, "statfs({}, _): success", path);
#endif
    return 0;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::statfs: {}", e.what());
    return -EIO;
  } catch (FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "statfs({}, _): failed with errno {}", path, e.getErrno());
#endif
    return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::flush(const bf::path &path, fuse_file_info *fileinfo) {
  const ThreadNameForDebugging _threadName("flush");
#ifdef FSPP_LOG
  LOG(WARN, "flush({}, _)", path);
#endif
  UNUSED(path);
  try {
    _fs->flush(fileinfo->fh);
#ifdef FSPP_LOG
    LOG(WARN, "flush({}, _): success", path);
#endif
    return 0;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::flush: {}", e.what());
    return -EIO;
  } catch (FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "flush({}, _): failed with errno {}", path, e.getErrno());
#endif
    return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::fsync(const bf::path &path, int datasync, fuse_file_info *fileinfo) {
  const ThreadNameForDebugging _threadName("fsync");
#ifdef FSPP_LOG
  LOG(DEBUG, "fsync({}, {}, _)", path, datasync);
#endif
  UNUSED(path);
  try {
    if (datasync) {
      _fs->fdatasync(fileinfo->fh);
    } else {
      _fs->fsync(fileinfo->fh);
    }
#ifdef FSPP_LOG
  LOG(DEBUG, "fsync({}, {}, _): success", path, datasync);
#endif
    return 0;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::fsync: {}", e.what());
    return -EIO;
  } catch (FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "fsync({}, {}, _): failed with errno {}", path, datasync, e.getErrno());
#endif
    return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::opendir(const bf::path &path, fuse_file_info *fileinfo) {
  UNUSED(path);
  UNUSED(fileinfo);
  const ThreadNameForDebugging _threadName("opendir");
  //LOG(DEBUG, "opendir({}, _)", path);
  //We don't need opendir, because readdir works directly on the path
  return 0;
}

int Fuse::readdir(const bf::path &path, void *buf, fuse_fill_dir_t filler, int64_t offset, fuse_file_info *fileinfo) {
  const ThreadNameForDebugging _threadName("readdir");
#ifdef FSPP_LOG
  LOG(DEBUG, "readdir({}, _, _, {}, _)", path, offset);
#endif
  UNUSED(fileinfo);
  UNUSED(offset);
  try {
    ASSERT(is_valid_fspp_path(path), "has to be an absolute path");
    auto entries = _fs->readDir(path);
    fspp::fuse::STAT stbuf{};
    for (const auto &entry : entries) {
      //We could pass more file metadata to filler() in its third parameter,
      //but it doesn't help performance since fuse ignores everything in stbuf
      //except for file-type bits in st_mode and (if used) st_ino.
      //It does getattr() calls on all entries nevertheless.
      if (entry.type == Dir::EntryType::DIR) {
        stbuf.st_mode = S_IFDIR;
      } else if (entry.type == Dir::EntryType::FILE) {
        stbuf.st_mode = S_IFREG;
      } else if (entry.type == Dir::EntryType::SYMLINK) {
        stbuf.st_mode = S_IFLNK;
      } else {
        ASSERT(false, "Unknown entry type");
      }
      if (filler(buf, entry.name.c_str(), &stbuf, 0) != 0) {
#ifdef FSPP_LOG
        LOG(DEBUG, "readdir({}, _, _, {}, _): failure with ENOMEM", path, offset);
#endif
        return -ENOMEM;
      }
    }
#ifdef FSPP_LOG
    LOG(DEBUG, "readdir({}, _, _, {}, _): success", path, offset);
#endif
    return 0;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::readdir: {}", e.what());
    return -EIO;
  } catch (FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "readdir({}, _, _, {}, _): failed with errno {}", path, offset, e.getErrno());
#endif
    return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::releasedir(const bf::path &path, fuse_file_info *fileinfo) {
  UNUSED(path);
  UNUSED(fileinfo);
  const ThreadNameForDebugging _threadName("releasedir");
  //LOG(DEBUG, "releasedir({}, _)", path);
  //We don't need releasedir, because readdir works directly on the path
  return 0;
}

//TODO
int Fuse::fsyncdir(const bf::path &path, int datasync, fuse_file_info *fileinfo) {
  UNUSED(fileinfo);
  UNUSED(datasync);
  UNUSED(path);
  const ThreadNameForDebugging _threadName("fsyncdir");
  //LOG(WARN, "Called non-implemented fsyncdir({}, {}, _)", path, datasync);
  return 0;
}

void Fuse::init(fuse_conn_info *conn) {
  UNUSED(conn);
  const ThreadNameForDebugging _threadName("init");
  _fs = _init(this);

  ASSERT(_context != boost::none, "Context should have been initialized in Fuse::run() but somehow didn't");
  _fs->setContext(fspp::Context { *_context });

  LOG(INFO, "Filesystem started.");

  _running = true;
  _onMounted();

#ifdef FSPP_LOG
  cpputils::logging::setLevel(DEBUG);
#endif
}

void Fuse::destroy() {
  const ThreadNameForDebugging _threadName("destroy");
  _fs = make_shared<InvalidFilesystem>();
  LOG(INFO, "Filesystem stopped.");
  _running = false;
  cpputils::logging::logger()->flush();
}

int Fuse::access(const bf::path &path, int mask) {
  const ThreadNameForDebugging _threadName("access");
#ifdef FSPP_LOG
  LOG(DEBUG, "access({}, {})", path, mask);
#endif
  try {
    ASSERT(is_valid_fspp_path(path), "has to be an absolute path");
    _fs->access(path, mask);
#ifdef FSPP_LOG
    LOG(DEBUG, "access({}, {}): success", path, mask);
#endif
    return 0;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::access: {}", e.what());
    return -EIO;
  } catch (FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "access({}, {}): failed with errno {}", path, mask, e.getErrno());
#endif
    return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::create(const bf::path &path, ::mode_t mode, fuse_file_info *fileinfo) {
  const ThreadNameForDebugging _threadName("create");
#ifdef FSPP_LOG
  LOG(DEBUG, "create({}, {}, _)", path, mode);
#endif
  try {
    ASSERT(is_valid_fspp_path(path), "has to be an absolute path");
    auto context = fuse_get_context();
    fileinfo->fh = _fs->createAndOpenFile(path, mode, context->uid, context->gid);
#ifdef FSPP_LOG
    LOG(DEBUG, "create({}, {}, _): success", path, mode);
#endif
    return 0;
  } catch(const cpputils::AssertFailed &e) {
    LOG(ERR, "AssertFailed in Fuse::create: {}", e.what());
    return -EIO;
  } catch (FuseErrnoException &e) {
#ifdef FSPP_LOG
    LOG(WARN, "create({}, {}, _): failed with errno {}", path, mode, e.getErrno());
#endif
    return -e.getErrno();
  } catch(const std::exception &e) {
    _logException(e);
    return -EIO;
  } catch(...) {
    _logUnknownException();
    return -EIO;
  }
}
