#pragma once
#ifndef BLOCKSTORE_UTILS_KEY_H_
#define BLOCKSTORE_UTILS_KEY_H_

#include <string>
#include "FixedSizeData.h"

namespace blockstore {

// A key here is NOT a key for encryption, but a key as used in key->value mappings ("access handle for a block").
using Key = FixedSizeData<16>;

}

#endif
