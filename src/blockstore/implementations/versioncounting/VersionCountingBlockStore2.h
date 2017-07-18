#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_VERSIONCOUNTINGBLOCKSTORE2_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_VERSIONCOUNTINGBLOCKSTORE2_H_

#include "../../interface/BlockStore2.h"
#include <cpp-utils/macros.h>
#include "KnownBlockVersions.h"
#include "IntegrityViolationError.h"

namespace blockstore {
namespace versioncounting {

//TODO Format version headers

class VersionCountingBlockStore2 final: public BlockStore2 {
public:
  VersionCountingBlockStore2(cpputils::unique_ref<BlockStore2> baseBlockStore, const boost::filesystem::path &integrityFilePath, uint32_t myClientId, bool missingBlockIsIntegrityViolation);

  boost::future<bool> tryCreate(const Key &key, const cpputils::Data &data) override;
  boost::future<bool> remove(const Key &key) override;
  boost::future<boost::optional<cpputils::Data>> load(const Key &key) const override;
  boost::future<void> store(const Key &key, const cpputils::Data &data) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
  void forEachBlock(std::function<void (const Key &)> callback) const override;

private:
  // This header is prepended to blocks to allow future versions to have compatibility.
  static constexpr uint16_t FORMAT_VERSION_HEADER = 0;

public:
  static constexpr uint64_t VERSION_ZERO = 0;
  static constexpr unsigned int CLIENTID_HEADER_OFFSET = sizeof(FORMAT_VERSION_HEADER);
  static constexpr unsigned int VERSION_HEADER_OFFSET = sizeof(FORMAT_VERSION_HEADER) + sizeof(uint32_t);
  static constexpr unsigned int HEADER_LENGTH = sizeof(FORMAT_VERSION_HEADER) + sizeof(uint32_t) + sizeof(VERSION_ZERO);

#ifndef CRYFS_NO_COMPATIBILITY
  static void migrateFromBlockstoreWithoutVersionNumbers(BlockStore2 *baseBlockStore, const boost::filesystem::path &integrityFilePath, uint32_t myClientId);
  static void migrateBlockFromBlockstoreWithoutVersionNumbers(BlockStore2* baseBlockStore, const blockstore::Key& key, KnownBlockVersions *knownBlockVersions);
#endif

private:

  static cpputils::Data _prependHeaderToData(uint32_t myClientId, uint64_t version, const cpputils::Data &data);
  void _checkHeader(const Key &key, const cpputils::Data &data) const;
  void _checkFormatHeader(const cpputils::Data &data) const;
  void _checkVersionHeader(const Key &key, const cpputils::Data &data) const;
  static uint32_t _readClientId(const cpputils::Data &data);
  static uint64_t _readVersion(const cpputils::Data &data);
  cpputils::Data _removeHeader(const cpputils::Data &data) const;
  void _checkNoPastIntegrityViolations() const;
  void integrityViolationDetected(const std::string &reason) const;

  cpputils::unique_ref<BlockStore2> _baseBlockStore;
  mutable KnownBlockVersions _knownBlockVersions;
  const bool _missingBlockIsIntegrityViolation;
  mutable bool _integrityViolationDetected;

  DISALLOW_COPY_AND_ASSIGN(VersionCountingBlockStore2);
};

}
}

#endif
