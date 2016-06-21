#pragma once
#ifndef MESSMER_CPPUTILS_SYSTEM_GETTOTALMEMORY_H
#define MESSMER_CPPUTILS_SYSTEM_GETTOTALMEMORY_H

#include <boost/filesystem/path.hpp>

namespace cpputils {
    namespace system {

        boost::filesystem::path home_directory();

    }
}

#endif
