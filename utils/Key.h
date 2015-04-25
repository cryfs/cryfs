#pragma once
#ifndef BLOCKSTORE_UTILS_KEY_H_
#define BLOCKSTORE_UTILS_KEY_H_

#include <string>
#include <messmer/cpp-utils/data/FixedSizeData.h>

namespace blockstore {

// A key here is NOT a key for encryption, but a key as used in key->value mappings ("access handle for a block").
using Key = cpputils::FixedSizeData<16>;

}

namespace std {
  //Allow using blockstore::Key in std::unordered_map / std::unordered_set
  template <> struct hash<blockstore::Key> {
    size_t operator()(const blockstore::Key &key) const {
      //Keys are random, so it is enough to use the first few bytes as a hash
      return *(size_t*)(key.data());
    }
  };

  //Allow using blockstore::Key in std::map / std::set
  template <> struct less<blockstore::Key> {
    bool operator()(const blockstore::Key &lhs, const blockstore::Key &rhs) const {
      return 0 > std::memcmp(lhs.data(), rhs.data(), blockstore::Key::BINARY_LENGTH);
    }
  };
}

#endif
