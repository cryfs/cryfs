#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_VERSIONCOUNTINGBLOCKSTORE_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_VERSIONCOUNTINGBLOCKSTORE_H_

#include "../../interface/BlockStore.h"
#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/cast.h>
#include "KnownBlockVersions.h"
#include <iostream>

namespace blockstore {
namespace versioncounting {

class VersionCountingBlockStore final: public BlockStore {
public:
  VersionCountingBlockStore(cpputils::unique_ref<BlockStore> baseBlockStore, const boost::filesystem::path &integrityFilePath, uint32_t myClientId, bool missingBlockIsIntegrityViolation);

  Key createKey() override;
  boost::optional<cpputils::unique_ref<Block>> tryCreate(const Key &key, cpputils::Data data) override;
  cpputils::unique_ref<Block> overwrite(const blockstore::Key &key, cpputils::Data data) override;
  boost::optional<cpputils::unique_ref<Block>> load(const Key &key) override;
  void remove(const Key &key) override;
  void removeIfExists(const Key &key) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
  void forEachBlock(std::function<void (const Key &)> callback) const override;
  bool exists(const Key &key) const override;

  void integrityViolationDetected(const std::string &reason) const;
  KnownBlockVersions *knownBlockVersions();

#ifndef CRYFS_NO_COMPATIBILITY
  static void migrateFromBlockstoreWithoutVersionNumbers(BlockStore *baseBlockStore, const boost::filesystem::path &integrityFilePath, uint32_t myClientId);
#endif

private:
  cpputils::unique_ref<BlockStore> _baseBlockStore;
  KnownBlockVersions _knownBlockVersions;
  const bool _missingBlockIsIntegrityViolation;
  mutable bool _integrityViolationDetected;

  void _checkNoPastIntegrityViolations();

  DISALLOW_COPY_AND_ASSIGN(VersionCountingBlockStore);
};

inline KnownBlockVersions *VersionCountingBlockStore::knownBlockVersions() {
  return &_knownBlockVersions;
}

}
}

#endif
