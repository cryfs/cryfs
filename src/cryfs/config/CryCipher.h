#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCIPHER_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYCIPHER_H

#include <vector>
#include <string>
#include <cpp-utils/pointer/unique_ref.h>
#include <blockstore/interface/BlockStore.h>
#include <cpp-utils/random/RandomGenerator.h>
#include "crypto/inner/InnerEncryptor.h"

namespace cryfs {

class CryCipher;

class CryCiphers final {
public:
    static std::vector<std::string> supportedCipherNames();

    //A static_assert in CryCipherInstance ensures that there is no cipher with a key size larger than specified here.
    //TODO Calculate this from SUPPORTED_CIPHERS instead of setting it manually
    static constexpr size_t MAX_KEY_SIZE = 56; // in bytes

    static const CryCipher& find(const std::string &cipherName);

private:
    static const std::string INTEGRITY_WARNING;

    static const std::vector<std::shared_ptr<CryCipher>> SUPPORTED_CIPHERS;
};


class CryCipher {
public:
    virtual ~CryCipher() {}

    virtual std::string cipherName() const = 0;
    virtual const boost::optional<std::string> &warning() const = 0;
    virtual cpputils::unique_ref<blockstore::BlockStore> createEncryptedBlockstore(cpputils::unique_ref<blockstore::BlockStore> baseBlockStore, const std::string &encKey) const = 0;
    virtual std::string createKey(cpputils::RandomGenerator &randomGenerator) const = 0;
    virtual cpputils::unique_ref<InnerEncryptor> createInnerConfigEncryptor(const cpputils::FixedSizeData<CryCiphers::MAX_KEY_SIZE> &key) const = 0;
};


}

#endif
