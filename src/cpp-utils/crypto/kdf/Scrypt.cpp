#include "Scrypt.h"
#include <openssl/core_dispatch.h>
#include <openssl/core_names.h>
#include <openssl/params.h>
#include <openssl/kdf.h>

using std::string;

namespace cpputils {

constexpr SCryptSettings SCrypt::ParanoidSettings;
constexpr SCryptSettings SCrypt::DefaultSettings;
constexpr SCryptSettings SCrypt::TestSettings;

namespace {
EncryptionKey _derive(size_t keySize, const std::string& password, const SCryptParameters& kdfParameters) {
    auto result = EncryptionKey::Null(keySize);

    EVP_KDF *kdf = EVP_KDF_fetch(nullptr, "SCRYPT", nullptr);
    if (!kdf) {
        throw std::runtime_error("EVP_KDF_fetch failed");
    }
    EVP_KDF_CTX *kctx = EVP_KDF_CTX_new(kdf);
    if (!kctx) {
        throw std::runtime_error("EVP_KDF_CTX_new failed");
    }
    EVP_KDF_free(kdf);

    OSSL_PARAM params[6], *par = params;
    string password_clone = password;
    Data salt_clone = kdfParameters.salt().copy();
    uint64_t N = kdfParameters.n();
    uint32_t r = kdfParameters.r();
    uint32_t p = kdfParameters.p();
    *par++ = OSSL_PARAM_construct_octet_string(OSSL_KDF_PARAM_PASSWORD, const_cast<char*>(password_clone.data()), password.size());
    *par++ = OSSL_PARAM_construct_octet_string(OSSL_KDF_PARAM_SALT, const_cast<void*>(salt_clone.data()), kdfParameters.salt().size());
    *par++ = OSSL_PARAM_construct_uint64(OSSL_KDF_PARAM_SCRYPT_N, &N);
    *par++ = OSSL_PARAM_construct_uint32(OSSL_KDF_PARAM_SCRYPT_R, &r);
    *par++ = OSSL_PARAM_construct_uint32(OSSL_KDF_PARAM_SCRYPT_P, &p);
    *par = OSSL_PARAM_construct_end();

    if (EVP_KDF_derive(kctx, static_cast<unsigned char*>(result.data()), result.binaryLength(), params) <= 0) {
        throw std::runtime_error("EVP_KDF_derive failed");
    }

    EVP_KDF_CTX_free(kctx);

    return result;
}

SCryptParameters _createNewSCryptParameters(const SCryptSettings& settings) {
    return SCryptParameters(Random::PseudoRandom().get(settings.SALT_LEN), settings.N, settings.r, settings.p);
}
}

SCrypt::SCrypt(const SCryptSettings& settingsForNewKeys)
        :_settingsForNewKeys(settingsForNewKeys) {
}

EncryptionKey SCrypt::deriveExistingKey(size_t keySize, const std::string& password, const Data& kdfParameters) {
    const SCryptParameters parameters = SCryptParameters::deserialize(kdfParameters);
    auto key = _derive(keySize, password, parameters);
    return key;
}

SCrypt::KeyResult SCrypt::deriveNewKey(size_t keySize, const std::string& password) {
    const SCryptParameters kdfParameters = _createNewSCryptParameters(_settingsForNewKeys);
    auto key = _derive(keySize, password, kdfParameters);
    return SCrypt::KeyResult {
        key,
        kdfParameters.serialize()
    };
}
}
