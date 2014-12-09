#pragma once
#ifndef BLOCKSTORE_UTILS_KEY_H_
#define BLOCKSTORE_UTILS_KEY_H_

#include <string>

namespace blockstore {

// A key here is NOT a key for encryption, but a key as used in key->value mappings ("access handle for a block").
class Key {
public:
  //Non-virtual destructor because we want Key objects to be small
  ~Key();

  static constexpr unsigned int KEYLENGTH_BINARY = 16;
  static constexpr unsigned int KEYLENGTH_STRING = 2 * KEYLENGTH_BINARY; // Hex encoding

  static Key CreateRandomKey();
  static Key CreateDummyKey();

  static Key FromString(const std::string &key);
  std::string AsString() const;

  const unsigned char *data() const;

private:
  Key();

  unsigned char _key[KEYLENGTH_BINARY];
};

bool operator==(const Key &lhs, const Key &rhs);
bool operator!=(const Key &lhs, const Key &rhs);

}

#endif
