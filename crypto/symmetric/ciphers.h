#pragma once
#ifndef MESSMER_CPPUTILS_CRYPTO_SYMMETRIC_CIPHERS_H_
#define MESSMER_CPPUTILS_CRYPTO_SYMMETRIC_CIPHERS_H_

#include <cryptopp/cryptopp/aes.h>
#include <cryptopp/cryptopp/twofish.h>
#include <cryptopp/cryptopp/serpent.h>
#include <cryptopp/cryptopp/cast.h>
#include <cryptopp/cryptopp/mars.h>
#include "GCM_Cipher.h"
#include "CFB_Cipher.h"

#define DEFINE_CIPHER(InstanceName, StringName, Mode, Base, Keysize) \
    class InstanceName final: public Mode<Base, Keysize> {           \
    public:                                                          \
        static constexpr const char *NAME = StringName;              \
    }                                                                \


namespace cpputils {

static_assert(32 == CryptoPP::AES::MAX_KEYLENGTH, "If AES offered larger keys, we should offer a variant with it");
DEFINE_CIPHER(AES256_GCM, "aes-256-gcm", GCM_Cipher, CryptoPP::AES, 32);
DEFINE_CIPHER(AES256_CFB, "aes-256-cfb", CFB_Cipher, CryptoPP::AES, 32);
DEFINE_CIPHER(AES128_GCM, "aes-128-gcm", GCM_Cipher, CryptoPP::AES, 16);
DEFINE_CIPHER(AES128_CFB, "aes-128-cfb", CFB_Cipher, CryptoPP::AES, 16);

static_assert(32 == CryptoPP::Twofish::MAX_KEYLENGTH, "If Twofish offered larger keys, we should offer a variant with it");
DEFINE_CIPHER(Twofish256_GCM, "twofish-256-gcm", GCM_Cipher, CryptoPP::Twofish, 32);
DEFINE_CIPHER(Twofish256_CFB, "twofish-256-cfb", CFB_Cipher, CryptoPP::Twofish, 32);
DEFINE_CIPHER(Twofish128_GCM, "twofish-128-gcm", GCM_Cipher, CryptoPP::Twofish, 16);
DEFINE_CIPHER(Twofish128_CFB, "twofish-128-cfb", CFB_Cipher, CryptoPP::Twofish, 16);

static_assert(32 == CryptoPP::Serpent::MAX_KEYLENGTH, "If Serpent offered larger keys, we should offer a variant with it");
DEFINE_CIPHER(Serpent256_GCM, "serpent-256-gcm", GCM_Cipher, CryptoPP::Serpent, 32);
DEFINE_CIPHER(Serpent256_CFB, "serpent-256-cfb", CFB_Cipher, CryptoPP::Serpent, 32);
DEFINE_CIPHER(Serpent128_GCM, "serpent-128-gcm", GCM_Cipher, CryptoPP::Serpent, 16);
DEFINE_CIPHER(Serpent128_CFB, "serpent-128-cfb", CFB_Cipher, CryptoPP::Serpent, 16);

static_assert(32 == CryptoPP::CAST256::MAX_KEYLENGTH, "If Cast offered larger keys, we should offer a variant with it");
DEFINE_CIPHER(Cast256_GCM, "cast-256-gcm", GCM_Cipher, CryptoPP::CAST256, 32);
DEFINE_CIPHER(Cast256_CFB, "cast-256-cfb", CFB_Cipher, CryptoPP::CAST256, 32);

static_assert(56 == CryptoPP::MARS::MAX_KEYLENGTH, "If Mars offered larger keys, we should offer a variant with it");
DEFINE_CIPHER(Mars448_GCM, "mars-448-gcm", GCM_Cipher, CryptoPP::MARS, 56);
DEFINE_CIPHER(Mars448_CFB, "mars-448-cfb", CFB_Cipher, CryptoPP::MARS, 56);
DEFINE_CIPHER(Mars256_GCM, "mars-256-gcm", GCM_Cipher, CryptoPP::MARS, 32);
DEFINE_CIPHER(Mars256_CFB, "mars-256-cfb", CFB_Cipher, CryptoPP::MARS, 32);
DEFINE_CIPHER(Mars128_GCM, "mars-128-gcm", GCM_Cipher, CryptoPP::MARS, 16);
DEFINE_CIPHER(Mars128_CFB, "mars-128-cfb", CFB_Cipher, CryptoPP::MARS, 16);

}

#endif
