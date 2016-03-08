#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYDEVICE_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYDEVICE_H_

#include <blockstore/interface/BlockStore.h>
#include "../config/CryConfigFile.h"

#include <boost/filesystem.hpp>
#include <fspp/fs_interface/Device.h>

#include "parallelaccessfsblobstore/ParallelAccessFsBlobStore.h"
#include "parallelaccessfsblobstore/DirBlobRef.h"
#include "parallelaccessfsblobstore/FileBlobRef.h"
#include "parallelaccessfsblobstore/SymlinkBlobRef.h"

namespace cryfs {

class CryDevice final: public fspp::Device {
public:
  CryDevice(CryConfigFile config, cpputils::unique_ref<blockstore::BlockStore> blockStore);

  void statfs(const boost::filesystem::path &path, struct ::statvfs *fsstat) override;

  cpputils::unique_ref<parallelaccessfsblobstore::FileBlobRef> CreateFileBlob();
  cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef> CreateDirBlob();
  cpputils::unique_ref<parallelaccessfsblobstore::SymlinkBlobRef> CreateSymlinkBlob(const boost::filesystem::path &target);
  cpputils::unique_ref<parallelaccessfsblobstore::FsBlobRef> LoadBlob(const blockstore::Key &key); //TODO Do I still need this function?
  cpputils::unique_ref<parallelaccessfsblobstore::FsBlobRef> LoadBlob(const boost::filesystem::path &path);
  cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef> LoadDirBlob(const boost::filesystem::path &path);
  void RemoveBlob(const blockstore::Key &key);

  void onFsAction(std::function<void()> callback);

  boost::optional<cpputils::unique_ref<fspp::Node>> Load(const boost::filesystem::path &path) override;

  const CryConfig &config() const;
  void callFsActionCallbacks() const;

private:

  cpputils::unique_ref<parallelaccessfsblobstore::ParallelAccessFsBlobStore> _fsBlobStore;

  blockstore::Key _rootKey;
  CryConfigFile _configFile;
  std::vector<std::function<void()>> _onFsAction;

  blockstore::Key GetOrCreateRootKey(CryConfigFile *config);
  blockstore::Key CreateRootBlobAndReturnKey();
  static cpputils::unique_ref<blockstore::BlockStore> CreateEncryptedBlockStore(const CryConfig &config, cpputils::unique_ref<blockstore::BlockStore> baseBlockStore);

  DISALLOW_COPY_AND_ASSIGN(CryDevice);
};

}

#endif
