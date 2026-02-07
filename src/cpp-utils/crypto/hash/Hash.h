#pragma once
#ifndef MESSMER_CPPUTILS_CRYPTO_HASH_HASH_H
#define MESSMER_CPPUTILS_CRYPTO_HASH_HASH_H

#include <cpp-utils/data/FixedSizeData.h>
#include <cpp-utils/data/Data.h>

namespace cpputils {
namespace hash {

using Digest = FixedSizeData<64>;
using Salt = FixedSizeData<8>;

struct Hash final {
  Digest digest;
  Salt salt;
};


Salt generateSalt();
Hash hash(const cpputils::Data& data, Salt salt);


}
}


#endif
