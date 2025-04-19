// NOMINMAX works around an MSVC issue, see https://github.com/microsoft/cppwinrt/issues/479
#if defined(_MSC_VER)
#define NOMINMAX
#endif

#include "Fuse.h"
#include <memory>
#include <cassert>

#include "../fs_interface/FuseErrnoException.h"
#include "Filesystem.h"
#include <iostream>
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/logging/logging.h>
#include <cpp-utils/process/subprocess.h>
#include <cpp-utils/thread/debugging.h>
#include <csignal>
#include "InvalidFilesystem.h"
#include <codecvt>
#include <boost/algorithm/string/replace.hpp>

#include <range/v3/view/split.hpp>
#include <range/v3/view/join.hpp>
#include <range/v3/view/filter.hpp>
#include <range/v3/range/conversion.hpp>

#if defined(_MSC_VER)
#include <dokan/dokan.h>
#endif

using std::string;
using std::vector;

namespace bf = boost::filesystem;
using namespace cpputils::logging;
using std::make_shared;
using std::shared_ptr;
using std::string;
using namespace fspp::fuse;
using cpputils::set_thread_name;

#define FUSE_OBJ (static_cast<Fuse *>(fuse_get_context()->private_data))

// Remove the following line, if you don't want to output each fuse operation on the console
// #define FSPP_LOG 1

Fuse::Fuse(std::function<shared_ptr<Filesystem>(Fuse *fuse)> init, std::function<void()> onMounted, std::string fstype, boost::optional<std::string> fsname)
    : _init(std::move(init)), _onMounted(std::move(onMounted)), _fs(make_shared<InvalidFilesystem>()), _mountdir(), _running(false), _fstype(std::move(fstype)), _fsname(std::move(fsname))
{
  ASSERT(static_cast<bool>(_init), "Invalid init given");
  ASSERT(static_cast<bool>(_onMounted), "Invalid onMounted given");
}

void Fuse::runInForeground(const bf::path &mountdir, vector<string> fuseOptions)
{
  vector<string> realFuseOptions = std::move(fuseOptions);
  if (std::find(realFuseOptions.begin(), realFuseOptions.end(), "-f") == realFuseOptions.end())
  {
    realFuseOptions.push_back("-f");
  }
  _run(mountdir, std::move(realFuseOptions));
}

void Fuse::runInBackground(const bf::path &mountdir, vector<string> fuseOptions)
{
  vector<string> realFuseOptions = std::move(fuseOptions);
  _removeAndWarnIfExists(&realFuseOptions, "-f");
  _removeAndWarnIfExists(&realFuseOptions, "-d");
  _run(mountdir, std::move(realFuseOptions));
}

void Fuse::_removeAndWarnIfExists(vector<string> *fuseOptions, const std::string &option)
{
  auto found = std::find(fuseOptions->begin(), fuseOptions->end(), option);
  if (found != fuseOptions->end())
  {
    LOG(WARN, "The fuse option {} only works when running in foreground. Removing fuse option.", option);
    do
    {
      fuseOptions->erase(found);
      found = std::find(fuseOptions->begin(), fuseOptions->end(), option);
    } while (found != fuseOptions->end());
  }
}

namespace
{
  void extractAllAtimeOptionsAndRemoveOnesUnknownToLibfuse_(string *csv_options, vector<string> *result)
  {
    const auto is_fuse_supported_atime_flag = [](const std::string &flag)
    {
      constexpr std::array<const char *, 2> flags = {"noatime", "atime"};
      return flags.end() != std::find(flags.begin(), flags.end(), flag);
    };
    const auto is_fuse_unsupported_atime_flag = [](const std::string &flag)
    {
      constexpr std::array<const char *, 3> flags = {"strictatime", "relatime", "nodiratime"};
      return flags.end() != std::find(flags.begin(), flags.end(), flag);
    };
    *csv_options = ranges::make_subrange(csv_options->begin(), csv_options->end()) | ranges::views::split(',') | ranges::views::filter([&](auto &&elem_)
                                                                                                                                       {
                // TODO string_view would be better
                std::string elem(&*elem_.begin(), ranges::distance(elem_));
                if (is_fuse_unsupported_atime_flag(elem)) {
                    result->push_back(elem);
                    return false;
                }
                if (is_fuse_supported_atime_flag(elem)) {
                    result->push_back(elem);
                }
                return true; }) |
                   ranges::views::join(',') | ranges::to<string>();
  }

  // Return a list of all atime options (e.g. atime, noatime, relatime, strictatime, nodiratime) that occur in the
  // fuseOptions input. They must be preceded by a '-o', i.e. {..., '-o', 'noatime', ...} and multiple ones can be
  // csv-concatenated, i.e. {..., '-o', 'atime,nodiratime', ...}.
  // Also, this function removes all of these atime options that are unknown to libfuse (i.e. all except atime and noatime)
  // from the input fuseOptions so we can pass it on to libfuse without crashing.
  vector<string> extractAllAtimeOptionsAndRemoveOnesUnknownToLibfuse_(vector<string> *fuseOptions)
  {
    vector<string> result;
    bool lastOptionWasDashO = false;
    for (string &option : *fuseOptions)
    {
      if (lastOptionWasDashO)
      {
        extractAllAtimeOptionsAndRemoveOnesUnknownToLibfuse_(&option, &result);
      }
      lastOptionWasDashO = (option == "-o");
    }
    return result;
  }
}

void Fuse::_run(const bf::path &mountdir, vector<string> fuseOptions)
{
#if defined(__GLIBC__) || defined(__APPLE__) || defined(_MSC_VER)
  // Avoid encoding errors for non-utf8 characters, see https://github.com/cryfs/cryfs/issues/247
  // this is ifdef'd out for non-glibc linux, because musl doesn't handle this correctly.
  bf::path::imbue(std::locale(std::locale(), new std::codecvt_utf8_utf16<wchar_t>()));
#endif

  _mountdir = mountdir;

  ASSERT(_argv.size() == 0, "Filesystem already started");

  vector<string> atimeOptions = extractAllAtimeOptionsAndRemoveOnesUnknownToLibfuse_(&fuseOptions);
  _createContext(atimeOptions);

  _argv = _build_argv(mountdir, fuseOptions);

  fuse_main(_argv.size(), _argv.data(), operations(), this);
}

void Fuse::_createContext(const vector<string> &fuseOptions)
{
  const bool has_atime_flag = fuseOptions.end() != std::find(fuseOptions.begin(), fuseOptions.end(), "atime");
  const bool has_noatime_flag = fuseOptions.end() != std::find(fuseOptions.begin(), fuseOptions.end(), "noatime");
  const bool has_relatime_flag = fuseOptions.end() != std::find(fuseOptions.begin(), fuseOptions.end(), "relatime");
  const bool has_strictatime_flag = fuseOptions.end() != std::find(fuseOptions.begin(), fuseOptions.end(), "strictatime");
  const bool has_nodiratime_flag = fuseOptions.end() != std::find(fuseOptions.begin(), fuseOptions.end(), "nodiratime");

  // Default is NOATIME, this reduces the probability for synchronization conflicts
  _context = Context(noatime());

  if (has_noatime_flag)
  {
    ASSERT(!has_atime_flag, "Cannot have both, noatime and atime flags set.");
    ASSERT(!has_relatime_flag, "Cannot have both, noatime and relatime flags set.");
    ASSERT(!has_strictatime_flag, "Cannot have both, noatime and strictatime flags set.");
    // note: can have nodiratime flag set but that is ignored because it is already included in the noatime policy.
    _context->setTimestampUpdateBehavior(noatime());
  }
  else if (has_relatime_flag)
  {
    // note: can have atime and relatime both set, they're identical
    ASSERT(!has_noatime_flag, "This shouldn't happen, or we would have hit a case above.");
    ASSERT(!has_strictatime_flag, "Cannot have both, relatime and strictatime flags set.");
    if (has_nodiratime_flag)
    {
      _context->setTimestampUpdateBehavior(nodiratime_relatime());
    }
    else
    {
      _context->setTimestampUpdateBehavior(relatime());
    }
  }
  else if (has_atime_flag)
  {
    // note: can have atime and relatime both set, they're identical
    ASSERT(!has_noatime_flag, "This shouldn't happen, or we would have hit a case above");
    ASSERT(!has_strictatime_flag, "Cannot have both, atime and strictatime flags set.");
    if (has_nodiratime_flag)
    {
      _context->setTimestampUpdateBehavior(nodiratime_relatime());
    }
    else
    {
      _context->setTimestampUpdateBehavior(relatime());
    }
  }
  else if (has_strictatime_flag)
  {
    ASSERT(!has_noatime_flag, "This shouldn't happen, or we would have hit a case above");
    ASSERT(!has_atime_flag, "This shouldn't happen, or we would have hit a case above");
    ASSERT(!has_relatime_flag, "This shouldn't happen, or we would have hit a case above");
    if (has_nodiratime_flag)
    {
      _context->setTimestampUpdateBehavior(nodiratime_strictatime());
    }
    else
    {
      _context->setTimestampUpdateBehavior(strictatime());
    }
  }
  else if (has_nodiratime_flag)
  {
    ASSERT(!has_noatime_flag, "This shouldn't happen, or we would have hit a case above");
    ASSERT(!has_atime_flag, "This shouldn't happen, or we would have hit a case above");
    ASSERT(!has_relatime_flag, "This shouldn't happen, or we would have hit a case above");
    ASSERT(!has_strictatime_flag, "This shouldn't happen, or we would have hit a case above");
    _context->setTimestampUpdateBehavior(noatime()); // use noatime by default
  }
}

vector<char *> Fuse::_build_argv(const bf::path &mountdir, const vector<string> &fuseOptions)
{
  vector<char *> argv;
  argv.reserve(6 + fuseOptions.size());                // fuseOptions + executable name + mountdir + 2x fuse options (subtype, fsname), each taking 2 entries ("-o", "key=value").
  argv.push_back(_create_c_string(_fstype));           // The first argument (executable name) is the file system type
  argv.push_back(_create_c_string(mountdir.string())); // The second argument is the mountdir
  for (const string &option : fuseOptions)
  {
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
  // argv.push_back(_create_c_string("-o"));
  // argv.push_back(_create_c_string("large_read"));
  argv.push_back(_create_c_string("-o"));
  argv.push_back(_create_c_string("big_writes"));
  return argv;
}

void Fuse::_add_fuse_option_if_not_exists(vector<char *> *argv, const string &key, const string &value)
{
  if (!_has_option(*argv, key))
  {
    argv->push_back(_create_c_string("-o"));
    argv->push_back(_create_c_string(key + "=" + value));
  }
}

bool Fuse::_has_option(const vector<char *> &vec, const string &key)
{
  // The fuse option can either be present as "-okey=value" or as "-o key=value", we have to check both.
  return _has_entry_with_prefix(key + "=", vec) || _has_entry_with_prefix("-o" + key + "=", vec);
}

bool Fuse::_has_entry_with_prefix(const string &prefix, const vector<char *> &vec)
{
  auto found = std::find_if(vec.begin(), vec.end(), [&prefix](const char *entry)
                            { return 0 == std::strncmp(prefix.c_str(), entry, prefix.size()); });
  return found != vec.end();
}

char *Fuse::_create_c_string(const string &str)
{
  // The memory allocated here is destroyed in the destructor of the Fuse class.
  char *c_str = new char[str.size() + 1];
  std::memcpy(c_str, str.c_str(), str.size() + 1);
  return c_str;
}

bool Fuse::running() const
{
  return _running;
}

void Fuse::stop()
{
  unmount(_mountdir, false);
}

void Fuse::unmount(const bf::path &mountdir, bool force)
{
  // TODO Find better way to unmount (i.e. don't use external fusermount). Unmounting by kill(getpid(), SIGINT) worked, but left the mount directory transport endpoint as not connected.
#if defined(__APPLE__)
  UNUSED(force);
  int returncode = cpputils::Subprocess::call("umount", {mountdir.string()}, "").exitcode;
#elif defined(_MSC_VER)
  UNUSED(force);
  std::wstring mountdir_ = std::wstring_convert<std::codecvt_utf8_utf16<wchar_t>>().from_bytes(mountdir.string());
  BOOL success = DokanRemoveMountPoint(mountdir_.c_str());
  int returncode = success ? 0 : -1;
#else
  std::vector<std::string> args = force ? std::vector<std::string>({"-u", mountdir.string()}) : std::vector<std::string>({"-u", "-z", mountdir.string()}); // "-z" takes care that if the filesystem can't be unmounted right now because something is opened, it will be unmounted as soon as it can be.
  int returncode = cpputils::Subprocess::call("fusermount", args, "").exitcode;
#endif
  if (returncode != 0)
  {
    throw std::runtime_error("Could not unmount filesystem");
  }
}

void Fuse::init(fuse_conn_info *conn)
{
  UNUSED(conn);
  ThreadNameForDebugging _threadName("init");
  _fs = _init(this);

  ASSERT(_context != boost::none, "Context should have been initialized in Fuse::run() but somehow didn't");
  _fs->setContext(fspp::Context{*_context});

  LOG(INFO, "Filesystem started.");

  _running = true;
  _onMounted();

#ifdef FSPP_LOG
  cpputils::logging::setLevel(DEBUG);
#endif
}

void Fuse::destroy()
{
  ThreadNameForDebugging _threadName("destroy");
  _fs = make_shared<InvalidFilesystem>();
  LOG(INFO, "Filesystem stopped.");
  _running = false;
  cpputils::logging::logger()->flush();
}

int Fuse::access(const bf::path &path, int mask)
{
  ThreadNameForDebugging _threadName("access");
#ifdef FSPP_LOG
  LOG(DEBUG, "access({}, {})", path, mask);
#endif
  try
  {
    ASSERT(is_valid_fspp_path(path), "has to be an absolute path");
    _fs->access(path, mask);
#ifdef FSPP_LOG
    LOG(DEBUG, "access({}, {}): success", path, mask);
#endif
    return 0;
  }
  catch (const cpputils::AssertFailed &e)
  {
    LOG(ERR, "AssertFailed in Fuse::access: {}", e.what());
    return -EIO;
  }
  catch (FuseErrnoException &e)
  {
#ifdef FSPP_LOG
    LOG(WARN, "access({}, {}): failed with errno {}", path, mask, e.getErrno());
#endif
    return -e.getErrno();
  }
  catch (const std::exception &e)
  {
    _logException(e);
    return -EIO;
  }
  catch (...)
  {
    _logUnknownException();
    return -EIO;
  }
}

int Fuse::create(const bf::path &path, ::mode_t mode, fuse_file_info *fileinfo)
{
  ThreadNameForDebugging _threadName("create");
#ifdef FSPP_LOG
  LOG(DEBUG, "create({}, {}, _)", path, mode);
#endif
  try
  {
    ASSERT(is_valid_fspp_path(path), "has to be an absolute path");
    auto context = fuse_get_context();
    fileinfo->fh = _fs->createAndOpenFile(path, mode, context->uid, context->gid);
#ifdef FSPP_LOG
    LOG(DEBUG, "create({}, {}, _): success", path, mode);
#endif
    return 0;
  }
  catch (const cpputils::AssertFailed &e)
  {
    LOG(ERR, "AssertFailed in Fuse::create: {}", e.what());
    return -EIO;
  }
  catch (FuseErrnoException &e)
  {
#ifdef FSPP_LOG
    LOG(WARN, "create({}, {}, _): failed with errno {}", path, mode, e.getErrno());
#endif
    return -e.getErrno();
  }
  catch (const std::exception &e)
  {
    _logException(e);
    return -EIO;
  }
  catch (...)
  {
    _logUnknownException();
    return -EIO;
  }
}
