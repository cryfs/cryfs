#pragma once
#ifndef CRYFS_LIB_CRYDEVICE_H_
#define CRYFS_LIB_CRYDEVICE_H_

#include <messmer/blockstore/interface/BlockStore.h>
#include <messmer/blobstore/interface/BlobStore.h>
#include "CryConfig.h"

#include <boost/filesystem.hpp>
#include <messmer/fspp/fs_interface/Device.h>

#include "messmer/cpp-utils/macros.h"

namespace cryfs {
class DirBlob;

namespace bf = boost::filesystem;

class CryDevice: public fspp::Device {
public:
  static constexpr uint32_t BLOCKSIZE_BYTES = 8 * 1024;

  CryDevice(std::unique_ptr<CryConfig> config, std::unique_ptr<blockstore::BlockStore> blockStore);
  virtual ~CryDevice();

  void statfs(const boost::filesystem::path &path, struct ::statvfs *fsstat) override;

  std::unique_ptr<blobstore::Blob> CreateBlob();
  std::unique_ptr<blobstore::Blob> LoadBlob(const blockstore::Key &key);
  void RemoveBlob(const blockstore::Key &key);

private:
  blockstore::Key GetOrCreateRootKey(CryConfig *config);
  blockstore::Key CreateRootBlobAndReturnKey();
  std::unique_ptr<fspp::Node> Load(const bf::path &path) override;

  std::unique_ptr<DirBlob> LoadDirBlob(const bf::path &path);

  std::unique_ptr<blobstore::BlobStore> _blobStore;

  blockstore::Key _rootKey;

  DISALLOW_COPY_AND_ASSIGN(CryDevice);
};

}

#endif
