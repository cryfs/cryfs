#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYDEVICE_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYDEVICE_H_

#include <blockstore/interface/BlockStore.h>
#include <blockstore/interface/BlockStore2.h>
#include "../config/CryConfigFile.h"

#include <boost/filesystem.hpp>
#include <fspp/fs_interface/Device.h>
#include <cryfs/localstate/LocalStateDir.h>

#include "fsblobstore/FsBlobStore.h"
#include "fsblobstore/DirBlob.h"
#include "fsblobstore/FileBlob.h"
#include "fsblobstore/SymlinkBlob.h"

namespace cryfs {

class CryDevice final: public fspp::Device {
public:
  CryDevice(CryConfigFile config, cpputils::unique_ref<blockstore::BlockStore2> blockStore, const LocalStateDir& localStateDir, uint32_t myClientId, bool allowIntegrityViolations, bool missingBlockIsIntegrityViolation);

  void statfs(const boost::filesystem::path &path, struct ::statvfs *fsstat) override;

  cpputils::unique_ref<fsblobstore::FileBlob> CreateFileBlob(const blockstore::BlockId &parent);
  cpputils::unique_ref<fsblobstore::DirBlob> CreateDirBlob(const blockstore::BlockId &parent);
  cpputils::unique_ref<fsblobstore::SymlinkBlob> CreateSymlinkBlob(const boost::filesystem::path &target, const blockstore::BlockId &parent);
  cpputils::unique_ref<fsblobstore::FsBlob> LoadBlob(const blockstore::BlockId &blockId);
  struct DirBlobWithParent {
      std::shared_ptr<fsblobstore::DirBlob> blob;
      boost::optional<std::shared_ptr<fsblobstore::DirBlob>> parent;
  };
  DirBlobWithParent LoadDirBlobWithParent(const boost::filesystem::path &absoutePath);
  DirBlobWithParent LoadDirBlobWithParent(const boost::filesystem::path &relativePath, std::shared_ptr<fsblobstore::FsBlob> anchor);
  void RemoveBlob(const blockstore::BlockId &blockId);

  void onFsAction(std::function<void()> callback);

  boost::optional<cpputils::unique_ref<fspp::Node>> Load(const boost::filesystem::path &path) override;
  boost::optional<cpputils::unique_ref<fspp::File>> LoadFile(const boost::filesystem::path &path) override;
  boost::optional<cpputils::unique_ref<fspp::Dir>> LoadDir(const boost::filesystem::path &path) override;
  boost::optional<cpputils::unique_ref<fspp::Symlink>> LoadSymlink(const boost::filesystem::path &path) override;

  void callFsActionCallbacks() const;

  uint64_t numBlocks() const;

private:

  cpputils::unique_ref<fsblobstore::FsBlobStore> _fsBlobStore;

  blockstore::BlockId _rootBlobId;
  std::vector<std::function<void()>> _onFsAction;

  blockstore::BlockId GetOrCreateRootBlobId(CryConfigFile *config);
  blockstore::BlockId CreateRootBlobAndReturnId();
  static cpputils::unique_ref<fsblobstore::FsBlobStore> CreateFsBlobStore(cpputils::unique_ref<blockstore::BlockStore2> blockStore, CryConfigFile *configFile, const LocalStateDir& localStateDir, uint32_t myClientId, bool allowIntegrityViolations, bool missingBlockIsIntegrityViolation);
#ifndef CRYFS_NO_COMPATIBILITY
  static cpputils::unique_ref<fsblobstore::FsBlobStore> MigrateOrCreateFsBlobStore(cpputils::unique_ref<blobstore::BlobStore> blobStore, CryConfigFile *configFile);
#endif
  static cpputils::unique_ref<blobstore::BlobStore> CreateBlobStore(cpputils::unique_ref<blockstore::BlockStore2> blockStore, const LocalStateDir& localStateDir, CryConfigFile *configFile, uint32_t myClientId, bool allowIntegrityViolations, bool missingBlockIsIntegrityViolation);
  static cpputils::unique_ref<blockstore::BlockStore2> CreateIntegrityEncryptedBlockStore(cpputils::unique_ref<blockstore::BlockStore2> blockStore, const LocalStateDir& localStateDir, CryConfigFile *configFile, uint32_t myClientId, bool allowIntegrityViolations, bool missingBlockIsIntegrityViolation);
  static cpputils::unique_ref<blockstore::BlockStore2> CreateEncryptedBlockStore(const CryConfig &config, cpputils::unique_ref<blockstore::BlockStore2> baseBlockStore);

  struct BlobWithParent {
      std::shared_ptr<fsblobstore::FsBlob> blob;
      boost::optional<std::shared_ptr<fsblobstore::DirBlob>> parent;
  };
  BlobWithParent LoadBlobWithParent(const boost::filesystem::path &absolutePath);
  BlobWithParent LoadBlobWithParent(const boost::filesystem::path &relativePath, std::shared_ptr<fsblobstore::FsBlob> anchor);
  DirBlobWithParent makeDirBlobWithParent(BlobWithParent blob);

  DISALLOW_COPY_AND_ASSIGN(CryDevice);
};

}

#endif
