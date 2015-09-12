#include "CryCipher.h"

#include <messmer/blockstore/implementations/encrypted/ciphers/ciphers.h>
#include <messmer/blockstore/implementations/encrypted/EncryptedBlockStore.h>

using std::vector;
using std::string;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using blockstore::BlockStore;
using std::shared_ptr;
using std::make_shared;
using boost::optional;
using boost::none;

using namespace cryfs;
using namespace blockstore::encrypted;

template<typename Cipher>
class CryCipherInstance: public CryCipher {
public:
    BOOST_CONCEPT_ASSERT((CipherConcept<Cipher>));

    CryCipherInstance(const std::string &cipherName, const optional<string> warning = none): _cipherName(cipherName), _warning(warning) {
    }

    const string &cipherName() const override {
        return _cipherName;
    }

    const optional<string> &warning() const override {
        return _warning;
    }

    unique_ref<BlockStore> createEncryptedBlockstore(unique_ref<BlockStore> baseBlockStore, const string &encKey) const override {
        return make_unique_ref<EncryptedBlockStore<Cipher>>(std::move(baseBlockStore), Cipher::EncryptionKey::FromString(encKey));
    }

    string createKey() const override {
        return Cipher::EncryptionKey::CreateOSRandom().ToString();
    }

private:
    string _cipherName;
    optional<string> _warning;
};

const string INTEGRITY_WARNING = "This cipher does not ensure integrity.";

//We have to use shared_ptr instead of unique_ref, because c++ initializer_list needs copyable values
const vector<shared_ptr<CryCipher>> CryCiphers::SUPPORTED_CIPHERS = {
        make_shared<CryCipherInstance<AES256_GCM>>("aes-256-gcm"),
        make_shared<CryCipherInstance<AES256_CFB>>("aes-256-cfb", INTEGRITY_WARNING),
        make_shared<CryCipherInstance<AES128_GCM>>("aes-128-gcm"),
        make_shared<CryCipherInstance<AES128_CFB>>("aes-128-cfb", INTEGRITY_WARNING),
        make_shared<CryCipherInstance<Twofish256_GCM>>("twofish-256-gcm"),
        make_shared<CryCipherInstance<Twofish256_CFB>>("twofish-256-cfb", INTEGRITY_WARNING),
        make_shared<CryCipherInstance<Twofish128_GCM>>("twofish-128-gcm"),
        make_shared<CryCipherInstance<Twofish128_CFB>>("twofish-128-cfb", INTEGRITY_WARNING),
        make_shared<CryCipherInstance<Serpent256_GCM>>("serpent-256-gcm"),
        make_shared<CryCipherInstance<Serpent256_CFB>>("serpent-256-cfb", INTEGRITY_WARNING),
        make_shared<CryCipherInstance<Serpent128_GCM>>("serpent-128-gcm"),
        make_shared<CryCipherInstance<Serpent128_CFB>>("serpent-128-cfb", INTEGRITY_WARNING),
        make_shared<CryCipherInstance<Cast256_GCM>>("cast-256-gcm"),
        make_shared<CryCipherInstance<Cast256_CFB>>("cast-256-cfb", INTEGRITY_WARNING),
        make_shared<CryCipherInstance<Mars448_GCM>>("mars-448-gcm"),
        make_shared<CryCipherInstance<Mars448_CFB>>("mars-448-cfb", INTEGRITY_WARNING),
        make_shared<CryCipherInstance<Mars256_GCM>>("mars-256-gcm"),
        make_shared<CryCipherInstance<Mars256_CFB>>("mars-256-cfb", INTEGRITY_WARNING),
        make_shared<CryCipherInstance<Mars128_GCM>>("mars-128-gcm"),
        make_shared<CryCipherInstance<Mars128_CFB>>("mars-128-cfb", INTEGRITY_WARNING)
};

const CryCipher& CryCiphers::find(const string &cipherName) {
    auto found = std::find_if(CryCiphers::SUPPORTED_CIPHERS.begin(), CryCiphers::SUPPORTED_CIPHERS.end(),
                              [cipherName] (const auto& element) {
                                  return element->cipherName() == cipherName;
                              });
    ASSERT(found != CryCiphers::SUPPORTED_CIPHERS.end(), "Unknown Cipher");
    return **found;
}

vector<string> CryCiphers::supportedCipherNames() {
    vector<string> result;
    for (const auto& cipher : CryCiphers::SUPPORTED_CIPHERS) {
        result.push_back(cipher->cipherName());
    }
    return result;
}