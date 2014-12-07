#pragma once
#ifndef CRYFS_LIB_CRYDEVICE_H_
#define CRYFS_LIB_CRYDEVICE_H_

#include "CryConfig.h"

#include <boost/filesystem.hpp>
#include <fspp/fs_interface/Device.h>

#include "blobstore/interface/BlobStore.h"

#include "fspp/utils/macros.h"

namespace cryfs {

namespace bf = boost::filesystem;

class CryDevice: public fspp::Device {
public:
  static constexpr size_t DIR_BLOBSIZE = 4096;

  CryDevice(std::unique_ptr<CryConfig> config, std::unique_ptr<blobstore::BlobStore> blobStore);
  virtual ~CryDevice();

  void statfs(const boost::filesystem::path &path, struct ::statvfs *fsstat) override;

  blobstore::BlobWithKey CreateBlob(size_t size);

private:
  std::string CreateRootBlobAndReturnKey();
  std::unique_ptr<fspp::Node> Load(const bf::path &path) override;

  std::unique_ptr<blobstore::BlobStore> _blob_store;

  std::string _root_key;

  DISALLOW_COPY_AND_ASSIGN(CryDevice);
};

}

#endif
