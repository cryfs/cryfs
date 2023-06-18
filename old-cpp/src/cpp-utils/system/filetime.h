#pragma once
#ifndef MESSMER_CPPUTILS_SYSTEM_FILETIME_H

#include <ctime>

namespace cpputils {

int set_filetime(const char *filepath, timespec lastAccessTime, timespec lastModificationTime);
int get_filetime(const char *filepath, timespec* lastAccessTime, timespec* lastModificationTime);

}

#endif
