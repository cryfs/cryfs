#pragma once
#ifndef CRYFS_LIB_CRYDEVICE_H_
#define CRYFS_LIB_CRYDEVICE_H_

#include <messmer/blockstore/interface/BlockStore.h>
#include "../config/CryConfigLoader.h"

#include <boost/filesystem.hpp>
#include <messmer/fspp/fs_interface/Device.h>

#include "fsblobstore/FsBlobStore.h"
#include "fsblobstore/DirBlob.h"
#include "fsblobstore/FileBlob.h"
#include "fsblobstore/SymlinkBlob.h"

namespace cryfs {

class CryDevice: public fspp::Device {
public:
  static constexpr uint32_t BLOCKSIZE_BYTES = 32 * 1024;

  CryDevice(cpputils::unique_ref<CryConfig> config, cpputils::unique_ref<blockstore::BlockStore> blockStore);
  virtual ~CryDevice();

  void statfs(const boost::filesystem::path &path, struct ::statvfs *fsstat) override;

  cpputils::unique_ref<fsblobstore::FileBlob> CreateFileBlob();
  cpputils::unique_ref<fsblobstore::DirBlob> CreateDirBlob();
  cpputils::unique_ref<fsblobstore::SymlinkBlob> CreateSymlinkBlob(const boost::filesystem::path &target);
  cpputils::unique_ref<fsblobstore::FsBlob> LoadBlob(const blockstore::Key &key); //TODO Do I still need this function?
  cpputils::unique_ref<fsblobstore::FsBlob> LoadBlob(const boost::filesystem::path &path);
  cpputils::unique_ref<fsblobstore::DirBlob> LoadDirBlob(const boost::filesystem::path &path);
  void RemoveBlob(const blockstore::Key &key);

  boost::optional<cpputils::unique_ref<fspp::Node>> Load(const boost::filesystem::path &path) override;


private:

  cpputils::unique_ref<fsblobstore::FsBlobStore> _fsBlobStore;

  blockstore::Key _rootKey;

  blockstore::Key GetOrCreateRootKey(CryConfig *config);
  blockstore::Key CreateRootBlobAndReturnKey();
  static cpputils::unique_ref<blockstore::BlockStore> CreateEncryptedBlockStore(const CryConfig &config, cpputils::unique_ref<blockstore::BlockStore> baseBlockStore);

  DISALLOW_COPY_AND_ASSIGN(CryDevice);
};

}

#endif
