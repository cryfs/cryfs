#pragma once
#ifndef MESSMER_CPPUTILS_SYSTEM_PATH_H
#define MESSMER_CPPUTILS_SYSTEM_PATH_H

#include <boost/filesystem/path.hpp>
#include <cpp-utils/macros.h>

namespace cpputils {

#if defined(_MSC_VER)

inline bool path_is_just_drive_letter(const boost::filesystem::path& path) {
    return path.has_root_path() && !path.has_root_directory() && !path.has_parent_path();
}

#else

inline constexpr bool path_is_just_drive_letter(const boost::filesystem::path& /*path*/) {
    return false;
}

#endif

}

#endif
