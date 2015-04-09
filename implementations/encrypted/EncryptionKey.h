#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTIONKEY_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ENCRYPTED_ENCRYPTIONKEY_H_

#include "../../utils/FixedSizeData.h"

namespace blockstore {
namespace encrypted {

using EncryptionKey = FixedSizeData<32>;

}
}

#endif
