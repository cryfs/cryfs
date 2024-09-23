#include "FsBlobRef.h"
#include "CachingFsBlobStore.h"
#include <utility>

namespace cryfs {
namespace cachingfsblobstore {

FsBlobRef::~FsBlobRef() {
    if (_baseBlob.is_valid()) {
        _fsBlobStore->releaseForCache(std::move(_baseBlob));
    }
}

}
}
