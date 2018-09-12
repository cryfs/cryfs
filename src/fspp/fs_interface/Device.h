#pragma once
#ifndef MESSMER_FSPP_FSINTERFACE_DEVICE_H_
#define MESSMER_FSPP_FSINTERFACE_DEVICE_H_

#include <boost/filesystem.hpp>
#include <cpp-utils/pointer/unique_ref.h>

namespace fspp {
class Node;
class File;
class Dir;
class Symlink;

class Device {
public:
	virtual ~Device() {}

	struct statvfs final {
		uint32_t max_filename_length;

		uint32_t blocksize;
		uint64_t num_total_blocks;
		uint64_t num_free_blocks;
		uint64_t num_available_blocks; // free blocks for unprivileged users

		uint64_t num_total_inodes;
		uint64_t num_free_inodes;
		uint64_t num_available_inodes; // free inodes for unprivileged users
	};

	virtual statvfs statfs(const boost::filesystem::path &path) = 0;
	virtual boost::optional<cpputils::unique_ref<Node>> Load(const boost::filesystem::path &path) = 0;

	//TODO Test default implementation (Device.cpp)
	//TODO Test client implementation (fstest)
	//TODO When it exists but is wrong node type, don't throw exception, but return this somehow (or at least throw specific exception, not just FuseErrnoException)
	virtual boost::optional<cpputils::unique_ref<File>> LoadFile(const boost::filesystem::path &path) = 0;
	virtual boost::optional<cpputils::unique_ref<Dir>> LoadDir(const boost::filesystem::path &path) = 0;
	virtual boost::optional<cpputils::unique_ref<Symlink>> LoadSymlink(const boost::filesystem::path &path) = 0;

};

}

#endif
