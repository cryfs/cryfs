#pragma once
#ifndef CRYFS_LIB_CRYDEVICE_H_
#define CRYFS_LIB_CRYDEVICE_H_

#include <boost/filesystem.hpp>
#include <fspp/fs_interface/Device.h>

#include "blobstore/interface/BlobStore.h"

#include "fspp/utils/macros.h"

namespace cryfs {

namespace bf = boost::filesystem;

class CryDevice: public fspp::Device {
public:
  CryDevice(std::unique_ptr<blobstore::BlobStore> blobStore);
  virtual ~CryDevice();

  void statfs(const boost::filesystem::path &path, struct ::statvfs *fsstat) override;

private:
  std::unique_ptr<fspp::Node> Load(const bf::path &path) override;

  std::unique_ptr<blobstore::BlobStore> _blobStore;

  DISALLOW_COPY_AND_ASSIGN(CryDevice);
};

}

#endif
