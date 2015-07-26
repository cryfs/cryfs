#pragma once
#ifndef MESSMER_CRYFS_SRC_CRYCONFIGLOADER_H_
#define MESSMER_CRYFS_SRC_CRYCONFIGLOADER_H_

#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <boost/filesystem/path.hpp>
#include "CryConfig.h"
#include <messmer/blockstore/implementations/encrypted/ciphers/AES256_GCM.h>
#include "utils/Console.h"

namespace cryfs {

class CryConfigLoader {
public:
  //TODO Get rid of this and use dynamically configured Cipher instead
  using Cipher = blockstore::encrypted::AES256_GCM;

  CryConfigLoader();
  explicit CryConfigLoader(cpputils::unique_ref<Console> console);

  cpputils::unique_ref<CryConfig> loadOrCreate(const boost::filesystem::path &filename);

  cpputils::unique_ref<CryConfig> createNew(const boost::filesystem::path &filename);
  boost::optional<cpputils::unique_ref<CryConfig>> loadExisting(const boost::filesystem::path &filename);

  //This method is only for testing purposes, because creating weak keys is much faster than creating strong keys.
  cpputils::unique_ref<CryConfig> loadOrCreateWithWeakKey(const boost::filesystem::path &filename);
  cpputils::unique_ref<CryConfig> createNewWithWeakKey(const boost::filesystem::path &filename);

private:
  void _initializeConfig(CryConfig *config);
  void _generateCipher(CryConfig *config);
  void _generateEncKey(CryConfig *config);
  void _generateRootBlobKey(CryConfig *config);

  void _initializeConfigWithWeakKey(CryConfig *config);  // TODO Rename to _initializeConfigForTest
  void _generateWeakEncKey(CryConfig *config); // TODO Rename to _generateTestEncKey
  void _generateTestCipher(CryConfig *config);

  cpputils::unique_ref<Console> _console;
};

}

#endif
