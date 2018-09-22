#pragma once
#ifndef MESSMER_FSPP_FSINTERFACE_DEVICE_H_
#define MESSMER_FSPP_FSINTERFACE_DEVICE_H_

#include <boost/filesystem.hpp>
#include <cpp-utils/pointer/unique_ref.h>
#include "Types.h"

namespace fspp {
class Node;
class File;
class Dir;
class Symlink;

class Device {
public:
	virtual ~Device() {}

	using statvfs = fspp::statvfs;

	virtual statvfs statfs() = 0;
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
