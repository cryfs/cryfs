#pragma once
#ifndef MESSMER_BLOCKSTORE_UTILS_BLOCKID_H_
#define MESSMER_BLOCKSTORE_UTILS_BLOCKID_H_

#include "IdWrapper.h"

namespace blockstore {

struct BlockIdTag final {};
// TODO Split from a BlobId (i.e. IdWrapper<BlobIdTag>)
using BlockId = IdWrapper<BlockIdTag>;

}

DEFINE_IDWRAPPER(blockstore::BlockId);

#endif
