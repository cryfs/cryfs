#include "homedir.h"
#include <sys/types.h>

namespace bf = boost::filesystem;
using std::string;

#if !defined(_MSC_VER)

#include <pwd.h>
namespace {
	bf::path _get_home_directory() {
		const char* homedir_ = getenv("HOME");
		string homedir = (homedir_ == nullptr) ? "" : homedir_;
		if (homedir == "") {
			// try the /etc/passwd entry
			struct passwd* pwd = getpwuid(getuid());
			if (pwd) {
				homedir = pwd->pw_dir;
			}
		}
		if (homedir == "") {
			throw std::runtime_error("Couldn't determine home directory for user");
		}
		return homedir;
	}

	bf::path _get_appdata_directory() {
		const char* xdg_data_dir = std::getenv("XDG_DATA_HOME");
		if (xdg_data_dir != nullptr) {
			return xdg_data_dir;
		}

		return _get_home_directory() / ".local" / "share";
	}
}

#else

#include <Shlobj.h>
namespace {
	struct PathBuffer final {
		PWSTR path = nullptr;

		~PathBuffer() {
			CoTaskMemFree(path);
		}
	};

	bf::path _get_known_path(KNOWNFOLDERID folderId) {
		PathBuffer path;
		HRESULT result_code = ::SHGetKnownFolderPath(folderId, 0, nullptr, &path.path);
		if (S_OK != result_code) {
			throw std::runtime_error("Failed getting user home directory. Hresult: " + std::to_string(result_code));
		}
		bf::path result(path.path);
		return result;
	}

	bf::path _get_home_directory() {
		return _get_known_path(FOLDERID_Profile);
	}

	bf::path _get_appdata_directory() {
		return _get_known_path(FOLDERID_LocalAppData);
	}
}

#endif

namespace bf = boost::filesystem;
using std::string;

namespace cpputils {
    namespace system {

        HomeDirectory::HomeDirectory()
             : _home_directory(::_get_home_directory())
			 , _appdata_directory(::_get_appdata_directory()) {
        }

        HomeDirectory &HomeDirectory::singleton() {
            static HomeDirectory _singleton;
            return _singleton;
        }

        const bf::path &HomeDirectory::get() {
            return singleton()._home_directory;
        }

        const bf::path &HomeDirectory::getXDGDataDir() {
			return singleton()._appdata_directory;
        }

        FakeHomeDirectoryRAII::FakeHomeDirectoryRAII(const bf::path& fakeHomeDirectory, const bf::path& fakeAppdataDirectory)
                : _oldHomeDirectory(HomeDirectory::singleton()._home_directory)
		        , _oldAppdataDirectory(HomeDirectory::singleton()._appdata_directory) {
            HomeDirectory::singleton()._home_directory = fakeHomeDirectory;
			HomeDirectory::singleton()._appdata_directory = fakeAppdataDirectory;
        }

        FakeHomeDirectoryRAII::~FakeHomeDirectoryRAII() {
            // Reset to old (non-fake) value
            HomeDirectory::singleton()._home_directory = _oldHomeDirectory;
			HomeDirectory::singleton()._appdata_directory = _oldAppdataDirectory;
        }

		FakeTempHomeDirectoryRAII::FakeTempHomeDirectoryRAII()
			: _tempDir(), _fakeHome(_tempDir.path() / "home", _tempDir.path() / "appdata") {}
    }
}
