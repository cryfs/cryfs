#pragma once
#ifndef MESSMER_BLOCKSTORE_UTILS_BLOCKID_H_
#define MESSMER_BLOCKSTORE_UTILS_BLOCKID_H_

#include "IdWrapper.h"
#include <fspp/fs_interface/Types.h>

namespace blockstore {

struct _BlockIdTag final {};
// TODO Split from a BlobId (i.e. IdWrapper<BlobIdTag>)
using BlockId = IdWrapper<_BlockIdTag>;

}

#endif
