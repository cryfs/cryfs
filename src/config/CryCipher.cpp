#include "CryCipher.h"

#include <messmer/cpp-utils/crypto/symmetric/ciphers.h>
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
using blockstore::encrypted::EncryptedBlockStore;

using namespace cryfs;
using namespace cpputils;

template<typename Cipher>
class CryCipherInstance: public CryCipher {
public:
    BOOST_CONCEPT_ASSERT((CipherConcept<Cipher>));

    CryCipherInstance(const optional<string> warning = none): _warning(warning) {
    }

    string cipherName() const override {
        return Cipher::NAME;
    }

    const optional<string> &warning() const override {
        return _warning;
    }

    unique_ref<BlockStore> createEncryptedBlockstore(unique_ref<BlockStore> baseBlockStore, const string &encKey) const override {
        return make_unique_ref<EncryptedBlockStore<Cipher>>(std::move(baseBlockStore), Cipher::EncryptionKey::FromString(encKey));
    }

    string createKey(cpputils::RandomGenerator &randomGenerator) const override {
        return Cipher::CreateKey(randomGenerator).ToString();
    }

private:
    optional<string> _warning;
};

const string CryCiphers::INTEGRITY_WARNING = "This cipher does not ensure integrity.";

//We have to use shared_ptr instead of unique_ref, because c++ initializer_list needs copyable values
const vector<shared_ptr<CryCipher>> CryCiphers::SUPPORTED_CIPHERS = {
        make_shared<CryCipherInstance<AES256_GCM>>(),
        make_shared<CryCipherInstance<AES256_CFB>>(INTEGRITY_WARNING),
        make_shared<CryCipherInstance<AES128_GCM>>(),
        make_shared<CryCipherInstance<AES128_CFB>>(INTEGRITY_WARNING),
        make_shared<CryCipherInstance<Twofish256_GCM>>(),
        make_shared<CryCipherInstance<Twofish256_CFB>>(INTEGRITY_WARNING),
        make_shared<CryCipherInstance<Twofish128_GCM>>(),
        make_shared<CryCipherInstance<Twofish128_CFB>>(INTEGRITY_WARNING),
        make_shared<CryCipherInstance<Serpent256_GCM>>(),
        make_shared<CryCipherInstance<Serpent256_CFB>>(INTEGRITY_WARNING),
        make_shared<CryCipherInstance<Serpent128_GCM>>(),
        make_shared<CryCipherInstance<Serpent128_CFB>>(INTEGRITY_WARNING),
        make_shared<CryCipherInstance<Cast256_GCM>>(),
        make_shared<CryCipherInstance<Cast256_CFB>>(INTEGRITY_WARNING),
        make_shared<CryCipherInstance<Mars448_GCM>>(),
        make_shared<CryCipherInstance<Mars448_CFB>>(INTEGRITY_WARNING),
        make_shared<CryCipherInstance<Mars256_GCM>>(),
        make_shared<CryCipherInstance<Mars256_CFB>>(INTEGRITY_WARNING),
        make_shared<CryCipherInstance<Mars128_GCM>>(),
        make_shared<CryCipherInstance<Mars128_CFB>>(INTEGRITY_WARNING)
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