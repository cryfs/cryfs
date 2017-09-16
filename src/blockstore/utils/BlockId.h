#pragma once
#ifndef MESSMER_BLOCKSTORE_UTILS_KEY_H_
#define MESSMER_BLOCKSTORE_UTILS_KEY_H_

#include "IdWrapper.h"

namespace blockstore {

struct _BlockIdTag final {};
// A key here is NOT a key for encryption, but a key as used in key->value mappings ("access handle for a block").
// TODO Rename to BlockId and split from a BlobId (i.e. IdWrapper<BlobIdTag>)
using BlockId = IdWrapper<_BlockIdTag>;

}

DEFINE_IDWRAPPER(blockstore::BlockId);

#endif
