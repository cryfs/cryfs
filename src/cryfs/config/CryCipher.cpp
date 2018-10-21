#include "CryCipher.h"

#include <cpp-utils/crypto/symmetric/ciphers.h>
#include <blockstore/implementations/encrypted/EncryptedBlockStore2.h>
#include "crypto/inner/ConcreteInnerEncryptor.h"

using std::vector;
using std::string;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using blockstore::BlockStore2;
using std::shared_ptr;
using std::make_shared;
using boost::optional;
using boost::none;
using blockstore::encrypted::EncryptedBlockStore2;

using namespace cryfs;
using namespace cpputils;

constexpr size_t CryCiphers::MAX_KEY_SIZE;

template<typename Cipher>
class CryCipherInstance: public CryCipher {
public:
    BOOST_CONCEPT_ASSERT((CipherConcept<Cipher>));

    static_assert(Cipher::KEYSIZE <= CryCiphers::MAX_KEY_SIZE, "The key size for this cipher is too large. Please modify CryCiphers::MAX_KEY_SIZE");

    CryCipherInstance(const optional<string> warning = none): _warning(warning) {
    }

    string cipherName() const override {
        return Cipher::NAME;
    }

    const optional<string> &warning() const override {
        return _warning;
    }

    unique_ref<BlockStore2> createEncryptedBlockstore(unique_ref<BlockStore2> baseBlockStore, const string &encKey) const override {
        return make_unique_ref<EncryptedBlockStore2<Cipher>>(std::move(baseBlockStore), Cipher::EncryptionKey::FromString(encKey));
    }

    string createKey(cpputils::RandomGenerator &randomGenerator) const override {
        return Cipher::EncryptionKey::CreateKey(randomGenerator, Cipher::KEYSIZE).ToString();
    }

    unique_ref<InnerEncryptor> createInnerConfigEncryptor(const EncryptionKey& key) const override {
        ASSERT(key.binaryLength() == CryCiphers::MAX_KEY_SIZE, "Wrong key size");
        return make_unique_ref<ConcreteInnerEncryptor<Cipher>>(key.take(Cipher::KEYSIZE));
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
#if CRYPTOPP_VERSION != 564
        make_shared<CryCipherInstance<Mars448_GCM>>(),
        make_shared<CryCipherInstance<Mars448_CFB>>(INTEGRITY_WARNING),
#endif
        make_shared<CryCipherInstance<Mars256_GCM>>(),
        make_shared<CryCipherInstance<Mars256_CFB>>(INTEGRITY_WARNING),
        make_shared<CryCipherInstance<Mars128_GCM>>(),
        make_shared<CryCipherInstance<Mars128_CFB>>(INTEGRITY_WARNING)
};

const CryCipher& CryCiphers::find(const string &cipherName) {
    auto found = std::find_if(CryCiphers::SUPPORTED_CIPHERS.begin(), CryCiphers::SUPPORTED_CIPHERS.end(),
                              [cipherName] (const std::shared_ptr<CryCipher>& element) {
                                  return element->cipherName() == cipherName;
                              });
    ASSERT(found != CryCiphers::SUPPORTED_CIPHERS.end(), "Unknown Cipher: "+cipherName);
    return **found;
}

vector<string> CryCiphers::_buildSupportedCipherNames() {
	vector<string> result;
	for (const auto& cipher : CryCiphers::SUPPORTED_CIPHERS) {
		result.push_back(cipher->cipherName());
	}
	return result;
}

const vector<string>& CryCiphers::supportedCipherNames() {
	static vector<string> supportedCipherNames = _buildSupportedCipherNames();
	return supportedCipherNames;
}
