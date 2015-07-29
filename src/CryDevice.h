#pragma once
#ifndef CRYFS_LIB_CRYDEVICE_H_
#define CRYFS_LIB_CRYDEVICE_H_

#include <messmer/blockstore/interface/BlockStore.h>
#include <messmer/blobstore/interface/BlobStore.h>
#include "CryConfigLoader.h"

#include <boost/filesystem.hpp>
#include <messmer/fspp/fs_interface/Device.h>

#include "messmer/cpp-utils/macros.h"

namespace cryfs {
class DirBlob;

class CryDevice: public fspp::Device {
public:
  static constexpr uint32_t BLOCKSIZE_BYTES = 32 * 1024;

  CryDevice(cpputils::unique_ref<CryConfig> config, cpputils::unique_ref<blockstore::BlockStore> blockStore);
  virtual ~CryDevice();

  void statfs(const boost::filesystem::path &path, struct ::statvfs *fsstat) override;

  cpputils::unique_ref<blobstore::Blob> CreateBlob();
  boost::optional<cpputils::unique_ref<blobstore::Blob>> LoadBlob(const blockstore::Key &key);
  void RemoveBlob(const blockstore::Key &key);

  boost::optional<cpputils::unique_ref<fspp::Node>> Load(const boost::filesystem::path &path) override;

  boost::optional<cpputils::unique_ref<DirBlob>> LoadDirBlob(const boost::filesystem::path &path);

private:

  cpputils::unique_ref<blobstore::BlobStore> _blobStore;

  blockstore::Key _rootKey;

  blockstore::Key GetOrCreateRootKey(CryConfig *config);
  blockstore::Key CreateRootBlobAndReturnKey();
  static cpputils::unique_ref<blockstore::BlockStore> CreateEncryptedBlockStore(const CryConfig &config, cpputils::unique_ref<blockstore::BlockStore> baseBlockStore);

  DISALLOW_COPY_AND_ASSIGN(CryDevice);
};

}

#endif
