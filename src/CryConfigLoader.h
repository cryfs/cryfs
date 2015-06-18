#pragma once
#ifndef MESSMER_CRYFS_SRC_CRYCONFIGLOADER_H_
#define MESSMER_CRYFS_SRC_CRYCONFIGLOADER_H_

#include <messmer/cpp-utils/unique_ref.h>
#include <boost/filesystem/path.hpp>
#include "CryConfig.h"
#include <messmer/blockstore/implementations/encrypted/ciphers/AES256_GCM.h>

namespace cryfs {

class CryConfigLoader {
public:
  using Cipher = blockstore::encrypted::AES256_GCM;

  static cpputils::unique_ref<CryConfig> loadOrCreate(const boost::filesystem::path &filename);

  static cpputils::unique_ref<CryConfig> createNew(const boost::filesystem::path &filename);
  static boost::optional<cpputils::unique_ref<CryConfig>> loadExisting(const boost::filesystem::path &filename);

  //This method is only for testing purposes, because creating weak keys is much faster than creating strong keys.
  static cpputils::unique_ref<CryConfig> loadOrCreateWithWeakKey(const boost::filesystem::path &filename);
  static cpputils::unique_ref<CryConfig> createNewWithWeakKey(const boost::filesystem::path &filename);

private:
  static void _initializeConfig(CryConfig *config);
  static void _generateEncKey(CryConfig *config);
  static void _generateRootBlobKey(CryConfig *config);

  static void _initializeConfigWithWeakKey(CryConfig *config);
  static void _generateWeakEncKey(CryConfig *config);
};

}

#endif
