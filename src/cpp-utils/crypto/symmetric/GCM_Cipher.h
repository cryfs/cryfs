#pragma once
#ifndef MESSMER_CPPUTILS_CRYPTO_SYMMETRIC_GCMCIPHER_H_
#define MESSMER_CPPUTILS_CRYPTO_SYMMETRIC_GCMCIPHER_H_

#include "AEAD_Cipher.h"
#include <vendor_cryptopp/gcm.h>

namespace cpputils {

template<typename BlockCipher, unsigned int KeySize>
using GCM_Cipher = AEADCipher<CryptoPP::GCM<BlockCipher, CryptoPP::GCM_64K_Tables>, KeySize, BlockCipher::BLOCKSIZE, 16>;
    
}

#endif
