#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTIONKEY_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTIONKEY_H_

#include "../../utils/FixedSizeData.h"

namespace blockstore {
namespace encrypted {

//TODO Does EncryptionKey::GenerateRandom() use a PseudoRandomGenerator? Would be better to use real randomness.
using EncryptionKey = FixedSizeData<CryptoPP::AES::MAX_KEYLENGTH>;

}
}

#endif
