#pragma once
#ifndef MESSMER_FSPP_FUSE_FUSE_H_
#define MESSMER_FSPP_FUSE_FUSE_H_

#include "params.h"
#include <cstdio>
#include <string>
#include <vector>
#include <sys/stat.h>
#include <boost/filesystem.hpp>
#include <boost/optional.hpp>
#include <cpp-utils/macros.h>
#include <atomic>
#include "stat_compatibility.h"

namespace fspp {
class Device;

namespace fuse {
class Filesystem;

class Fuse final {
public:
  explicit Fuse(std::function<std::shared_ptr<Filesystem> (Fuse *fuse)> init, std::function<void()> onMounted, std::string fstype, boost::optional<std::string> fsname);
  ~Fuse();

  void run(const boost::filesystem::path &mountdir, const std::vector<std::string> &fuseOptions);
  bool running() const;
  void stop();

  static void unmount(const boost::filesystem::path &mountdir, bool force = false);

  int getattr(const boost::filesystem::path &path, fspp::fuse::STAT *stbuf);
  int fgetattr(const boost::filesystem::path &path, fspp::fuse::STAT *stbuf, fuse_file_info *fileinfo);
  int readlink(const boost::filesystem::path &path, char *buf, size_t size);
  int mknod(const boost::filesystem::path &path, ::mode_t mode, dev_t rdev);
  int mkdir(const boost::filesystem::path &path, ::mode_t mode);
  int unlink(const boost::filesystem::path &path);
  int rmdir(const boost::filesystem::path &path);
  int symlink(const boost::filesystem::path &from, const boost::filesystem::path &to);
  int rename(const boost::filesystem::path &from, const boost::filesystem::path &to);
  int link(const boost::filesystem::path &from, const boost::filesystem::path &to);
  int chmod(const boost::filesystem::path &path, ::mode_t mode);
  int chown(const boost::filesystem::path &path, ::uid_t uid, ::gid_t gid);
  int truncate(const boost::filesystem::path &path, int64_t size);
  int ftruncate(const boost::filesystem::path &path, int64_t size, fuse_file_info *fileinfo);
  int utimens(const boost::filesystem::path &path, const std::array<timespec, 2> times);
  int open(const boost::filesystem::path &path, fuse_file_info *fileinfo);
  int release(const boost::filesystem::path &path, fuse_file_info *fileinfo);
  int read(const boost::filesystem::path &path, char *buf, size_t size, int64_t offset, fuse_file_info *fileinfo);
  int write(const boost::filesystem::path &path, const char *buf, size_t size, int64_t offset, fuse_file_info *fileinfo);
  int statfs(const boost::filesystem::path &path, struct ::statvfs *fsstat);
  int flush(const boost::filesystem::path &path, fuse_file_info *fileinfo);
  int fsync(const boost::filesystem::path &path, int flags, fuse_file_info *fileinfo);
  int opendir(const boost::filesystem::path &path, fuse_file_info *fileinfo);
  int readdir(const boost::filesystem::path &path, void *buf, fuse_fill_dir_t filler, int64_t offset, fuse_file_info *fileinfo);
  int releasedir(const boost::filesystem::path &path, fuse_file_info *fileinfo);
  int fsyncdir(const boost::filesystem::path &path, int datasync, fuse_file_info *fileinfo);
  void init(fuse_conn_info *conn);
  void destroy();
  int access(const boost::filesystem::path &path, int mask);
  int create(const boost::filesystem::path &path, ::mode_t mode, fuse_file_info *fileinfo);

private:
  static void _logException(const std::exception &e);
  static void _logUnknownException();
  static char *_create_c_string(const std::string &str);
  static bool _has_option(const std::vector<char *> &vec, const std::string &key);
  static bool _has_entry_with_prefix(const std::string &prefix, const std::vector<char *> &vec);
  std::vector<char *> _build_argv(const boost::filesystem::path &mountdir, const std::vector<std::string> &fuseOptions);
  void _add_fuse_option_if_not_exists(std::vector<char *> *argv, const std::string &key, const std::string &value);

  std::function<std::shared_ptr<Filesystem> (Fuse *fuse)> _init;
  std::function<void()> _onMounted;
  std::shared_ptr<Filesystem> _fs;
  boost::filesystem::path _mountdir;
  std::vector<char*> _argv;
  std::atomic<bool> _running;
  std::string _fstype;
  boost::optional<std::string> _fsname;

  DISALLOW_COPY_AND_ASSIGN(Fuse);
};
}
}

#endif
