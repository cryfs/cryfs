#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYDEVICE_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYDEVICE_H_

#include <blockstore/interface/BlockStore.h>
#include <blockstore/interface/BlockStore2.h>
#include "cryfs/impl/config/CryConfigFile.h"

#include <boost/filesystem.hpp>
#include <fspp/fs_interface/Device.h>
#include <cryfs/impl/localstate/LocalStateDir.h>

#include "cryfs/impl/filesystem/parallelaccessfsblobstore/ParallelAccessFsBlobStore.h"
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/DirBlobRef.h"
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/FileBlobRef.h"
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/SymlinkBlobRef.h"


namespace cryfs {

class CryDevice final: public fspp::Device {
public:
  CryDevice(std::shared_ptr<CryConfigFile> config, const boost::filesystem::path& basedir, const LocalStateDir& localStateDir, uint32_t myClientId, bool allowIntegrityViolations, bool missingBlockIsIntegrityViolation, std::function<void ()> onIntegrityViolation);

  // Only for tests: Create a CryDevice with a fake block store
  CryDevice(std::shared_ptr<CryConfigFile> config, const LocalStateDir& localStateDir, uint32_t myClientId, bool allowIntegrityViolations, bool missingBlockIsIntegrityViolation, std::function<void ()> onIntegrityViolation);

  statvfs statfs() override;

  cpputils::unique_ref<parallelaccessfsblobstore::FileBlobRef> CreateFileBlob(const blockstore::BlockId &parent);
  cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef> CreateDirBlob(const blockstore::BlockId &parent);
  cpputils::unique_ref<parallelaccessfsblobstore::SymlinkBlobRef> CreateSymlinkBlob(const boost::filesystem::path &target, const blockstore::BlockId &parent);
  cpputils::unique_ref<parallelaccessfsblobstore::FsBlobRef> LoadBlob(const blockstore::BlockId &blockId);
  struct DirBlobWithAncestors {
    cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef> blob;
    boost::optional<cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef>> parent;
  };
  boost::optional<DirBlobWithAncestors> LoadDirBlobWithAncestors(const boost::filesystem::path &path, std::function<void (const blockstore::BlockId&)> ancestor_callback);
  void RemoveBlob(const blockstore::BlockId &blockId);

  void onFsAction(std::function<void()> callback);

  boost::optional<cpputils::unique_ref<fspp::Node>> Load(const boost::filesystem::path &path) override;
  boost::optional<cpputils::unique_ref<fspp::File>> LoadFile(const boost::filesystem::path &path) override;
  boost::optional<cpputils::unique_ref<fspp::Dir>> LoadDir(const boost::filesystem::path &path) override;
  boost::optional<cpputils::unique_ref<fspp::Symlink>> LoadSymlink(const boost::filesystem::path &path) override;

  const CryConfig &config() const;
  void callFsActionCallbacks() const;

  uint64_t numBlocks() const;

private:

  cpputils::unique_ref<parallelaccessfsblobstore::ParallelAccessFsBlobStore> _fsBlobStore;

  blockstore::BlockId _rootBlobId;
  std::shared_ptr<CryConfigFile> _configFile;
  std::vector<std::function<void()>> _onFsAction;

  blockstore::BlockId GetOrCreateRootBlobId(CryConfigFile *config);
  blockstore::BlockId CreateRootBlobAndReturnId();
  static cpputils::unique_ref<parallelaccessfsblobstore::ParallelAccessFsBlobStore> CreateFsBlobStore(cpputils::unique_ref<blobstore::BlobStore> blobStore, CryConfigFile *configFile);
#ifndef CRYFS_NO_COMPATIBILITY
  static cpputils::unique_ref<fsblobstore::FsBlobStore> MigrateOrCreateFsBlobStore(cpputils::unique_ref<blobstore::BlobStore> blobStore, CryConfigFile *configFile);
#endif
  static cpputils::unique_ref<blobstore::BlobStore> CreateBlobStore(const boost::filesystem::path& basedir, const LocalStateDir& localStateDir, CryConfigFile *configFile, uint32_t myClientId, bool allowIntegrityViolations, bool missingBlockIsIntegrityViolation, std::function<void()> onIntegrityViolation);
  static cpputils::unique_ref<blobstore::BlobStore> CreateFakeBlobStore(const LocalStateDir& localStateDir, CryConfigFile *configFile, uint32_t myClientId, bool allowIntegrityViolations, bool missingBlockIsIntegrityViolation, std::function<void()> onIntegrityViolation);
  //static cpputils::unique_ref<blockstore::BlockStore2> CreateIntegrityEncryptedBlockStore(cpputils::unique_ref<blockstore::BlockStore2> blockStore, const LocalStateDir& localStateDir, CryConfigFile *configFile, uint32_t myClientId, bool allowIntegrityViolations, bool missingBlockIsIntegrityViolation, std::function<void()> onIntegrityViolation);
  static cpputils::unique_ref<blockstore::BlockStore2> CreateEncryptedBlockStore(const CryConfig &config, cpputils::unique_ref<blockstore::BlockStore2> baseBlockStore);

  struct BlobWithAncestors {
    cpputils::unique_ref<parallelaccessfsblobstore::FsBlobRef> blob;
    boost::optional<cpputils::unique_ref<parallelaccessfsblobstore::DirBlobRef>> parent;
  };
  boost::optional<BlobWithAncestors> LoadBlobWithAncestors(const boost::filesystem::path &path, std::function<void (const blockstore::BlockId&)> ancestor_callback);

  DISALLOW_COPY_AND_ASSIGN(CryDevice);
};

}

#endif
