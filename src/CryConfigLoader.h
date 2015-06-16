#pragma once
#ifndef MESSMER_CRYFS_SRC_CRYCONFIGLOADER_H_
#define MESSMER_CRYFS_SRC_CRYCONFIGLOADER_H_

#include <memory>
#include <boost/filesystem/path.hpp>
#include "CryConfig.h"
#include <messmer/blockstore/implementations/encrypted/ciphers/AES256_GCM.h>

namespace cryfs {

class CryConfigLoader {
public:
  using Cipher = blockstore::encrypted::AES256_GCM;

  static std::unique_ptr<CryConfig> loadOrCreate(const boost::filesystem::path &filename);

  static std::unique_ptr<CryConfig> createNew(const boost::filesystem::path &filename);
  static std::unique_ptr<CryConfig> loadExisting(const boost::filesystem::path &filename);

private:
  static void _initializeConfig(CryConfig *config);
  static void _generateEncKey(CryConfig *config);
  static void _generateRootBlobKey(CryConfig *config);
};

}

#endif
