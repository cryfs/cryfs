#include "env.h"

#include <stdexcept>
#include <string>
#include <cerrno>

#if !defined(_MSC_VER)

#include <cstdlib>
namespace cpputils {

void setenv(const char* key, const char* value) {
	const int retval = ::setenv(key, value, 1);
	if (0 != retval) {
		throw std::runtime_error("Error setting environment variable. Errno: " + std::to_string(errno));
	}
}

void unsetenv(const char* key) {
	const int retval = ::unsetenv(key);
	if (0 != retval) {
		throw std::runtime_error("Error unsetting environment variable. Errno: " + std::to_string(errno));
	}
}

}

#else

#include <Windows.h>
#include <sstream>
namespace cpputils {

void setenv(const char* key, const char* value) {
	std::ostringstream command;
	command << key << "=" << value;
	const int retval = _putenv(command.str().c_str());
	if (0 != retval) {
		throw std::runtime_error("Error setting environment variable. Errno: " + std::to_string(errno));
	}
}

void unsetenv(const char* key) {
	setenv(key, "");
}

}

#endif
