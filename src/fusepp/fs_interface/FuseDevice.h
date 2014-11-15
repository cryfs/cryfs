#pragma once
#ifndef FUSEPP_FUSEDEVICE_H_
#define FUSEPP_FUSEDEVICE_H_

#include <boost/filesystem.hpp>
#include <memory>
#include <sys/statvfs.h>

namespace fusepp {
class FuseNode;

class FuseDevice {
public:
	virtual ~FuseDevice() {}

	virtual void statfs(const boost::filesystem::path &path, struct ::statvfs *fsstat) = 0;
	virtual std::unique_ptr<FuseNode> Load(const boost::filesystem::path &path) = 0;
};

}

#endif /* FUSEPP_FUSEDEVICE_H_ */
