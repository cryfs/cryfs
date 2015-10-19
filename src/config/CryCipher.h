#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCIPHER_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYCIPHER_H

#include <vector>
#include <string>
#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <messmer/blockstore/interface/BlockStore.h>

namespace cryfs {

class CryCipher {
public:
    virtual ~CryCipher() {}

    virtual const std::string &cipherName() const = 0;
    virtual const boost::optional<std::string> &warning() const = 0;
    virtual cpputils::unique_ref<blockstore::BlockStore> createEncryptedBlockstore(cpputils::unique_ref<blockstore::BlockStore> baseBlockStore, const std::string &encKey) const = 0;
    virtual std::string createKey() const = 0;
};

class CryCiphers {
public:
    static std::vector<std::string> supportedCipherNames();

    static const CryCipher& find(const std::string &cipherName);

private:
    static const std::string INTEGRITY_WARNING;

    static const std::vector<std::shared_ptr<CryCipher>> SUPPORTED_CIPHERS;
};

}

#endif
