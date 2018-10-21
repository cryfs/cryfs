#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCIPHER_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYCIPHER_H

#include <vector>
#include <string>
#include <cpp-utils/pointer/unique_ref.h>
#include <blockstore/interface/BlockStore2.h>
#include <cpp-utils/random/RandomGenerator.h>
#include "crypto/inner/InnerEncryptor.h"
#include <cpp-utils/crypto/symmetric/EncryptionKey.h>

namespace cryfs {

class CryCipher;

class CryCiphers final {
public:
    static const std::vector<std::string>& supportedCipherNames();

    //A static_assert in CryCipherInstance ensures that there is no cipher with a key size larger than specified here.
    //TODO Calculate this from SUPPORTED_CIPHERS instead of setting it manually
    static constexpr size_t MAX_KEY_SIZE = 56; // in bytes

    static const CryCipher& find(const std::string &cipherName);

private:
    static const std::string INTEGRITY_WARNING;

    static const std::vector<std::shared_ptr<CryCipher>> SUPPORTED_CIPHERS;

	static std::vector<std::string> _buildSupportedCipherNames();
};


class CryCipher {
public:
    virtual ~CryCipher() {}

    virtual std::string cipherName() const = 0;
    virtual const boost::optional<std::string> &warning() const = 0;
    virtual cpputils::unique_ref<blockstore::BlockStore2> createEncryptedBlockstore(cpputils::unique_ref<blockstore::BlockStore2> baseBlockStore, const std::string &encKey) const = 0;
    virtual std::string createKey(cpputils::RandomGenerator &randomGenerator) const = 0;
    virtual cpputils::unique_ref<InnerEncryptor> createInnerConfigEncryptor(const cpputils::EncryptionKey &key) const = 0;
};


}

#endif
