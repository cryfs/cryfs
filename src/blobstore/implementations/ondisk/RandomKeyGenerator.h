#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONDISK_RANDOMKEYGENERATOR_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONDISK_RANDOMKEYGENERATOR_H_

#include "Data.h"

#include "fspp/utils/macros.h"
#include <memory>

namespace CryptoPP {
class AutoSeededRandomPool;
}

namespace blobstore {
namespace ondisk {

// Creates random keys for use as block access handles.
// A key here is NOT a key for encryption, but a key as used in key->value mappings ("access handle for a block").
class RandomKeyGenerator {
public:
  virtual ~RandomKeyGenerator();

  static constexpr unsigned int KEYLENGTH_ENTROPY = 16; // random bytes in the key
  static constexpr unsigned int KEYLENGTH = KEYLENGTH_ENTROPY * 2;

  static RandomKeyGenerator &singleton();

  std::string create();

private:
  RandomKeyGenerator();

  std::unique_ptr<CryptoPP::AutoSeededRandomPool> _randomPool;

  DISALLOW_COPY_AND_ASSIGN(RandomKeyGenerator);
};

} /* namespace ondisk */
} /* namespace blobstore */

#endif
