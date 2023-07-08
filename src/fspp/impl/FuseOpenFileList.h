#pragma once
#ifndef MESSMER_FSPP_IMPL_FUSEOPENFILELIST_H_
#define MESSMER_FSPP_IMPL_FUSEOPENFILELIST_H_

#include "../fs_interface/File.h"
#include "../fs_interface/OpenFile.h"
#include "../fs_interface/FuseErrnoException.h"
#include <cpp-utils/macros.h>
#include <cpp-utils/assert/assert.h>
#include "IdList.h"
#include <condition_variable>

namespace fspp {
namespace detail {
class OnScopeExit final {
public:
	explicit OnScopeExit(std::function<void()> handler)
		: _handler(std::move(handler)) {}

	~OnScopeExit() {
		_handler();
	}

private:
	std::function<void()> _handler;
};
}

class FuseOpenFileList final {
public:
  FuseOpenFileList();
  ~FuseOpenFileList();

  int open(cpputils::unique_ref<OpenFile> file);
  template<class Func>
  auto load(int descriptor, Func&& callback);
  void close(int descriptor);

private:
  IdList<OpenFile> _open_files;

  std::unordered_map<int, size_t> _refcounts;
  std::mutex _mutex;

  std::condition_variable _refcount_zero_cv;

  DISALLOW_COPY_AND_ASSIGN(FuseOpenFileList);
};

inline FuseOpenFileList::FuseOpenFileList()
  :_open_files(), _refcounts(), _mutex(), _refcount_zero_cv() {
}

inline FuseOpenFileList::~FuseOpenFileList() {
	std::unique_lock<std::mutex> lock(_mutex);

	// Wait until all pending requests are done
	_refcount_zero_cv.wait(lock, [&] {
		for (const auto& refcount : _refcounts) {
			if (0 != refcount.second) {
				return false;
			}
		}
		return true;
	});

	// There might still be open files when the file system is shutdown, so we can't assert it's empty.
	// But to check that _refcounts has been updated correctly, we can assert the invariant that we have as many
	// refcounts as open files.
	ASSERT(_refcounts.size() == _refcounts.size(), "Didn't clean up refcounts properly");
}

inline int FuseOpenFileList::open(cpputils::unique_ref<OpenFile> file) {
  const std::lock_guard<std::mutex> lock(_mutex);

  const int descriptor = _open_files.add(std::move(file));
  _refcounts.emplace(descriptor, 0);
  return descriptor;
}

template<class Func>
inline auto FuseOpenFileList::load(int descriptor, Func&& callback) {
  try {
    std::unique_lock<std::mutex> lock(_mutex);
	_refcounts.at(descriptor) += 1;
	const detail::OnScopeExit _([&] {
		if (!lock.owns_lock()) { // own_lock can be true when _open_files.get() below fails before the lock is unlocked
		  lock.lock();
		}
		_refcounts.at(descriptor) -= 1;
		_refcount_zero_cv.notify_all();
	});

	OpenFile* loaded = _open_files.get(descriptor);
	lock.unlock();

	return std::forward<Func>(callback)(loaded);
  } catch (const std::out_of_range& e) {
    throw fspp::fuse::FuseErrnoException(EBADF);
  }
}

inline void FuseOpenFileList::close(int descriptor) {
  std::unique_lock<std::mutex> lock(_mutex);

  _refcount_zero_cv.wait(lock, [&] {
	  return 0 == _refcounts.at(descriptor);
  });

  //The destructor of the stored FuseOpenFile closes the file
  _open_files.remove(descriptor);
  _refcounts.erase(descriptor);
}

}

#endif
