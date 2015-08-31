#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_CIPHERS_CIPHERS_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_CIPHERS_CIPHERS_H_

#include <cryptopp/cryptopp/aes.h>
#include <cryptopp/cryptopp/twofish.h>
#include <cryptopp/cryptopp/serpent.h>
#include <cryptopp/cryptopp/cast.h>
#include <cryptopp/cryptopp/mars.h>
#include "GCM_Cipher.h"
#include "CFB_Cipher.h"

namespace blockstore {
namespace encrypted {

static_assert(32 == CryptoPP::AES::MAX_KEYLENGTH, "If AES offered larger keys, we should offer a variant with it");
using AES256_GCM = GCM_Cipher<CryptoPP::AES, 32>;
using AES256_CFB = CFB_Cipher<CryptoPP::AES, 32>;
using AES128_GCM = GCM_Cipher<CryptoPP::AES, 16>;
using AES128_CFB = CFB_Cipher<CryptoPP::AES, 16>;

static_assert(32 == CryptoPP::Twofish::MAX_KEYLENGTH, "If Twofish offered larger keys, we should offer a variant with it");
using Twofish256_GCM = GCM_Cipher<CryptoPP::Twofish, 32>;
using Twofish256_CFB = CFB_Cipher<CryptoPP::Twofish, 32>;
using Twofish128_GCM = GCM_Cipher<CryptoPP::Twofish, 16>;
using Twofish128_CFB = CFB_Cipher<CryptoPP::Twofish, 16>;

static_assert(32 == CryptoPP::Serpent::MAX_KEYLENGTH, "If Serpent offered larger keys, we should offer a variant with it");
using Serpent256_GCM = GCM_Cipher<CryptoPP::Serpent, 32>;
using Serpent256_CFB = CFB_Cipher<CryptoPP::Serpent, 32>;
using Serpent128_GCM = GCM_Cipher<CryptoPP::Serpent, 16>;
using Serpent128_CFB = CFB_Cipher<CryptoPP::Serpent, 16>;

static_assert(32 == CryptoPP::CAST256::MAX_KEYLENGTH, "If Cast offered larger keys, we should offer a variant with it");
using Cast256_GCM = GCM_Cipher<CryptoPP::CAST256, 32>;
using Cast256_CFB = CFB_Cipher<CryptoPP::CAST256, 32>;

static_assert(56 == CryptoPP::MARS::MAX_KEYLENGTH, "If Mars offered larger keys, we should offer a variant with it");
using Mars448_GCM = GCM_Cipher<CryptoPP::MARS, 56>;
using Mars448_CFB = CFB_Cipher<CryptoPP::MARS, 56>;
using Mars256_GCM = GCM_Cipher<CryptoPP::MARS, 32>;
using Mars256_CFB = CFB_Cipher<CryptoPP::MARS, 32>;
using Mars128_GCM = GCM_Cipher<CryptoPP::MARS, 16>;
using Mars128_CFB = CFB_Cipher<CryptoPP::MARS, 16>;

}
}

#endif
