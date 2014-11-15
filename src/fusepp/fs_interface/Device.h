#pragma once
#ifndef FUSEPP_DEVICE_H_
#define FUSEPP_DEVICE_H_

#include <boost/filesystem.hpp>
#include <memory>
#include <sys/statvfs.h>

namespace fusepp {
class Node;

class Device {
public:
	virtual ~Device() {}

	virtual void statfs(const boost::filesystem::path &path, struct ::statvfs *fsstat) = 0;
	virtual std::unique_ptr<Node> Load(const boost::filesystem::path &path) = 0;
};

}

#endif /* FUSEPP_DEVICE_H_ */
