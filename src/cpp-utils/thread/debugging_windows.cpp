#if defined(_MSC_VER)

#include <Windows.h>
#include "debugging.h"
#include <codecvt>
#include <cpp-utils/assert/assert.h>

using std::string;
using std::wstring;
using std::wstring_convert;

namespace cpputils {

namespace {
struct NameData final {
  wchar_t *name = nullptr;

  ~NameData() {
    if (nullptr != LocalFree(name)) {
      throw std::runtime_error("Error releasing thread description memory. Error code: " + std::to_string(GetLastError()));
    }
  }
};

struct ModuleHandle final {
	HMODULE module;

	ModuleHandle(const char* dll) {
		bool success = GetModuleHandleExA(0, dll, &module);
		if (!success) {
			throw std::runtime_error(string() + "Error loading dll: " + dll + ". Error code: " + std::to_string(GetLastError()));
		}
	}

	~ModuleHandle() {
		bool success = FreeLibrary(module);
		if (!success) {
			throw std::runtime_error("Error unloading dll. Error code: " + std::to_string(GetLastError()));
		}
	}
};
template<class Fn>
class APIFunction final {
private:
	ModuleHandle module_;
	Fn func_;

public:
	APIFunction(const char* dll, const char* function)
	: module_(dll), func_(reinterpret_cast<Fn>(GetProcAddress(module_.module, function))) {
	}

	bool valid() const {
		return func_ != nullptr;
	}

	Fn func() const {
		return func_;
	}
};

std::string get_thread_name(HANDLE thread) {
  // The GetThreadDescription API was brought in version 1607 of Windows 10.
  typedef HRESULT(WINAPI* GetThreadDescriptionFn)(HANDLE hThread, PWSTR* ppszThreadDescription);
  static APIFunction<GetThreadDescriptionFn> get_thread_description_func("Kernel32.dll", "GetThreadDescription");

  if (get_thread_description_func.valid()) {
	  NameData name_data;

	  HRESULT status = get_thread_description_func.func()(thread, &name_data.name);
	  if (FAILED(status)) {
		throw std::runtime_error("Error getting thread description. Error code: " + std::to_string(status));
	  }
	  return wstring_convert<std::codecvt_utf8_utf16<wchar_t>>().to_bytes(name_data.name);
  }
  else {
	  // GetThreadDescription API is not available.
	  return "";
  }
}
}

void set_thread_name(const char* name) {
  // The GetThreadDescription API was brought in version 1607 of Windows 10.
  typedef HRESULT(WINAPI* SetThreadDescriptionFn)(HANDLE hThread, PCWSTR lpThreadDescription);
  static APIFunction<SetThreadDescriptionFn> set_thread_description_func("Kernel32.dll", "SetThreadDescription");

  if (set_thread_description_func.valid()) {
	  wstring wname = wstring_convert<std::codecvt_utf8_utf16<wchar_t>>().from_bytes(name);
	  HRESULT status = set_thread_description_func.func()(GetCurrentThread(), wname.c_str());
	  if (FAILED(status)) {
		  throw std::runtime_error("Error setting thread description. Error code: " + std::to_string(status));
	  }
  }
  else {
	  // intentionally empty. SetThreadDescription API is not available.
  }
}

std::string get_thread_name() {
  return get_thread_name(GetCurrentThread());
}

std::string get_thread_name(std::thread* thread) {
  ASSERT(thread->joinable(), "Thread not running");
  return get_thread_name(static_cast<HANDLE>(thread->native_handle()));
}

}

#endif
