#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_INTEGRITY_INTEGRITYBLOCKSTORE2_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_INTEGRITY_INTEGRITYBLOCKSTORE2_H_

#include "../../interface/BlockStore2.h"
#include <cpp-utils/macros.h>
#include "KnownBlockVersions.h"
#include "IntegrityViolationError.h"

namespace blockstore {
namespace integrity {

//TODO Format version headers

// This blockstore implements integrity measures.
// It depends on being used on top of an encrypted block store that protects integrity of the block contents (i.e. uses an authenticated cipher).
class IntegrityBlockStore2 final: public BlockStore2 {
public:
  IntegrityBlockStore2(cpputils::unique_ref<BlockStore2> baseBlockStore, const boost::filesystem::path &integrityFilePath, uint32_t myClientId, bool missingBlockIsIntegrityViolation);

  bool tryCreate(const Key &key, const cpputils::Data &data) override;
  bool remove(const Key &key) override;
  boost::optional<cpputils::Data> load(const Key &key) const override;
  void store(const Key &key, const cpputils::Data &data) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
  void forEachBlock(std::function<void (const Key &)> callback) const override;

private:
  // This format version is prepended to blocks to allow future versions to have compatibility.
#ifndef CRYFS_NO_COMPATIBILITY
  static constexpr uint16_t FORMAT_VERSION_HEADER_OLD = 0;
#endif
  static constexpr uint16_t FORMAT_VERSION_HEADER = 1;

public:
  static constexpr uint64_t VERSION_ZERO = 0;
  static constexpr unsigned int ID_HEADER_OFFSET = sizeof(FORMAT_VERSION_HEADER);
  static constexpr unsigned int CLIENTID_HEADER_OFFSET = ID_HEADER_OFFSET + Key::BINARY_LENGTH;
  static constexpr unsigned int VERSION_HEADER_OFFSET = CLIENTID_HEADER_OFFSET + sizeof(uint32_t);
  static constexpr unsigned int HEADER_LENGTH = VERSION_HEADER_OFFSET + sizeof(VERSION_ZERO);

#ifndef CRYFS_NO_COMPATIBILITY
  static void migrateFromBlockstoreWithoutVersionNumbers(BlockStore2 *baseBlockStore, const boost::filesystem::path &integrityFilePath, uint32_t myClientId);
  static void migrateBlockFromBlockstoreWithoutVersionNumbers(BlockStore2* baseBlockStore, const blockstore::Key& key, KnownBlockVersions *knownBlockVersions);
#endif

private:

  static cpputils::Data _prependHeaderToData(const Key& key, uint32_t myClientId, uint64_t version, const cpputils::Data &data);
  void _checkHeader(const Key &key, const cpputils::Data &data) const;
  void _checkFormatHeader(const cpputils::Data &data) const;
  void _checkIdHeader(const Key &expectedKey, const cpputils::Data &data) const;
  void _checkVersionHeader(const Key &key, const cpputils::Data &data) const;
  static uint16_t _readFormatHeader(const cpputils::Data &data);
  static uint32_t _readClientId(const cpputils::Data &data);
  static Key _readBlockId(const cpputils::Data &data);
  static uint64_t _readVersion(const cpputils::Data &data);
#ifndef CRYFS_NO_COMPATIBILITY
  static cpputils::Data _migrateBlock(const Key &key, const cpputils::Data &data);
#endif
  static cpputils::Data _removeHeader(const cpputils::Data &data);
  void _checkNoPastIntegrityViolations() const;
  void integrityViolationDetected(const std::string &reason) const;

  cpputils::unique_ref<BlockStore2> _baseBlockStore;
  mutable KnownBlockVersions _knownBlockVersions;
  const bool _missingBlockIsIntegrityViolation;
  mutable bool _integrityViolationDetected;

  DISALLOW_COPY_AND_ASSIGN(IntegrityBlockStore2);
};

}
}

#endif
