#if defined(_MSC_VER)

#include "filetime.h"
#include <Windows.h>
#include <stdexcept>

namespace cpputils {

namespace {
FILETIME to_filetime(timespec value) {
  ULARGE_INTEGER ull;
  ull.QuadPart = (value.tv_sec * 10000000ULL) + 116444736000000000ULL + (value.tv_nsec / 100);
  FILETIME result;
  result.dwLowDateTime = ull.LowPart;
  result.dwHighDateTime = ull.HighPart;
  return result;
}

timespec to_timespec(FILETIME value) {
	// function taken from https://github.com/wishstudio/flinux/blob/afbb7d7509d4f1e0fdd62bf02124e7c4ce20ca6d/src/datetime.c under GPLv3
	constexpr uint64_t NANOSECONDS_PER_TICK = 100ULL;
	constexpr uint64_t NANOSECONDS_PER_SECOND = 1000000000ULL;
	constexpr uint64_t TICKS_PER_SECOND = 10000000ULL;
	constexpr uint64_t SEC_TO_UNIX_EPOCH = 11644473600ULL;
	constexpr uint64_t TICKS_TO_UNIX_EPOCH = TICKS_PER_SECOND * SEC_TO_UNIX_EPOCH;
	
	uint64_t ticks = ((uint64_t)value.dwHighDateTime << 32ULL) + value.dwLowDateTime;
	uint64_t nsec = 0;
	if (ticks >= TICKS_TO_UNIX_EPOCH) { // otherwise out of range
		ticks -= TICKS_TO_UNIX_EPOCH;
		nsec = ticks * NANOSECONDS_PER_TICK;
	}

	timespec result;
	result.tv_sec = nsec / NANOSECONDS_PER_SECOND;
	result.tv_nsec = nsec % NANOSECONDS_PER_SECOND;
	return result;
}

struct OpenFileRAII final {
	HANDLE handle;

	OpenFileRAII(const char* filepath, DWORD access)
			:handle(CreateFileA(filepath, access, 0, nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr)) {
	}

	BOOL close() {
		if (INVALID_HANDLE_VALUE == handle) {
			return 1;
		}

		BOOL success = CloseHandle(handle);
		handle = INVALID_HANDLE_VALUE;
		return success;
	}

	~OpenFileRAII() {
		close();
	}
};

}

int set_filetime(const char *filepath, timespec lastAccessTime, timespec lastModificationTime) {
	OpenFileRAII file(filepath, FILE_WRITE_ATTRIBUTES);
	if (INVALID_HANDLE_VALUE == file.handle) {
		return GetLastError();
	}

	FILETIME atime = to_filetime(lastAccessTime);
	FILETIME mtime = to_filetime(lastModificationTime);
		
	BOOL success = SetFileTime(file.handle, nullptr, &atime, &mtime);
	if (!success) {
		return GetLastError();
	}

	success = file.close();
	if (!success) {
		return GetLastError();
	}

	return 0;
}

int get_filetime(const char *filepath, timespec* lastAccessTime, timespec* lastModificationTime) {
	OpenFileRAII file(filepath, FILE_READ_ATTRIBUTES);
	if (INVALID_HANDLE_VALUE == file.handle) {
		return GetLastError();
	}

	FILETIME atime;
	FILETIME mtime;

	BOOL success = GetFileTime(file.handle, nullptr, &atime, &mtime);
	if (!success) {
		return GetLastError();
	}

	success = file.close();
	if (!success) {
		return GetLastError();
	}

	*lastAccessTime = to_timespec(atime);
	*lastModificationTime = to_timespec(mtime);

	return 0;
}

}

#endif
