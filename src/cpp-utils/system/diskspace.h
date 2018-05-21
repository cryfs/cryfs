#pragma once
#ifndef MESSMER_CPPUTILS_SYSTEM_DISKSPACE_H
#define MESSMER_CPPUTILS_SYSTEM_DISKSPACE_H

#include <cstdlib>
#include <boost/filesystem/path.hpp>

namespace cpputils {
	// TODO Test
	uint64_t free_disk_space_in_bytes(const boost::filesystem::path& location);

}

#endif
